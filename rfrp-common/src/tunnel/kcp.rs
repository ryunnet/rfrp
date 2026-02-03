//! KCP 协议隧道实现
//!
//! 此模块提供了基于 KCP 协议的隧道实现，包括：
//! - `KcpSendStream` / `KcpRecvStream`: 流包装器
//! - `KcpMultiplexer`: 多路复用器，在单个 KCP 连接上支持多个虚拟流
//! - `KcpConnection`: 连接包装器
//! - `KcpConnector`: 客户端连接器
//! - `KcpListener`: 服务端监听器

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_kcp::{KcpConfig as TokioKcpConfig, KcpListener as TokioKcpListener, KcpStream};

use super::traits::{TunnelConnection, TunnelConnector, TunnelListener, TunnelRecvStream, TunnelSendStream};
use crate::config::KcpConfig;

/// 帧标志：创建新流
const FLAG_SYN: u8 = 0x01;
/// 帧标志：关闭流
const FLAG_FIN: u8 = 0x02;
/// 帧标志：数据帧
const FLAG_DATA: u8 = 0x00;

/// 帧头大小：流ID(4) + 标志(1) + 长度(2) = 7 字节
const FRAME_HEADER_SIZE: usize = 7;

/// KCP 发送流
///
/// 在 KCP 多路复用连接上的虚拟发送流。
pub struct KcpSendStream {
    stream_id: u32,
    multiplexer: Arc<KcpMultiplexer>,
    finished: bool,
}

impl KcpSendStream {
    fn new(stream_id: u32, multiplexer: Arc<KcpMultiplexer>) -> Self {
        Self {
            stream_id,
            multiplexer,
            finished: false,
        }
    }
}

#[async_trait]
impl TunnelSendStream for KcpSendStream {
    async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.multiplexer.send_frame(self.stream_id, FLAG_DATA, buf).await
    }

    async fn flush(&mut self) -> Result<()> {
        // KCP 内部处理刷新
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        if !self.finished {
            self.finished = true;
            let multiplexer = self.multiplexer.clone();
            let stream_id = self.stream_id;
            tokio::spawn(async move {
                let _ = multiplexer.send_frame(stream_id, FLAG_FIN, &[]).await;
            });
        }
        Ok(())
    }
}

impl Drop for KcpSendStream {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

/// KCP 接收流
///
/// 在 KCP 多路复用连接上的虚拟接收流。
pub struct KcpRecvStream {
    #[allow(dead_code)]
    stream_id: u32,
    receiver: mpsc::Receiver<Vec<u8>>,
    buffer: Vec<u8>,
    position: usize,
    closed: bool,
}

impl KcpRecvStream {
    fn new(stream_id: u32, receiver: mpsc::Receiver<Vec<u8>>) -> Self {
        Self {
            stream_id,
            receiver,
            buffer: Vec::new(),
            position: 0,
            closed: false,
        }
    }

    async fn fill_buffer(&mut self) -> Result<bool> {
        if self.closed {
            return Ok(false);
        }

        match self.receiver.recv().await {
            Some(data) => {
                if data.is_empty() {
                    // 空数据表示流关闭
                    self.closed = true;
                    Ok(false)
                } else {
                    self.buffer = data;
                    self.position = 0;
                    Ok(true)
                }
            }
            None => {
                self.closed = true;
                Ok(false)
            }
        }
    }
}

#[async_trait]
impl TunnelRecvStream for KcpRecvStream {
    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let mut filled = 0;
        while filled < buf.len() {
            // 检查缓冲区中是否有数据
            if self.position < self.buffer.len() {
                let available = self.buffer.len() - self.position;
                let to_copy = std::cmp::min(available, buf.len() - filled);
                buf[filled..filled + to_copy]
                    .copy_from_slice(&self.buffer[self.position..self.position + to_copy]);
                self.position += to_copy;
                filled += to_copy;
            } else {
                // 需要更多数据
                if !self.fill_buffer().await? {
                    return Err(anyhow!("Stream closed before read completed"));
                }
            }
        }
        Ok(())
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<Option<usize>> {
        // 检查缓冲区中是否有数据
        if self.position < self.buffer.len() {
            let available = self.buffer.len() - self.position;
            let to_copy = std::cmp::min(available, buf.len());
            buf[..to_copy].copy_from_slice(&self.buffer[self.position..self.position + to_copy]);
            self.position += to_copy;
            return Ok(Some(to_copy));
        }

        // 需要获取更多数据
        if !self.fill_buffer().await? {
            return Ok(None);
        }

        // 从缓冲区读取
        let available = self.buffer.len() - self.position;
        let to_copy = std::cmp::min(available, buf.len());
        buf[..to_copy].copy_from_slice(&self.buffer[self.position..self.position + to_copy]);
        self.position += to_copy;
        Ok(Some(to_copy))
    }
}

/// 流状态
struct StreamState {
    sender: mpsc::Sender<Vec<u8>>,
}

/// KCP 多路复用器
///
/// 在单个 KCP 连接上支持多个虚拟流的多路复用器。
/// 使用简单的帧协议：流ID(4) + 标志(1) + 长度(2) + 数据。
pub struct KcpMultiplexer {
    stream: Arc<Mutex<KcpStream>>,
    streams: Arc<RwLock<HashMap<u32, StreamState>>>,
    next_stream_id: AtomicU32,
    #[allow(dead_code)]
    is_client: bool,
    incoming_streams: mpsc::Sender<(u32, mpsc::Receiver<Vec<u8>>)>,
    accept_receiver: Mutex<mpsc::Receiver<(u32, mpsc::Receiver<Vec<u8>>)>>,
    remote_addr: SocketAddr,
    closed: Arc<RwLock<Option<String>>>,
}

impl KcpMultiplexer {
    /// 创建新的多路复用器
    ///
    /// # Arguments
    /// * `stream` - 底层 KCP 流
    /// * `remote_addr` - 远程地址
    /// * `is_client` - 是否为客户端（客户端使用奇数ID，服务端使用偶数ID）
    pub fn new(stream: KcpStream, remote_addr: SocketAddr, is_client: bool) -> Arc<Self> {
        // 客户端使用奇数ID (1, 3, 5...)，服务端使用偶数ID (2, 4, 6...)
        let initial_id = if is_client { 1 } else { 2 };
        let (incoming_tx, incoming_rx) = mpsc::channel(100);

        let mux = Arc::new(Self {
            stream: Arc::new(Mutex::new(stream)),
            streams: Arc::new(RwLock::new(HashMap::new())),
            next_stream_id: AtomicU32::new(initial_id),
            is_client,
            incoming_streams: incoming_tx,
            accept_receiver: Mutex::new(incoming_rx),
            remote_addr,
            closed: Arc::new(RwLock::new(None)),
        });

        // 启动接收循环
        let mux_clone = mux.clone();
        tokio::spawn(async move {
            if let Err(e) = mux_clone.receive_loop().await {
                let mut closed = mux_clone.closed.write().await;
                *closed = Some(e.to_string());
            }
        });

        mux
    }

    /// 分配新的流ID
    fn allocate_stream_id(&self) -> u32 {
        self.next_stream_id.fetch_add(2, Ordering::SeqCst)
    }

    /// 发送帧到 KCP 连接
    pub async fn send_frame(&self, stream_id: u32, flags: u8, data: &[u8]) -> Result<()> {
        let mut frame = Vec::with_capacity(FRAME_HEADER_SIZE + data.len());
        frame.extend_from_slice(&stream_id.to_be_bytes());
        frame.push(flags);
        frame.extend_from_slice(&(data.len() as u16).to_be_bytes());
        frame.extend_from_slice(data);

        let mut stream = self.stream.lock().await;
        stream.write_all(&frame).await?;
        stream.flush().await?;
        Ok(())
    }

    /// 打开新的双向流
    pub async fn open_bi(self: &Arc<Self>) -> Result<(KcpSendStream, KcpRecvStream)> {
        let stream_id = self.allocate_stream_id();

        // 创建此流的通道
        let (tx, rx) = mpsc::channel(100);
        let state = StreamState { sender: tx };

        {
            let mut streams = self.streams.write().await;
            streams.insert(stream_id, state);
        }

        // 发送 SYN 帧
        self.send_frame(stream_id, FLAG_SYN, &[]).await?;

        let send = KcpSendStream::new(stream_id, self.clone());
        let recv = KcpRecvStream::new(stream_id, rx);

        Ok((send, recv))
    }

    /// 接受传入的双向流
    pub async fn accept_bi(self: &Arc<Self>) -> Result<(KcpSendStream, KcpRecvStream)> {
        let mut accept_rx = self.accept_receiver.lock().await;
        match accept_rx.recv().await {
            Some((stream_id, rx)) => {
                let send = KcpSendStream::new(stream_id, self.clone());
                let recv = KcpRecvStream::new(stream_id, rx);
                Ok((send, recv))
            }
            None => Err(anyhow!("Connection closed")),
        }
    }

    /// 打开单向流
    pub async fn open_uni(self: &Arc<Self>) -> Result<KcpSendStream> {
        let stream_id = self.allocate_stream_id();

        // 对于单向流，不需要接收器
        let (tx, _rx) = mpsc::channel(1);
        let state = StreamState { sender: tx };

        {
            let mut streams = self.streams.write().await;
            streams.insert(stream_id, state);
        }

        // 发送 SYN 帧
        self.send_frame(stream_id, FLAG_SYN, &[]).await?;

        Ok(KcpSendStream::new(stream_id, self.clone()))
    }

    /// 主接收循环 - 读取帧并分发到各个流
    async fn receive_loop(self: &Arc<Self>) -> Result<()> {
        let mut header_buf = [0u8; FRAME_HEADER_SIZE];

        loop {
            // 读取帧头
            {
                let mut stream = self.stream.lock().await;
                if let Err(e) = stream.read_exact(&mut header_buf).await {
                    tracing::error!("KCP receive_loop: Failed to read frame header: {}", e);
                    return Err(anyhow!("Failed to read frame header: {}", e));
                }
            }

            let stream_id = u32::from_be_bytes([header_buf[0], header_buf[1], header_buf[2], header_buf[3]]);
            let flags = header_buf[4];
            let length = u16::from_be_bytes([header_buf[5], header_buf[6]]) as usize;

            tracing::debug!("KCP receive_loop: stream_id={}, flags={}, length={}", stream_id, flags, length);

            // 读取帧数据
            let mut data = vec![0u8; length];
            if length > 0 {
                let mut stream = self.stream.lock().await;
                stream.read_exact(&mut data).await?;
            }

            match flags {
                FLAG_SYN => {
                    tracing::debug!("KCP receive_loop: Received SYN for stream {}", stream_id);
                    // 新的传入流
                    let (tx, rx) = mpsc::channel(100);
                    let state = StreamState { sender: tx };

                    {
                        let mut streams = self.streams.write().await;
                        streams.insert(stream_id, state);
                    }

                    // 通知接受者
                    if self.incoming_streams.send((stream_id, rx)).await.is_err() {
                        tracing::warn!("KCP receive_loop: Failed to notify incoming stream {}", stream_id);
                    }
                }
                FLAG_FIN => {
                    tracing::debug!("KCP receive_loop: Received FIN for stream {}", stream_id);
                    // 流关闭
                    let sender = {
                        let mut streams = self.streams.write().await;
                        streams.remove(&stream_id).map(|s| s.sender)
                    };

                    if let Some(tx) = sender {
                        // 发送空数据表示关闭
                        let _ = tx.send(Vec::new()).await;
                    }
                }
                FLAG_DATA => {
                    tracing::debug!("KCP receive_loop: Received DATA for stream {}, {} bytes", stream_id, data.len());
                    // 数据帧
                    let sender = {
                        let streams = self.streams.read().await;
                        streams.get(&stream_id).map(|s| s.sender.clone())
                    };

                    if let Some(tx) = sender {
                        if tx.send(data).await.is_err() {
                            // 接收器已丢弃，移除流
                            let mut streams = self.streams.write().await;
                            streams.remove(&stream_id);
                        }
                    } else {
                        tracing::warn!("KCP receive_loop: No sender found for stream {}", stream_id);
                    }
                }
                _ => {
                    tracing::warn!("KCP receive_loop: Unknown flag {} for stream {}", flags, stream_id);
                }
            }
        }
    }
}

/// KCP 连接包装器
pub struct KcpConnection {
    multiplexer: Arc<KcpMultiplexer>,
}

impl KcpConnection {
    /// 创建新的 KCP 连接
    pub fn new(stream: KcpStream, remote_addr: SocketAddr, is_client: bool) -> Self {
        Self {
            multiplexer: KcpMultiplexer::new(stream, remote_addr, is_client),
        }
    }

    /// 异步获取关闭原因
    pub async fn get_close_reason(&self) -> Option<String> {
        let closed = self.multiplexer.closed.read().await;
        closed.clone()
    }
}

#[async_trait]
impl TunnelConnection for KcpConnection {
    async fn open_bi(&self) -> Result<(Box<dyn TunnelSendStream>, Box<dyn TunnelRecvStream>)> {
        let (send, recv) = self.multiplexer.open_bi().await?;
        Ok((Box::new(send), Box::new(recv)))
    }

    async fn accept_bi(&self) -> Result<(Box<dyn TunnelSendStream>, Box<dyn TunnelRecvStream>)> {
        let (send, recv) = self.multiplexer.accept_bi().await?;
        Ok((Box::new(send), Box::new(recv)))
    }

    async fn open_uni(&self) -> Result<Box<dyn TunnelSendStream>> {
        let send = self.multiplexer.open_uni().await?;
        Ok(Box::new(send))
    }

    async fn accept_uni(&self) -> Result<Box<dyn TunnelRecvStream>> {
        // 接受双向流但只返回接收部分
        let (_send, recv) = self.multiplexer.accept_bi().await?;
        Ok(Box::new(recv))
    }

    fn remote_address(&self) -> SocketAddr {
        self.multiplexer.remote_addr
    }

    fn close_reason(&self) -> Option<String> {
        // 使用 try_read 避免阻塞
        match self.multiplexer.closed.try_read() {
            Ok(guard) => guard.clone(),
            Err(_) => None,
        }
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
        let stream = KcpStream::connect(&kcp_config, addr).await?;
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
        let listener = TokioKcpListener::bind(kcp_config, bind_addr).await?;
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
