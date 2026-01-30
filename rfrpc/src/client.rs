use anyhow::Result;
use quinn::{ClientConfig, Endpoint, crypto::rustls::QuicClientConfig, TransportConfig};
use rustls::pki_types::ServerName;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::net::{TcpStream, UdpSocket};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, error, warn, debug};
use crate::log_collector::LogCollector;

// 心跳配置
const HEARTBEAT_INTERVAL_SECS: u64 = 10;  // 心跳发送间隔
const HEARTBEAT_TIMEOUT_SECS: u64 = 30;   // 心跳超时时间

pub async fn run(server_addr: SocketAddr, token: String, log_collector: LogCollector) -> Result<()> {
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

    // 创建QUIC端点并保持引用
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);

    info!("🔧 QUIC客户端配置完成");
    info!("🌐 连接到服务器: {}", server_addr);
    info!("⏱️  空闲超时: 60秒, 心跳间隔: 5秒");

    // 连接循环，支持自动重连
    loop {
        match connect_to_server(&endpoint, server_addr, &token, log_collector.clone()).await {
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
    log_collector: LogCollector,
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

            // 启动应用层心跳任务
            let conn_heartbeat = conn.clone();
            let heartbeat_failed = Arc::new(AtomicBool::new(false));
            let heartbeat_failed_clone = heartbeat_failed.clone();

            let mut heartbeat_handle = tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
                let mut consecutive_failures = 0u32;
                const MAX_FAILURES: u32 = 3;

                loop {
                    interval.tick().await;

                    // 检查连接是否仍然有效
                    if conn_heartbeat.close_reason().is_some() {
                        warn!("⚠️  检测到与服务器的连接已断开");
                        heartbeat_failed_clone.store(true, Ordering::SeqCst);
                        break;
                    }

                    // 发送应用层心跳
                    match send_heartbeat(&conn_heartbeat).await {
                        Ok(_) => {
                            consecutive_failures = 0;
                            debug!("💓 心跳发送成功");
                        }
                        Err(e) => {
                            consecutive_failures += 1;
                            warn!("⚠️  心跳发送失败 ({}/{}): {}", consecutive_failures, MAX_FAILURES, e);

                            if consecutive_failures >= MAX_FAILURES {
                                error!("❌ 心跳连续失败 {} 次，判定连接已断开", MAX_FAILURES);
                                heartbeat_failed_clone.store(true, Ordering::SeqCst);
                                break;
                            }
                        }
                    }
                }
            });

            // 循环接受来自服务器的QUIC流
            loop {
                // 检查心跳是否失败
                if heartbeat_failed.load(Ordering::SeqCst) {
                    error!("❌ 心跳检测失败，准备重连");
                    return Err(anyhow::anyhow!("Heartbeat failed"));
                }

                tokio::select! {
                    // 监听心跳任务
                    _ = &mut heartbeat_handle => {
                        error!("❌ 心跳任务结束，准备重连");
                        return Err(anyhow::anyhow!("Heartbeat task ended"));
                    }
                    // 接受新的流
                    result = conn.accept_bi() => {
                        match result {
                            Ok((quic_send, quic_recv)) => {
                                let collector = log_collector.clone();

                                tokio::spawn(async move {
                                    // 读取消息类型（1字节）
                                    let mut msg_type_buf = [0u8; 1];
                                    let mut recv_clone = quic_recv;
                                    if recv_clone.read_exact(&mut msg_type_buf).await.is_err() {
                                        return;
                                    }

                                    match msg_type_buf[0] {
                                        b'p' => {
                                            // 'p' = proxy request (代理请求)
                                            info!("📨 收到新的代理请求");
                                            if let Err(e) = handle_proxy_stream(quic_send, recv_clone).await {
                                                error!("❌ 处理代理流错误: {}", e);
                                            }
                                            info!("🔚 代理流已关闭");
                                        }
                                        b'l' => {
                                            // 'l' = log request (日志请求)
                                            info!("📋 收到日志请求");
                                            if let Err(e) = handle_log_request(quic_send, recv_clone, collector).await {
                                                error!("❌ 处理日志请求错误: {}", e);
                                            }
                                        }
                                        _ => {
                                            warn!("❌ 未知的消息类型: {}", msg_type_buf[0]);
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                error!("❌ 接受流失败: {}", e);
                                return Err(e.into());
                            }
                        }
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
    quic_send: quinn::SendStream,
    mut quic_recv: quinn::RecvStream,
) -> Result<()> {
    // 读取协议类型（1字节）
    let mut proto_buf = [0u8; 1];
    quic_recv.read_exact(&mut proto_buf).await?;
    let protocol_type = proto_buf[0];

    // 读取目标地址（格式：2字节长度 + 内容）
    let mut len_buf = [0u8; 2];
    quic_recv.read_exact(&mut len_buf).await?;
    let len = u16::from_be_bytes(len_buf) as usize;

    let mut addr_buf = vec![0u8; len];
    quic_recv.read_exact(&mut addr_buf).await?;
    let target_addr = String::from_utf8(addr_buf)?;

    info!("🎯 目标地址: {}, 协议: {}", target_addr,
          if protocol_type == b'u' { "UDP" } else { "TCP" });

    // 根据协议类型连接到目标服务
    match protocol_type {
        b't' => {
            // TCP连接
            handle_tcp_proxy(quic_send, quic_recv, &target_addr).await?;
        }
        b'u' => {
            // UDP连接
            handle_udp_proxy(quic_send, quic_recv, &target_addr).await?;
        }
        _ => {
            error!("❌ 未知的协议类型: {}", protocol_type);
            return Err(anyhow::anyhow!("Unknown protocol type: {}", protocol_type));
        }
    }

    Ok(())
}

async fn handle_tcp_proxy(
    mut quic_send: quinn::SendStream,
    mut quic_recv: quinn::RecvStream,
    target_addr: &str,
) -> Result<()> {
    // 连接到目标服务
    let mut tcp_stream = TcpStream::connect(target_addr).await?;

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

async fn handle_udp_proxy(
    mut quic_send: quinn::SendStream,
    mut quic_recv: quinn::RecvStream,
    target_addr: &str,
) -> Result<()> {
    // 绑定一个UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let local_addr = socket.local_addr()?;
    info!("🔗 UDP Socket已绑定: {}", local_addr);

    // 读取来自服务器的初始UDP数据
    let mut recv_buf = vec![0u8; 65535];
    let initial_len = match quic_recv.read(&mut recv_buf).await? {
        Some(n) => n,
        None => {
            error!("❌ 未收到初始UDP数据");
            return Ok(());
        }
    };

    // 将数据发送到目标地址
    socket.send_to(&recv_buf[..initial_len], target_addr).await?;
    debug!("📤 已发送 {} 字节UDP数据到 {}", initial_len, target_addr);

    // 设置TTL
    socket.set_ttl(64)?;

    // 循环接收来自目标的响应并转发回服务器
    loop {
        let mut response_buf = vec![0u8; 65535];
        tokio::select! {
            // 从QUIC读取数据（来自服务器的更多UDP数据包）
            result = quic_recv.read(&mut recv_buf) => {
                match result? {
                    Some(n) => {
                        if n > 0 {
                            // 转发到目标
                            socket.send_to(&recv_buf[..n], target_addr).await?;
                            debug!("📤 转发UDP数据包: {} 字节", n);
                        } else {
                            break;
                        }
                    }
                    None => break,
                }
            }
            // 从目标读取UDP响应
            result = socket.recv_from(&mut response_buf) => {
                match result {
                    Ok((len, _from)) => {
                        // 发送回服务器
                        quic_send.write_all(&response_buf[..len]).await?;
                        debug!("📥 转发UDP响应: {} 字节", len);
                    }
                    Err(e) => {
                        error!("❌ UDP接收错误: {}", e);
                        break;
                    }
                }
            }
        }
    }

    // 关闭QUIC流
    quic_send.finish()?;

    Ok(())
}

/// 发送应用层心跳
/// 心跳协议: 客户端发送 'h' (heartbeat)，服务器回复 'h'
async fn send_heartbeat(conn: &quinn::Connection) -> Result<()> {
    // 打开双向流发送心跳
    let (mut send, mut recv) = tokio::time::timeout(
        Duration::from_secs(HEARTBEAT_TIMEOUT_SECS),
        conn.open_bi()
    ).await.map_err(|_| anyhow::anyhow!("Heartbeat open_bi timeout"))??;

    // 发送心跳请求 'h'
    send.write_all(&[b'h']).await?;
    send.flush().await?;

    // 等待服务器回复
    let mut response = [0u8; 1];
    tokio::time::timeout(
        Duration::from_secs(HEARTBEAT_TIMEOUT_SECS),
        recv.read_exact(&mut response)
    ).await.map_err(|_| anyhow::anyhow!("Heartbeat response timeout"))??;

    if response[0] != b'h' {
        return Err(anyhow::anyhow!("Invalid heartbeat response: {}", response[0]));
    }

    // 关闭流
    send.finish()?;

    Ok(())
}

/// 处理日志请求
async fn handle_log_request(
    mut quic_send: quinn::SendStream,
    mut quic_recv: quinn::RecvStream,
    log_collector: LogCollector,
) -> Result<()> {
    // 读取请求的日志数量（2字节）
    let mut count_buf = [0u8; 2];
    quic_recv.read_exact(&mut count_buf).await?;
    let count = u16::from_be_bytes(count_buf) as usize;

    debug!("📋 请求日志数量: {}", count);

    // 获取日志
    let logs = if count == 0 {
        log_collector.get_all_logs()
    } else {
        log_collector.get_recent_logs(count)
    };

    // 将日志序列化为JSON
    let logs_json = serde_json::to_string(&logs)?;
    let logs_bytes = logs_json.as_bytes();

    // 发送日志数据长度（4字节）
    let len = logs_bytes.len() as u32;
    quic_send.write_all(&len.to_be_bytes()).await?;

    // 发送日志数据
    quic_send.write_all(logs_bytes).await?;
    quic_send.finish()?;

    info!("✅ 已发送 {} 条日志 ({} 字节)", logs.len(), logs_bytes.len());

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
