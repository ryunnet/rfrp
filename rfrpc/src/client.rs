use anyhow::Result;
use quinn::{ClientConfig, Endpoint, crypto::rustls::QuicClientConfig, TransportConfig, SendStream, ConnectionError};
use rustls::pki_types::ServerName;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, error, warn, debug};

pub async fn run(server_addr: SocketAddr, token: String) -> Result<()> {
    // 创建传输配置
    let mut transport_config = TransportConfig::default();
    transport_config.max_concurrent_uni_streams(0u32.into());
    transport_config.keep_alive_interval(Some(Duration::from_secs(5)));
    transport_config.max_idle_timeout(Some(Duration::from_secs(600).try_into()?));

    // 创建客户端配置（跳过证书验证）
    let crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipVerification))
        .with_no_client_auth();

    let mut client_config = ClientConfig::new(Arc::new(QuicClientConfig::try_from(crypto)?));
    client_config.transport_config(Arc::new(transport_config));

    // 创建QUIC端点并保持引用
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);

    info!("🔧 QUIC客户端配置完成");
    info!("🌐 连接到服务器: {}", server_addr);
    info!("⏱️  空闲超时: 600秒, 心跳间隔: 5秒");

    // 连接循环，支持自动重连
    loop {
        match connect_to_server(&endpoint, server_addr, &token).await {
            Ok(_) => {
                info!("连接已关闭");
            }
            Err(e) => {
                error!("连接错误: {}", e);
            }
        }

        warn!("连接已断开，5秒后重连...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn connect_to_server(
    endpoint: &Endpoint,
    server_addr: SocketAddr,
    token: &str,
) -> Result<()> {
    // 连接到服务器
    let conn = endpoint
        .connect(server_addr, "rfrp")?
        .await?;

    info!("✅ 已连接到服务器: {}", server_addr);

    // 发送 token 进行认证
    info!("🌐 正在发送Token，进行认证: {}", token);
    match conn.open_uni().await {
        Ok(mut uni_stream) => {
            debug!("获取到流");
            let token_bytes = token.as_bytes();
            let len = token_bytes.len() as u16;
            uni_stream.write_all(&len.to_be_bytes()).await.unwrap();
            uni_stream.write_all(token_bytes).await.unwrap();
            uni_stream.finish().unwrap();

            info!("✅ 认证成功");
            info!("⏳ 等待代理请求...");

            let conn = Arc::new(conn);

            // 循环接受来自服务器的QUIC流
            loop {
                match conn.accept_bi().await {
                    Ok((quic_send, quic_recv)) => {
                        info!("📨 收到新的代理请求");

                        tokio::spawn(async move {
                            if let Err(e) = handle_proxy_stream(quic_send, quic_recv).await {
                                error!("❌ 处理代理流错误: {}", e);
                            }
                            info!("🔚 代理流已关闭");
                        });
                    }
                    Err(e) => {
                        error!("❌ 接受流失败: {}", e);
                        return Err(e.into());
                    }
                }
            }
        }
        Err(err) => {
            error!("error => {}", err);
            return Err(err.into());
        }
    }
}

async fn handle_proxy_stream(
    mut quic_send: quinn::SendStream,
    mut quic_recv: quinn::RecvStream,
) -> Result<()> {
    // 首先读取目标地址（格式：2字节长度 + 内容）
    let mut len_buf = [0u8; 2];
    quic_recv.read_exact(&mut len_buf).await?;
    let len = u16::from_be_bytes(len_buf) as usize;

    let mut addr_buf = vec![0u8; len];
    quic_recv.read_exact(&mut addr_buf).await?;
    let target_addr = String::from_utf8(addr_buf)?;

    info!("🎯 目标地址: {}", target_addr);

    // 连接到目标服务
    let mut tcp_stream = TcpStream::connect(&target_addr).await?;

    info!("🔗 已连接到目标服务");

    let (mut tcp_read, mut tcp_write) = tcp_stream.split();

    // QUIC -> TCP
    let quic_to_tcp = async {
        let mut buf = vec![0u8; 8192];
        loop {
            match quic_recv.read(&mut buf).await? {
                Some(n) => {
                    if n == 0 {
                        break;
                    }
                    tcp_write.write_all(&buf[..n]).await?;
                }
                None => break,
            }
        }
        Ok::<_, anyhow::Error>(())
    };

    // TCP -> QUIC
    let tcp_to_quic = async {
        let mut buf = vec![0u8; 8192];
        loop {
            let n = tcp_read.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            quic_send.write_all(&buf[..n]).await?;
        }
        Ok::<_, anyhow::Error>(())
    };

    tokio::select! {
        res = quic_to_tcp => {
            if let Err(e) = res {
                error!("QUIC->TCP错误: {}", e);
            }
        }
        res = tcp_to_quic => {
            if let Err(e) = res {
                error!("TCP->QUIC错误: {}", e);
            }
        }
    }

    // 关闭QUIC流
    quic_send.finish()?;

    Ok(())
}

// 自定义证书验证器（跳过验证）
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
