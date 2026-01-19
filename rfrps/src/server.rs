use anyhow::Result;
use quinn::{Endpoint, ServerConfig, TransportConfig, VarInt};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, Set, ActiveModelTrait};
use tokio::task::JoinHandle;
use tracing::{info, warn, error, debug};

use crate::entity::{Proxy, Client, client, user_client, UserClient};
use crate::migration::get_connection;
use crate::traffic::TrafficManager;

pub struct ProxyServer {
    cert: CertificateDer<'static>,
    key: PrivateKeyDer<'static>,
}

// 代理监听器管理器
struct ProxyListenerManager {
    // client_id -> (proxy_id, JoinHandle)
    listeners: Arc<RwLock<HashMap<String, HashMap<i64, JoinHandle<()>>>>>,
    traffic_manager: Arc<TrafficManager>,
}

impl ProxyListenerManager {
    fn new(traffic_manager: Arc<TrafficManager>) -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
            traffic_manager,
        }
    }

    // 为客户端启动所有代理监听器
    async fn start_client_proxies(
        &self,
        client_id: String,
        connections: Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>>,
    ) -> Result<()> {
        let db = get_connection().await;

        // 查询该客户端的所有启用的代理
        let proxies = Proxy::find()
            .filter(crate::entity::proxy::Column::ClientId.eq(&client_id))
            .filter(crate::entity::proxy::Column::Enabled.eq(true))
            .all(db)
            .await?;

        if proxies.is_empty() {
            info!("  [客户端 {}] 没有启用的代理", client_id);
            return Ok(());
        }

        let mut listeners = self.listeners.write().await;
        let client_listeners = listeners.entry(client_id.clone()).or_insert_with(HashMap::new);

        for proxy in proxies {
            // 如果该代理的监听器已经运行，跳过
            if client_listeners.contains_key(&proxy.id) {
                continue;
            }

            let proxy_name = proxy.name.clone();
            let client_id_clone = client_id.clone();
            let listen_addr = format!("0.0.0.0:{}", proxy.remote_port);
            let target_addr = format!("{}:{}", proxy.local_ip, proxy.local_port);
            let proxy_id = proxy.id;
            let connections_clone = connections.clone();

            let traffic_mgr = self.traffic_manager.clone();
            let handle = tokio::spawn(async move {
                loop {
                    match run_proxy_listener(
                        proxy_name.clone(),
                        client_id_clone.clone(),
                        listen_addr.clone(),
                        target_addr.clone(),
                        connections_clone.clone(),
                        proxy_id,
                        traffic_mgr.clone(),
                    ).await {
                        Ok(_) => {},
                        Err(e) => {
                            error!("[{}] 代理监听失败: {}", proxy_name, e);
                            // 监听器失败，等待重试
                        }
                    }
                    // 如果监听器失败，等待一段时间后重新尝试启动（如果客户端仍在线）
                    tokio::time::sleep(Duration::from_secs(5)).await;

                    // 检查客户端是否仍在连接
                    let conns = connections_clone.read().await;
                    if !conns.contains_key(&client_id_clone) {
                        warn!("[{}] 客户端已离线，停止代理监听", proxy_name);
                        break;
                    }
                }
            });

            client_listeners.insert(proxy_id, handle);
            info!("  [客户端 {}] 启动代理: {} 端口: {}", client_id, proxy.name, proxy.remote_port);
        }

        Ok(())
    }

    // 停止客户端的所有代理监听器
    async fn stop_client_proxies(&self, client_id: &str) {
        let mut listeners = self.listeners.write().await;
        if let Some(client_listeners) = listeners.remove(client_id) {
            info!("  [客户端 {}] 停止 {} 个代理监听器", client_id, client_listeners.len());
            for (proxy_id, handle) in client_listeners {
                handle.abort();
                debug!("    代理 #{} 已停止", proxy_id);
            }
        }
    }
}

impl ProxyServer {
    pub fn new() -> Result<Self> {
        let cert = rcgen::generate_simple_self_signed(&["rfrp".to_string()])?;

        Ok(Self {
            cert: CertificateDer::from(cert.cert.der().to_vec()),
            key: PrivateKeyDer::from(PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der())),
        })
    }

    pub async fn run(&self, bind_addr: String) -> Result<()> {
        let mut transport_config = TransportConfig::default();
        transport_config.max_concurrent_uni_streams(VarInt::from_u32(100));
        transport_config.keep_alive_interval(Some(Duration::from_secs(5)));
        transport_config.max_idle_timeout(Some(Duration::from_secs(600).try_into()?));

        let mut server_config = ServerConfig::with_single_cert(
            vec![self.cert.clone()],
            self.key.clone_key(),
        )?;
        server_config.transport_config(Arc::new(transport_config));

        let endpoint = Endpoint::server(server_config, bind_addr.parse()?)?;

        info!("🚀 QUIC服务器启动成功!");
        info!("📡 监听地址: {}", bind_addr);
        info!("⏱️  空闲超时: 600秒, 心跳间隔: 5秒");

        let client_connections: Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>> =
            Arc::new(RwLock::new(HashMap::new()));

        // 初始化流量管理器
        let traffic_manager = Arc::new(TrafficManager::new());
        // 启动定期刷新任务
        traffic_manager.clone().start_periodic_flush();

        let listener_manager = Arc::new(ProxyListenerManager::new(traffic_manager.clone()));

        info!("⏳ 等待客户端连接...");

        // 接受客户端连接
        while let Some(connecting) = endpoint.accept().await {
            match connecting.await {
                Ok(conn) => {
                    let remote_addr = conn.remote_address();
                    info!("📡 新连接来自: {}", remote_addr);

                    // 等待客户端发送 token 认证
                    let conn_clone = Arc::new(conn);
                    let connections = client_connections.clone();
                    let listener_mgr = listener_manager.clone();

                    tokio::spawn(async move {
                        debug!("开始处理连接！");
                        if let Err(e) = handle_client_auth(conn_clone, connections, listener_mgr).await {
                            error!("❌ 客户端认证失败: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("❌ 连接接受失败: {}", e);
                }
            }
        }

        Ok(())
    }
}

async fn handle_client_auth(
    conn: Arc<quinn::Connection>,
    connections: Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>>,
    listener_manager: Arc<ProxyListenerManager>,
) -> Result<()> {
    // 等待客户端发送 token (格式: 2字节长度 + 内容)
    let mut recv_stream = match conn.accept_uni().await {
        Ok(s) => s,
        Err(_) => return Ok(()),
    };

    let mut len_buf = [0u8; 2];
    recv_stream.read_exact(&mut len_buf).await?;
    let len = u16::from_be_bytes(len_buf) as usize;
    debug!("接收token长度: {}", len);

    let mut token_buf = vec![0u8; len];
    recv_stream.read_exact(&mut token_buf).await?;
    let token = String::from_utf8(token_buf)?;
    debug!("接收token: {}", token);

    let db = get_connection().await;
    // 查找对应的客户端
    let client = match Client::find()
        .filter(client::Column::Token.eq(&token))
        .one(db)
        .await?
    {
        Some(c) => c,
        None => {
            error!("❌ 无效的 token");
            return Ok(());
        }
    };

    let client_id = client.id;
    let client_name = client.name.clone();

    // 更新客户端为在线状态
    let mut client_active: client::ActiveModel = client.into();
    client_active.is_online = Set(true);
    debug!("更新客户端状态: {:?}", client_active);
    let _ = client_active.update(db).await;

    info!("✅ 客户端认证成功: {} (ID: {}, 在线: {})", client_name, client_id, conn.remote_address());

    // 启动该客户端的所有代理监听器
    if let Err(e) = listener_manager.start_client_proxies(format!("{}", client_id), connections.clone()).await {
        error!("❌ 启动代理监听器失败: {}", e);
    }

    // 保存连接
    let mut conns = connections.write().await;
    conns.insert(format!("{}", client_id), conn.clone());
    drop(conns);

    // 循环接受代理流请求
    loop {
        match conn.accept_bi().await {
            Ok((send, recv)) => {
                let conn_clone = conn.clone();
                let connections_clone = connections.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_proxy_stream(send, recv, conn_clone, connections_clone).await {
                        error!("❌ 处理代理流错误: {}", e);
                    }
                });
            }
            Err(_) => {
                warn!("⚠️  客户端断开连接: {}", client_name);
                let mut conns = connections.write().await;
                conns.remove(&format!("{}", client_id));
                drop(conns);

                // 停止该客户端的所有代理监听器
                listener_manager.stop_client_proxies(&format!("{}", client_id)).await;

                // 更新客户端为离线状态
                let db = get_connection().await;
                if let Some(client) = Client::find_by_id(client_id).one(db).await.unwrap() {
                    let mut client_active: client::ActiveModel = client.into();
                    client_active.is_online = Set(false);
                    let _ = client_active.update(db).await;
                }
                break;
            }
        }
    }

    Ok(())
}

async fn run_proxy_listener(
    proxy_name: String,
    client_id: String,
    listen_addr: String,
    target_addr: String,
    connections: Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>>,
    proxy_id: i64,
    traffic_manager: Arc<TrafficManager>,
) -> Result<()> {
    let listener = TcpListener::bind(&listen_addr).await?;
    info!("[{}] 🔌 监听端口: {} -> {}", proxy_name, listen_addr, target_addr);

    loop {
        match listener.accept().await {
            Ok((tcp_stream, addr)) => {
                info!("[{}] 📥 新连接来自: {}", proxy_name, addr);

                let connections_clone = connections.clone();
                let client_id = client_id.clone();
                let target_addr = target_addr.clone();
                let proxy_name = proxy_name.clone();

                let traffic_mgr = traffic_manager.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_tcp_to_quic(tcp_stream, addr, target_addr, proxy_name, client_id, connections_clone, proxy_id, traffic_mgr).await {
                        error!("❌ 处理连接错误: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("[{}] ❌ 接受连接失败: {}", proxy_name, e);
            }
        }
    }
}

async fn handle_tcp_to_quic(
    mut tcp_stream: TcpStream,
    addr: std::net::SocketAddr,
    target_addr: String,
    proxy_name: String,
    client_id: String,
    connections: Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>>,
    proxy_id: i64,
    traffic_manager: Arc<TrafficManager>,
) -> Result<()> {
    // 获取客户端连接
    let conn = {
        let conns = connections.read().await;
        conns.get(&client_id).cloned()
    };

    let conn = match conn {
        Some(c) => c,
        None => {
            error!("[{}] ❌ 客户端未连接", proxy_name);
            return Ok(());
        }
    };

    // 打开双向QUIC流
    let (mut quic_send, mut quic_recv) = conn.open_bi().await?;

    info!("[{}] 🔗 QUIC流已打开: {}", proxy_name, addr);

    // 发送目标地址
    let target_bytes = target_addr.as_bytes();
    let len = target_bytes.len() as u16;

    quic_send.write_all(&len.to_be_bytes()).await?;
    quic_send.write_all(target_bytes).await?;
    quic_send.flush().await?;

    let (mut tcp_read, mut tcp_write) = tcp_stream.split();

    // 使用Arc<RwLock>>来在两个方向上统计流量
    let sent_stats = Arc::new(RwLock::new(0i64));
    let received_stats = Arc::new(RwLock::new(0i64));

    let sent_stats_clone = sent_stats.clone();
    let received_stats_clone = received_stats.clone();

    // TCP -> QUIC
    let tcp_to_quic = async {
        let mut buf = vec![0u8; 8192];
        loop {
            let n = tcp_read.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            quic_send.write_all(&buf[..n]).await?;
            // 统计发送字节数
            let mut stats = sent_stats_clone.write().await;
            *stats += n as i64;
        }
        Ok::<_, anyhow::Error>(())
    };

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
                    // 统计接收字节数
                    let mut stats = received_stats_clone.write().await;
                    *stats += n as i64;
                }
                None => break,
            }
        }
        Ok::<_, anyhow::Error>(())
    };

    tokio::select! {
        res = tcp_to_quic => {
            if let Err(e) = res {
                error!("[{}] TCP->QUIC错误: {}", proxy_name, e);
            }
        }
        res = quic_to_tcp => {
            if let Err(e) = res {
                error!("[{}] QUIC->TCP错误: {}", proxy_name, e);
            }
        }
    }

    quic_send.finish()?;
    info!("[{}] 🔚 连接已关闭: {}", proxy_name, addr);

    // 获取最终统计数据
    let bytes_sent = {
        let stats = sent_stats.read().await;
        *stats
    };
    let bytes_received = {
        let stats = received_stats.read().await;
        *stats
    };

    // 记录流量统计到 TrafficManager
    // bytes_sent: TCP -> QUIC (从用户到服务器) - 用户上传
    // bytes_received: QUIC -> TCP (从服务器到用户) - 用户下载
    if bytes_sent > 0 || bytes_received > 0 {
        let client_id_num = client_id.parse::<i64>().unwrap_or(0);

        // 查询绑定到该客户端的所有用户
        let db = get_connection().await;
        let user_clients = UserClient::find()
            .filter(user_client::Column::ClientId.eq(client_id_num))
            .all(db)
            .await
            .unwrap_or_default();

        let user_count = user_clients.len();

        // 为每个用户记录流量
        for uc in user_clients {
            traffic_manager.record_traffic(
                proxy_id,
                client_id_num,
                Some(uc.user_id),
                bytes_sent,
                bytes_received,
            ).await;
        }

        debug!("[{}] 流量统计: 发送={}, 接收={}, 关联用户数={}",
               proxy_name, bytes_sent, bytes_received, user_count);
    }

    Ok(())
}

async fn handle_proxy_stream(
    mut quic_send: quinn::SendStream,
    mut quic_recv: quinn::RecvStream,
    _conn: Arc<quinn::Connection>,
    _connections: Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>>,
) -> Result<()> {
    // 读取目标地址（客户端已连接）
    let mut len_buf = [0u8; 2];
    quic_recv.read_exact(&mut len_buf).await?;
    let len = u16::from_be_bytes(len_buf) as usize;

    let mut addr_buf = vec![0u8; len];
    quic_recv.read_exact(&mut addr_buf).await?;
    let target_addr = String::from_utf8(addr_buf)?;

    // 连接到目标服务
    let mut tcp_stream = TcpStream::connect(&target_addr).await?;

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

    quic_send.finish()?;

    Ok(())
}
