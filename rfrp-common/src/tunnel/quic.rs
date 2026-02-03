//! QUIC 协议隧道实现
//!
//! 此模块提供了基于 QUIC 协议的隧道实现，包括：
//! - `QuicSendStream` / `QuicRecvStream`: 流包装器
//! - `QuicConnection`: 连接包装器
//! - `QuicConnector`: 客户端连接器
//! - `QuicListener`: 服务端监听器

use anyhow::Result;
use async_trait::async_trait;
use quinn::{
    ClientConfig, Endpoint, ServerConfig, TransportConfig, VarInt,
    crypto::rustls::QuicClientConfig,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

use super::traits::{TunnelConnection, TunnelConnector, TunnelListener, TunnelRecvStream, TunnelSendStream};

/// QUIC 发送流包装器
pub struct QuicSendStream {
    inner: quinn::SendStream,
}

impl QuicSendStream {
    /// 创建新的 QUIC 发送流包装器
    pub fn new(inner: quinn::SendStream) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl TunnelSendStream for QuicSendStream {
    async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.inner.write_all(buf).await?;
        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        self.inner.flush().await?;
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        self.inner.finish()?;
        Ok(())
    }
}

/// QUIC 接收流包装器
pub struct QuicRecvStream {
    inner: quinn::RecvStream,
}

impl QuicRecvStream {
    /// 创建新的 QUIC 接收流包装器
    pub fn new(inner: quinn::RecvStream) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl TunnelRecvStream for QuicRecvStream {
    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.inner.read_exact(buf).await?;
        Ok(())
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<Option<usize>> {
        Ok(self.inner.read(buf).await?)
    }
}

/// QUIC 连接包装器
pub struct QuicConnection {
    inner: quinn::Connection,
}

impl QuicConnection {
    /// 创建新的 QUIC 连接包装器
    pub fn new(inner: quinn::Connection) -> Self {
        Self { inner }
    }

    /// 获取内部 quinn::Connection 引用
    pub fn inner(&self) -> &quinn::Connection {
        &self.inner
    }
}

#[async_trait]
impl TunnelConnection for QuicConnection {
    async fn open_bi(&self) -> Result<(Box<dyn TunnelSendStream>, Box<dyn TunnelRecvStream>)> {
        let (send, recv) = self.inner.open_bi().await?;
        Ok((
            Box::new(QuicSendStream::new(send)),
            Box::new(QuicRecvStream::new(recv)),
        ))
    }

    async fn accept_bi(&self) -> Result<(Box<dyn TunnelSendStream>, Box<dyn TunnelRecvStream>)> {
        let (send, recv) = self.inner.accept_bi().await?;
        Ok((
            Box::new(QuicSendStream::new(send)),
            Box::new(QuicRecvStream::new(recv)),
        ))
    }

    async fn open_uni(&self) -> Result<Box<dyn TunnelSendStream>> {
        let send = self.inner.open_uni().await?;
        Ok(Box::new(QuicSendStream::new(send)))
    }

    async fn accept_uni(&self) -> Result<Box<dyn TunnelRecvStream>> {
        let recv = self.inner.accept_uni().await?;
        Ok(Box::new(QuicRecvStream::new(recv)))
    }

    fn remote_address(&self) -> SocketAddr {
        self.inner.remote_address()
    }

    fn close_reason(&self) -> Option<String> {
        self.inner.close_reason().map(|r| r.to_string())
    }
}

/// QUIC 客户端连接器
///
/// 用于客户端连接到 QUIC 服务器，支持自签名证书（跳过验证）。
pub struct QuicConnector {
    endpoint: Endpoint,
}

impl QuicConnector {
    /// 创建新的 QUIC 连接器
    ///
    /// 配置了默认的传输参数和证书验证（跳过验证用于开发环境）。
    pub fn new() -> Result<Self> {
        // 创建传输配置
        let mut transport_config = TransportConfig::default();
        transport_config.max_concurrent_uni_streams(0u32.into());
        transport_config.keep_alive_interval(Some(Duration::from_secs(5)));
        transport_config.max_idle_timeout(Some(Duration::from_secs(60).try_into()?));

        // 创建客户端配置（跳过证书验证）
        let crypto = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(SkipVerification))
            .with_no_client_auth();

        let mut client_config = ClientConfig::new(Arc::new(QuicClientConfig::try_from(crypto)?));
        client_config.transport_config(Arc::new(transport_config));

        // 创建 QUIC 端点
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
        endpoint.set_default_client_config(client_config);

        Ok(Self { endpoint })
    }
}

#[async_trait]
impl TunnelConnector for QuicConnector {
    async fn connect(&self, addr: SocketAddr) -> Result<Box<dyn TunnelConnection>> {
        let conn = self.endpoint.connect(addr, "rfrp")?.await?;
        Ok(Box::new(QuicConnection::new(conn)))
    }
}

/// QUIC 服务端监听器
///
/// 用于服务端接受 QUIC 客户端连接。
pub struct QuicListener {
    endpoint: Endpoint,
}

impl QuicListener {
    /// 创建新的 QUIC 监听器
    ///
    /// # Arguments
    /// * `bind_addr` - 绑定地址
    /// * `cert` - TLS 证书
    /// * `key` - TLS 私钥
    /// * `idle_timeout` - 空闲超时时间（秒）
    /// * `max_streams` - 最大并发流数
    /// * `keep_alive_interval` - 心跳间隔（秒）
    pub fn new(
        bind_addr: SocketAddr,
        cert: CertificateDer<'static>,
        key: PrivateKeyDer<'static>,
        idle_timeout: u64,
        max_streams: u32,
        keep_alive_interval: u64,
    ) -> Result<Self> {
        let mut transport_config = TransportConfig::default();
        transport_config.max_concurrent_uni_streams(VarInt::from_u32(max_streams));
        transport_config.keep_alive_interval(Some(Duration::from_secs(keep_alive_interval)));
        transport_config.max_idle_timeout(Some(Duration::from_secs(idle_timeout).try_into()?));

        let mut server_config = ServerConfig::with_single_cert(
            vec![cert],
            key,
        )?;
        server_config.transport_config(Arc::new(transport_config));

        let endpoint = Endpoint::server(server_config, bind_addr)?;

        Ok(Self { endpoint })
    }
}

#[async_trait]
impl TunnelListener for QuicListener {
    async fn accept(&self) -> Result<Box<dyn TunnelConnection>> {
        loop {
            if let Some(connecting) = self.endpoint.accept().await {
                match connecting.await {
                    Ok(conn) => {
                        return Ok(Box::new(QuicConnection::new(conn)));
                    }
                    Err(e) => {
                        tracing::error!("Connection accept failed: {}", e);
                        continue;
                    }
                }
            }
        }
    }
}

/// 自定义证书验证器（跳过验证）
///
/// 仅用于开发和测试环境，生产环境应使用正确的证书验证。
#[derive(Debug)]
struct SkipVerification;

impl rustls::client::danger::ServerCertVerifier for SkipVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA1,
            rustls::SignatureScheme::ECDSA_SHA1_Legacy,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}
