//! KCP 协议隧道实现
//!
//! 此模块提供了基于 KCP 协议的隧道实现，使用 yamux 库进行多路复用：
//! - `KcpSendStream` / `KcpRecvStream`: 流包装器
//! - `KcpConnection`: 连接包装器（基于 yamux 多路复用）
//! - `KcpConnector`: 客户端连接器
//! - `KcpListener`: 服务端监听器

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::{AsyncReadExt, AsyncWriteExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_kcp::{KcpConfig as TokioKcpConfig, KcpListener as TokioKcpListener, KcpStream};
use tokio_util::compat::TokioAsyncReadCompatExt;
use yamux::{Config as YamuxConfig, Connection as YamuxConnection, Mode, Stream as YamuxStream};

use super::traits::{TunnelConnection, TunnelConnector, TunnelListener, TunnelRecvStream, TunnelSendStream};
use crate::config::KcpConfig;
use crate::utils::create_configured_udp_socket;

/// KCP 发送流
///
/// 基于 yamux Stream 的发送流包装器。
pub struct KcpSendStream {
    stream: Arc<Mutex<YamuxStream>>,
}

impl KcpSendStream {
    fn new(stream: Arc<Mutex<YamuxStream>>) -> Self {
        Self { stream }
    }
}

#[async_trait]
impl TunnelSendStream for KcpSendStream {
    async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        let mut stream = self.stream.lock().await;
        stream.write_all(buf).await?;
        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        let mut stream = self.stream.lock().await;
        stream.flush().await?;
        Ok(())
    }

    async fn finish(&mut self) -> Result<()> {
        let mut stream = self.stream.lock().await;
        stream.close().await?;
        Ok(())
    }
}

/// KCP 接收流
///
/// 基于 yamux Stream 的接收流包装器。
pub struct KcpRecvStream {
    stream: Arc<Mutex<YamuxStream>>,
}

impl KcpRecvStream {
    fn new(stream: Arc<Mutex<YamuxStream>>) -> Self {
        Self { stream }
    }
}

#[async_trait]
impl TunnelRecvStream for KcpRecvStream {
    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let mut stream = self.stream.lock().await;
        stream.read_exact(buf).await?;
        Ok(())
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<Option<usize>> {
        let mut stream = self.stream.lock().await;
        match stream.read(buf).await {
            Ok(0) => Ok(None),
            Ok(n) => Ok(Some(n)),
            Err(e) => Err(anyhow!("Read error: {}", e)),
        }
    }
}

/// Compat 包装器，将 tokio KcpStream 转换为 futures AsyncRead/AsyncWrite
type CompatKcpStream = tokio_util::compat::Compat<KcpStream>;

/// KCP 连接包装器
///
/// 使用 yamux 进行多路复用的 KCP 连接。
pub struct KcpConnection {
    connection: Arc<Mutex<YamuxConnection<CompatKcpStream>>>,
    remote_addr: SocketAddr,
}

impl KcpConnection {
    /// 创建新的 KCP 连接
    pub fn new(stream: KcpStream, remote_addr: SocketAddr, is_client: bool) -> Self {
        let compat_stream = stream.compat();
        let mode = if is_client { Mode::Client } else { Mode::Server };
        let config = YamuxConfig::default();
        let connection = YamuxConnection::new(compat_stream, config, mode);

        Self {
            connection: Arc::new(Mutex::new(connection)),
            remote_addr,
        }
    }
}

#[async_trait]
impl TunnelConnection for KcpConnection {
    async fn open_bi(&self) -> Result<(Box<dyn TunnelSendStream>, Box<dyn TunnelRecvStream>)> {
        let stream = {
            let mut conn = self.connection.lock().await;
            std::future::poll_fn(|cx| conn.poll_new_outbound(cx)).await?
        };

        // 使用 Arc<Mutex> 共享 stream 用于读写
        let shared_stream = Arc::new(Mutex::new(stream));

        Ok((
            Box::new(KcpSendStream::new(shared_stream.clone())),
            Box::new(KcpRecvStream::new(shared_stream)),
        ))
    }

    async fn accept_bi(&self) -> Result<(Box<dyn TunnelSendStream>, Box<dyn TunnelRecvStream>)> {
        let stream = {
            let mut conn = self.connection.lock().await;
            match std::future::poll_fn(|cx| conn.poll_next_inbound(cx)).await {
                Some(Ok(stream)) => stream,
                Some(Err(e)) => return Err(anyhow!("Accept error: {}", e)),
                None => return Err(anyhow!("Connection closed")),
            }
        };

        // 使用 Arc<Mutex> 共享 stream 用于读写
        let shared_stream = Arc::new(Mutex::new(stream));

        Ok((
            Box::new(KcpSendStream::new(shared_stream.clone())),
            Box::new(KcpRecvStream::new(shared_stream)),
        ))
    }

    async fn open_uni(&self) -> Result<Box<dyn TunnelSendStream>> {
        let stream = {
            let mut conn = self.connection.lock().await;
            std::future::poll_fn(|cx| conn.poll_new_outbound(cx)).await?
        };

        Ok(Box::new(KcpSendStream::new(Arc::new(Mutex::new(stream)))))
    }

    async fn accept_uni(&self) -> Result<Box<dyn TunnelRecvStream>> {
        let stream = {
            let mut conn = self.connection.lock().await;
            match std::future::poll_fn(|cx| conn.poll_next_inbound(cx)).await {
                Some(Ok(stream)) => stream,
                Some(Err(e)) => return Err(anyhow!("Accept error: {}", e)),
                None => return Err(anyhow!("Connection closed")),
            }
        };

        Ok(Box::new(KcpRecvStream::new(Arc::new(Mutex::new(stream)))))
    }

    fn remote_address(&self) -> SocketAddr {
        self.remote_addr
    }

    fn close_reason(&self) -> Option<String> {
        None
    }
}

/// KCP 客户端连接器
pub struct KcpConnector {
    config: KcpConfig,
}

impl KcpConnector {
    /// 创建新的 KCP 连接器
    pub fn new(config: Option<KcpConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
        }
    }

    fn build_kcp_config(&self) -> TokioKcpConfig {
        let mut config = TokioKcpConfig::default();
        config.nodelay = tokio_kcp::KcpNoDelayConfig {
            nodelay: self.config.nodelay,
            interval: self.config.interval as i32,
            resend: self.config.resend as i32,
            nc: self.config.nc,
        };
        config
    }
}

#[async_trait]
impl TunnelConnector for KcpConnector {
    async fn connect(&self, addr: SocketAddr) -> Result<Box<dyn TunnelConnection>> {
        let kcp_config = self.build_kcp_config();

        let local_addr: SocketAddr = if addr.is_ipv4() {
            "0.0.0.0:0".parse().unwrap()
        } else {
            "[::]:0".parse().unwrap()
        };
        let socket = create_configured_udp_socket(local_addr).await?;

        let stream = KcpStream::connect_with_socket(&kcp_config, socket, addr).await?;
        Ok(Box::new(KcpConnection::new(stream, addr, true)))
    }
}

/// KCP 服务端监听器
pub struct KcpListener {
    listener: Mutex<TokioKcpListener>,
}

impl KcpListener {
    /// 创建新的 KCP 监听器
    pub async fn new(bind_addr: SocketAddr, config: Option<KcpConfig>) -> Result<Self> {
        let kcp_config = build_kcp_config(config);
        let socket = create_configured_udp_socket(bind_addr).await?;
        let listener = TokioKcpListener::from_socket(kcp_config, socket).await?;
        Ok(Self { listener: Mutex::new(listener) })
    }
}

fn build_kcp_config(config: Option<KcpConfig>) -> TokioKcpConfig {
    let config = config.unwrap_or_default();
    let mut kcp_config = TokioKcpConfig::default();
    kcp_config.nodelay = tokio_kcp::KcpNoDelayConfig {
        nodelay: config.nodelay,
        interval: config.interval as i32,
        resend: config.resend as i32,
        nc: config.nc,
    };
    kcp_config
}

#[async_trait]
impl TunnelListener for KcpListener {
    async fn accept(&self) -> Result<Box<dyn TunnelConnection>> {
        let mut listener = self.listener.lock().await;
        let (stream, addr) = listener.accept().await?;
        Ok(Box::new(KcpConnection::new(stream, addr, false)))
    }
}
