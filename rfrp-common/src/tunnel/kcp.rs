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
use std::task::Poll;
use tokio::sync::{mpsc, oneshot, watch, Mutex};
use tokio_kcp::{KcpConfig as TokioKcpConfig, KcpListener as TokioKcpListener, KcpStream};
use tokio_util::compat::TokioAsyncReadCompatExt;
use tracing::{debug, warn};
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

/// 出站流请求，通过 channel 发送给后台驱动任务
struct OutboundRequest {
    response_tx: oneshot::Sender<Result<YamuxStream>>,
}

/// KCP 连接包装器
///
/// 使用 yamux 进行多路复用的 KCP 连接。
/// 通过后台驱动任务持续调用 yamux 的 poll_next_inbound 来驱动连接 I/O，
/// 避免 open_bi 和 accept_bi 之间的死锁。
pub struct KcpConnection {
    /// 接收入站流的 channel
    inbound_rx: Mutex<mpsc::Receiver<YamuxStream>>,
    /// 向后台任务发送出站流请求的 channel
    outbound_tx: mpsc::Sender<OutboundRequest>,
    /// 连接关闭原因
    close_reason_rx: watch::Receiver<Option<String>>,
    /// 后台驱动任务句柄
    _driver_handle: tokio::task::JoinHandle<()>,
    /// 远端地址
    remote_addr: SocketAddr,
}

impl KcpConnection {
    /// 创建新的 KCP 连接
    pub fn new(stream: KcpStream, remote_addr: SocketAddr, is_client: bool) -> Self {
        let compat_stream = stream.compat();
        let mode = if is_client { Mode::Client } else { Mode::Server };
        let config = YamuxConfig::default();
        let connection = YamuxConnection::new(compat_stream, config, mode);

        let (inbound_tx, inbound_rx) = mpsc::channel::<YamuxStream>(32);
        let (outbound_tx, outbound_rx) = mpsc::channel::<OutboundRequest>(32);
        let (close_reason_tx, close_reason_rx) = watch::channel(None);

        let driver_handle = tokio::spawn(run_yamux_driver(
            connection,
            inbound_tx,
            outbound_rx,
            close_reason_tx,
        ));

        Self {
            inbound_rx: Mutex::new(inbound_rx),
            outbound_tx,
            close_reason_rx,
            _driver_handle: driver_handle,
            remote_addr,
        }
    }
}

impl Drop for KcpConnection {
    fn drop(&mut self) {
        self._driver_handle.abort();
    }
}

/// yamux 连接后台驱动任务
///
/// 独占 YamuxConnection，持续调用 poll_next_inbound 驱动连接 I/O，
/// 同时处理出站流请求。
async fn run_yamux_driver(
    mut connection: YamuxConnection<CompatKcpStream>,
    inbound_tx: mpsc::Sender<YamuxStream>,
    mut outbound_rx: mpsc::Receiver<OutboundRequest>,
    close_reason_tx: watch::Sender<Option<String>>,
) {
    let mut pending_outbound: Vec<OutboundRequest> = Vec::new();

    let reason = std::future::poll_fn(|cx| {
        // 1. 持续驱动 poll_next_inbound（这是驱动整个连接 I/O 的核心）
        loop {
            match connection.poll_next_inbound(cx) {
                Poll::Ready(Some(Ok(stream))) => {
                    if inbound_tx.try_send(stream).is_err() {
                        warn!("yamux driver: inbound channel full or closed");
                    }
                    continue;
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(format!("yamux error: {}", e));
                }
                Poll::Ready(None) => {
                    return Poll::Ready("connection closed by peer".to_string());
                }
                Poll::Pending => break,
            }
        }

        // 2. 接收新的出站流请求
        while let Poll::Ready(Some(req)) = outbound_rx.poll_recv(cx) {
            pending_outbound.push(req);
        }

        // 3. 处理待完成的出站流请求
        while !pending_outbound.is_empty() {
            match connection.poll_new_outbound(cx) {
                Poll::Ready(Ok(stream)) => {
                    let req = pending_outbound.swap_remove(0);
                    let _ = req.response_tx.send(Ok(stream));
                }
                Poll::Ready(Err(e)) => {
                    let req = pending_outbound.swap_remove(0);
                    let _ = req.response_tx.send(Err(anyhow!("outbound error: {}", e)));
                }
                Poll::Pending => {
                    break;
                }
            }
        }

        // 4. 检查是否所有前端句柄都已关闭
        if outbound_rx.is_closed() && pending_outbound.is_empty() && inbound_tx.is_closed() {
            return Poll::Ready("all handles dropped".to_string());
        }

        Poll::Pending
    })
    .await;

    debug!("yamux driver ended: {}", reason);
    let _ = close_reason_tx.send(Some(reason));
}

#[async_trait]
impl TunnelConnection for KcpConnection {
    async fn open_bi(&self) -> Result<(Box<dyn TunnelSendStream>, Box<dyn TunnelRecvStream>)> {
        let (response_tx, response_rx) = oneshot::channel();

        self.outbound_tx
            .send(OutboundRequest { response_tx })
            .await
            .map_err(|_| anyhow!("connection driver closed"))?;

        let stream = response_rx
            .await
            .map_err(|_| anyhow!("connection driver closed"))??;

        let shared_stream = Arc::new(Mutex::new(stream));
        Ok((
            Box::new(KcpSendStream::new(shared_stream.clone())),
            Box::new(KcpRecvStream::new(shared_stream)),
        ))
    }

    async fn accept_bi(&self) -> Result<(Box<dyn TunnelSendStream>, Box<dyn TunnelRecvStream>)> {
        let stream = {
            let mut rx = self.inbound_rx.lock().await;
            rx.recv().await.ok_or_else(|| anyhow!("connection closed"))?
        };

        let shared_stream = Arc::new(Mutex::new(stream));
        Ok((
            Box::new(KcpSendStream::new(shared_stream.clone())),
            Box::new(KcpRecvStream::new(shared_stream)),
        ))
    }

    async fn open_uni(&self) -> Result<Box<dyn TunnelSendStream>> {
        let (response_tx, response_rx) = oneshot::channel();

        self.outbound_tx
            .send(OutboundRequest { response_tx })
            .await
            .map_err(|_| anyhow!("connection driver closed"))?;

        let stream = response_rx
            .await
            .map_err(|_| anyhow!("connection driver closed"))??;

        Ok(Box::new(KcpSendStream::new(Arc::new(Mutex::new(stream)))))
    }

    async fn accept_uni(&self) -> Result<Box<dyn TunnelRecvStream>> {
        let stream = {
            let mut rx = self.inbound_rx.lock().await;
            rx.recv().await.ok_or_else(|| anyhow!("connection closed"))?
        };

        Ok(Box::new(KcpRecvStream::new(Arc::new(Mutex::new(stream)))))
    }

    fn remote_address(&self) -> SocketAddr {
        self.remote_addr
    }

    fn close_reason(&self) -> Option<String> {
        self.close_reason_rx.borrow().clone()
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
