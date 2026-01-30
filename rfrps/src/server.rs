use anyhow::Result;
use quinn::{Endpoint, ServerConfig, TransportConfig, VarInt};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, Set, ActiveModelTrait};
use tokio::task::JoinHandle;
use tracing::{info, warn, error, debug};
use serde::{Serialize, Deserialize};

use crate::entity::{Proxy, Client, User, client, user_client, UserClient};
use crate::migration::get_connection;
use crate::traffic::TrafficManager;
use crate::config_manager::ConfigManager;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyProtocol {
    Tcp,
    Udp,
}

impl From<String> for ProxyProtocol {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "udp" => ProxyProtocol::Udp,
            _ => ProxyProtocol::Tcp,
        }
    }
}

impl From<&str> for ProxyProtocol {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "udp" => ProxyProtocol::Udp,
            _ => ProxyProtocol::Tcp,
        }
    }
}

impl ProxyProtocol {
    pub fn as_str(&self) -> &str {
        match self {
            ProxyProtocol::Tcp => "tcp",
            ProxyProtocol::Udp => "udp",
        }
    }
}

// UDP会话信息
#[allow(dead_code)]
struct UdpSession {
    target_addr: SocketAddr,
    last_activity: tokio::time::Instant,
}

pub struct ProxyServer {
    cert: CertificateDer<'static>,
    key: PrivateKeyDer<'static>,
    traffic_manager: Arc<TrafficManager>,
    listener_manager: Arc<ProxyListenerManager>,
    client_connections: Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>>,
    config_manager: Arc<ConfigManager>,
}

// 代理监听器管理器
pub struct ProxyListenerManager {
    // client_id -> (proxy_id, JoinHandle)
    listeners: Arc<RwLock<HashMap<String, HashMap<i64, JoinHandle<()>>>>>,
    // UDP会话管理: (client_id, proxy_id) -> (source_addr -> UdpSession)
    udp_sessions: Arc<RwLock<HashMap<(String, i64), HashMap<SocketAddr, UdpSession>>>>,
    traffic_manager: Arc<TrafficManager>,
}

impl ProxyListenerManager {
    pub fn new(traffic_manager: Arc<TrafficManager>) -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
            udp_sessions: Arc::new(RwLock::new(HashMap::new())),
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
            let proxy_protocol: ProxyProtocol = proxy.proxy_type.clone().into();
            let proxy_protocol_str = proxy_protocol.as_str().to_uppercase();
            let client_id_clone = client_id.clone();
            let listen_addr = format!("0.0.0.0:{}", proxy.remote_port);
            let target_addr = format!("{}:{}", proxy.local_ip, proxy.local_port);
            let proxy_id = proxy.id;
            let connections_clone = connections.clone();
            let traffic_manager = self.traffic_manager.clone();

            let udp_sessions = self.udp_sessions.clone();

            let handle = tokio::spawn(async move {
                loop {
                    let result = match proxy_protocol {
                        ProxyProtocol::Tcp => {
                            run_tcp_proxy_listener(
                                proxy_name.clone(),
                                client_id_clone.clone(),
                                listen_addr.clone(),
                                target_addr.clone(),
                                connections_clone.clone(),
                                proxy_id,
                                traffic_manager.clone(),
                            ).await
                        }
                        ProxyProtocol::Udp => {
                            run_udp_proxy_listener(
                                proxy_name.clone(),
                                client_id_clone.clone(),
                                listen_addr.clone(),
                                target_addr.clone(),
                                connections_clone.clone(),
                                proxy_id,
                                udp_sessions.clone(),
                                traffic_manager.clone(),
                            ).await
                        }
                    };

                    match result {
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
            info!("  [客户端 {}] 启动{}代理: {} 端口: {}",
                  client_id, proxy_protocol_str, proxy.name, proxy.remote_port);
        }

        Ok(())
    }

    // 停止客户端的所有代理监听器
    pub async fn stop_client_proxies(&self, client_id: &str) {
        let mut listeners = self.listeners.write().await;
        if let Some(client_listeners) = listeners.remove(client_id) {
            info!("  [客户端 {}] 停止 {} 个代理监听器", client_id, client_listeners.len());
            for (proxy_id, handle) in client_listeners {
                handle.abort();
                debug!("    代理 #{} 已停止", proxy_id);
            }
        }
    }

    // 动态启动单个代理监听器（用于新增代理时）
    pub async fn start_single_proxy(
        &self,
        client_id: String,
        proxy_id: i64,
        connections: Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>>,
    ) -> Result<()> {
        // 检查客户端是否在线
        let is_online = {
            let conns = connections.read().await;
            conns.contains_key(&client_id)
        };

        if !is_online {
            info!("  [客户端 {}] 离线，跳过启动代理 #{}", client_id, proxy_id);
            return Ok(());
        }

        let db = get_connection().await;

        // 查询指定的代理
        let proxy = match Proxy::find_by_id(proxy_id).one(db).await? {
            Some(p) => p,
            None => {
                warn!("  代理 #{} 不存在", proxy_id);
                return Ok(());
            }
        };

        // 检查代理是否启用且属于该客户端
        if proxy.client_id != client_id {
            warn!("  代理 #{} 不属于客户端 {}", proxy_id, client_id);
            return Ok(());
        }

        if !proxy.enabled {
            info!("  代理 #{} 未启用，跳过启动", proxy_id);
            return Ok(());
        }

        let mut listeners = self.listeners.write().await;
        let client_listeners = listeners.entry(client_id.clone()).or_insert_with(HashMap::new);

        // 如果该代理的监听器已经运行，跳过
        if client_listeners.contains_key(&proxy.id) {
            info!("  代理 #{} 监听器已运行", proxy_id);
            return Ok(());
        }

        let proxy_name = proxy.name.clone();
        let proxy_protocol: ProxyProtocol = proxy.proxy_type.clone().into();
        let proxy_protocol_str = proxy_protocol.as_str().to_uppercase();
        let client_id_clone = client_id.clone();
        let listen_addr = format!("0.0.0.0:{}", proxy.remote_port);
        let target_addr = format!("{}:{}", proxy.local_ip, proxy.local_port);
        let connections_clone = connections.clone();
        let traffic_manager = self.traffic_manager.clone();

        let udp_sessions = self.udp_sessions.clone();

        let handle = tokio::spawn(async move {
            loop {
                let result = match proxy_protocol {
                    ProxyProtocol::Tcp => {
                        run_tcp_proxy_listener(
                            proxy_name.clone(),
                            client_id_clone.clone(),
                            listen_addr.clone(),
                            target_addr.clone(),
                            connections_clone.clone(),
                            proxy_id,
                            traffic_manager.clone(),
                        ).await
                    }
                    ProxyProtocol::Udp => {
                        run_udp_proxy_listener(
                            proxy_name.clone(),
                            client_id_clone.clone(),
                            listen_addr.clone(),
                            target_addr.clone(),
                            connections_clone.clone(),
                            proxy_id,
                            udp_sessions.clone(),
                            traffic_manager.clone(),
                        ).await
                    }
                };

                match result {
                    Ok(_) => {},
                    Err(e) => {
                        error!("[{}] 代理监听失败: {}", proxy_name, e);
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
        info!("  [客户端 {}] 启动{}代理: {} 端口: {}",
              client_id, proxy_protocol_str, proxy.name, proxy.remote_port);

        Ok(())
    }

    // 停止单个代理监听器（用于删除或禁用代理时）
    pub async fn stop_single_proxy(&self, client_id: &str, proxy_id: i64) {
        let mut listeners = self.listeners.write().await;
        if let Some(client_listeners) = listeners.get_mut(client_id) {
            if let Some(handle) = client_listeners.remove(&proxy_id) {
                handle.abort();
                info!("  [客户端 {}] 停止代理 #{}", client_id, proxy_id);
            }
        }
    }
}

impl ProxyServer {
    pub fn new(traffic_manager: Arc<TrafficManager>, config_manager: Arc<ConfigManager>) -> Result<Self> {
        let cert = rcgen::generate_simple_self_signed(&["rfrp".to_string()])?;
        let listener_manager = Arc::new(ProxyListenerManager::new(traffic_manager.clone()));
        let client_connections = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            cert: CertificateDer::from(cert.cert.der().to_vec()),
            key: PrivateKeyDer::from(PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der())),
            traffic_manager,
            listener_manager,
            client_connections,
            config_manager,
        })
    }

    pub fn get_listener_manager(&self) -> Arc<ProxyListenerManager> {
        self.listener_manager.clone()
    }

    pub fn get_client_connections(&self) -> Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>> {
        self.client_connections.clone()
    }

    pub async fn run(&self, bind_addr: String) -> Result<()> {
        // 从配置管理器获取配置
        let idle_timeout = self.config_manager.get_number("idle_timeout", 60).await as u64;
        let max_streams = self.config_manager.get_number("max_concurrent_streams", 100).await as u32;
        let keep_alive_interval = self.config_manager.get_number("keep_alive_interval", 5).await as u64;

        let mut transport_config = TransportConfig::default();
        transport_config.max_concurrent_uni_streams(VarInt::from_u32(max_streams));
        // 服务器也发送心跳，确保连接稳定
        transport_config.keep_alive_interval(Some(Duration::from_secs(keep_alive_interval)));
        transport_config.max_idle_timeout(Some(Duration::from_secs(idle_timeout).try_into()?));

        let mut server_config = ServerConfig::with_single_cert(
            vec![self.cert.clone()],
            self.key.clone_key(),
        )?;
        server_config.transport_config(Arc::new(transport_config));

        let endpoint = Endpoint::server(server_config, bind_addr.parse()?)?;

        info!("🚀 QUIC服务器启动成功!");
        info!("📡 监听地址: {}", bind_addr);
        info!("⏱️  空闲超时: {}秒 (心跳由客户端主动发送)", idle_timeout);
        info!("🔢 最大并发流: {}", max_streams);

        info!("⏳ 等待客户端连接...");

        // 接受客户端连接
        while let Some(connecting) = endpoint.accept().await {
            match connecting.await {
                Ok(conn) => {
                    let remote_addr = conn.remote_address();
                    info!("📡 新连接来自: {}", remote_addr);

                    // 等待客户端发送 token 认证
                    let conn_clone = Arc::new(conn);
                    let connections = self.client_connections.clone();
                    let listener_mgr = self.listener_manager.clone();
                    let config_mgr = self.config_manager.clone();

                    tokio::spawn(async move {
                        debug!("开始处理连接！");
                        if let Err(e) = handle_client_auth(conn_clone, connections, listener_mgr, config_mgr).await {
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
    config_manager: Arc<ConfigManager>,
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

    // 检查该客户端绑定的用户是否有流量超限
    let user_clients = match UserClient::find()
        .filter(user_client::Column::ClientId.eq(client_id))
        .all(db)
        .await
    {
        Ok(ucs) => ucs,
        Err(e) => {
            error!("❌ 查询用户客户端关联失败: {}", e);
            return Ok(());
        }
    };

    // 检查所有关联用户的流量状态
    for uc in user_clients {
        if let Ok(Some(user)) = User::find_by_id(uc.user_id).one(db).await {
            // 如果用户已标记为流量超限，拒绝连接
            if user.is_traffic_exceeded {
                error!("❌ 客户端 {} 认证失败: 用户 {} (#{}) 流量已超限",
                    client_name, user.username, user.id);
                return Ok(());
            }
        }
    }

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

    // 启动连接健康检查任务
    let conn_health_check = conn.clone();
    let client_id_health = client_id;
    let client_name_health = client_name.clone();
    let connections_health = connections.clone();
    let listener_manager_health = listener_manager.clone();

    // 从配置获取健康检查间隔
    let health_check_interval = config_manager.get_number("health_check_interval", 15).await as u64;

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(health_check_interval));
        loop {
            interval.tick().await;

            // 检查连接是否仍然有效
            if conn_health_check.close_reason().is_some() {
                warn!("⚠️  检测到客户端连接已关闭: {}", client_name_health);

                // 清理连接
                let mut conns = connections_health.write().await;
                conns.remove(&format!("{}", client_id_health));
                drop(conns);

                // 停止该客户端的所有代理监听器
                listener_manager_health.stop_client_proxies(&format!("{}", client_id_health)).await;

                // 更新客户端为离线状态
                let db = get_connection().await;
                if let Some(client) = Client::find_by_id(client_id_health).one(db).await.unwrap() {
                    let mut client_active: client::ActiveModel = client.into();
                    client_active.is_online = Set(false);
                    let _ = client_active.update(db).await;
                }
                break;
            }
        }
    });

    // 循环接受代理流请求
    loop {
        match conn.accept_bi().await {
            Ok((send, recv)) => {
                let conn_clone = conn.clone();
                let connections_clone = connections.clone();

                tokio::spawn(async move {
                    // 先读取消息类型
                    let mut msg_type = [0u8; 1];
                    let mut recv = recv;
                    if recv.read_exact(&mut msg_type).await.is_err() {
                        return;
                    }

                    match msg_type[0] {
                        b'h' => {
                            // 心跳请求，回复心跳
                            if let Err(e) = handle_heartbeat(send).await {
                                debug!("心跳处理错误: {}", e);
                            }
                        }
                        _ => {
                            // 其他消息类型，交给代理流处理
                            if let Err(e) = handle_proxy_stream(send, recv, conn_clone, connections_clone).await {
                                error!("❌ 处理代理流错误: {}", e);
                            }
                        }
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

async fn run_tcp_proxy_listener(
    proxy_name: String,
    client_id: String,
    listen_addr: String,
    target_addr: String,
    connections: Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>>,
    proxy_id: i64,
    traffic_manager: Arc<TrafficManager>,
) -> Result<()> {
    let listener = TcpListener::bind(&listen_addr).await?;
    info!("[{}] 🔌 TCP监听端口: {} -> {}", proxy_name, listen_addr, target_addr);

    loop {
        match listener.accept().await {
            Ok((tcp_stream, addr)) => {
                info!("[{}] 📥 新连接来自: {}", proxy_name, addr);

                let connections_clone = connections.clone();
                let client_id = client_id.clone();
                let target_addr = target_addr.clone();
                let proxy_name = proxy_name.clone();
                let traffic_manager = traffic_manager.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_tcp_to_quic(tcp_stream, addr, target_addr, proxy_name, client_id, connections_clone, proxy_id, traffic_manager).await {
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

// UDP代理监听器
async fn run_udp_proxy_listener(
    proxy_name: String,
    client_id: String,
    listen_addr: String,
    target_addr: String,
    connections: Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>>,
    proxy_id: i64,
    udp_sessions: Arc<RwLock<HashMap<(String, i64), HashMap<SocketAddr, UdpSession>>>>,
    traffic_manager: Arc<TrafficManager>,
) -> Result<()> {
    let socket = Arc::new(UdpSocket::bind(&listen_addr).await?);
    info!("[{}] 🔌 UDP监听端口: {} -> {}", proxy_name, listen_addr, target_addr);

    let mut buf = vec![0u8; 65535];
    let session_timeout = Duration::from_secs(300); // 5分钟超时

    // 启动会话清理任务
    let udp_sessions_cleanup = udp_sessions.clone();
    let client_id_clone = client_id.clone();
    let proxy_name_clone = proxy_name.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let mut sessions = udp_sessions_cleanup.write().await;
            let key = (client_id_clone.clone(), proxy_id);
            if let Some(session_map) = sessions.get_mut(&key) {
                let now = tokio::time::Instant::now();
                session_map.retain(|addr, session| {
                    if now.duration_since(session.last_activity) > session_timeout {
                        debug!("[{}] UDP会话超时: {}", proxy_name_clone, addr);
                        false
                    } else {
                        true
                    }
                });
            }
        }
    });

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, src_addr)) => {
                let data = buf[..len].to_vec();
                let connections_clone = connections.clone();
                let client_id = client_id.clone();
                let target_addr = target_addr.clone();
                let proxy_name = proxy_name.clone();
                let udp_sessions = udp_sessions.clone();
                let socket = socket.clone();
                let traffic_manager = traffic_manager.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_udp_to_quic(
                        socket,
                        src_addr,
                        data,
                        target_addr,
                        proxy_name,
                        client_id,
                        connections_clone,
                        proxy_id,
                        udp_sessions,
                        traffic_manager,
                    ).await {
                        error!("❌ 处理UDP错误: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("[{}] ❌ 接收UDP数据失败: {}", proxy_name, e);
            }
        }
    }
}

async fn handle_udp_to_quic(
    socket: Arc<UdpSocket>,
    src_addr: SocketAddr,
    data: Vec<u8>,
    target_addr: String,
    proxy_name: String,
    client_id: String,
    connections: Arc<RwLock<HashMap<String, Arc<quinn::Connection>>>>,
    proxy_id: i64,
    _udp_sessions: Arc<RwLock<HashMap<(String, i64), HashMap<SocketAddr, UdpSession>>>>,
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

    info!("[{}] 🔗 UDP QUIC流已打开: {}", proxy_name, src_addr);

    // 发送协议类型和目标地址 (格式: 1字节协议类型 + 2字节长度 + 地址)
    quic_send.write_all(&[b'u']).await?; // 'u' 表示UDP
    let target_bytes = target_addr.as_bytes();
    let len = target_bytes.len() as u16;
    quic_send.write_all(&len.to_be_bytes()).await?;
    quic_send.write_all(target_bytes).await?;
    quic_send.write_all(&data).await?;
    quic_send.flush().await?;

    // 统计发送字节数
    traffic_manager.record_traffic(
        proxy_id,
        client_id.parse::<i64>().unwrap_or(0),
        None,
        data.len() as i64,
        0,
    ).await;

    // 读取响应并转发回源
    let mut recv_buf = vec![0u8; 65535];
    let mut bytes_received = 0i64;

    loop {
        match quic_recv.read(&mut recv_buf).await? {
            Some(n) => {
                if n == 0 {
                    break;
                }
                bytes_received += n as i64;
                socket.send_to(&recv_buf[..n], src_addr).await?;
            }
            None => break,
        }
    }

    // 统计接收字节数
    if bytes_received > 0 {
        traffic_manager.record_traffic(
            proxy_id,
            client_id.parse::<i64>().unwrap_or(0),
            None,
            0,
            bytes_received,
        ).await;
    }

    quic_send.finish()?;
    Ok(())
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

    // 发送协议类型和目标地址 (格式: 1字节协议类型 + 2字节长度 + 地址)
    quic_send.write_all(&[b't']).await?; // 't' 表示TCP
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

/// 处理心跳请求
async fn handle_heartbeat(mut send: quinn::SendStream) -> Result<()> {
    // 回复心跳 'h'
    send.write_all(&[b'h']).await?;
    send.finish()?;
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
