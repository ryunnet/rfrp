use anyhow::Result;
use quinn::{Endpoint, ServerConfig, TransportConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Server {
    bind_addr: SocketAddr,
    target_port: u16,
    cert: CertificateDer<'static>,
    key: PrivateKeyDer<'static>,
}

impl Server {
    pub fn new(bind_addr: SocketAddr, target_port: u16) -> Result<Self> {
        // 生成自签名证书
        let cert = rcgen::generate_simple_self_signed(&["rfrp".to_string()])?;

        Ok(Self {
            bind_addr,
            target_port,
            cert: CertificateDer::from(cert.cert.der().to_vec()),
            key: PrivateKeyDer::from(PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der())),
        })
    }

    pub async fn run(&self) -> Result<()> {
        // 创建传输配置
        let mut transport_config = TransportConfig::default();
        transport_config.max_concurrent_uni_streams(0u32.into());
        transport_config.keep_alive_interval(Some(Duration::from_secs(5)));
        transport_config.max_idle_timeout(Some(Duration::from_secs(60).try_into()?));

        // 创建服务器配置
        let mut server_config = ServerConfig::with_single_cert(
            vec![self.cert.clone()],
            self.key.clone_key(),
        )?;
        server_config.transport_config(Arc::new(transport_config));

        // 创建QUIC端点
        let endpoint = Endpoint::server(server_config, self.bind_addr)?;

        println!("🚀 QUIC服务器启动成功!");
        println!("📡 监听地址: {}", self.bind_addr);
        println!("🎯 目标端口: {}", self.target_port);

        // 接受客户端连接
        println!("⏳ 等待客户端连接...");

        while let Some(connecting) = endpoint.accept().await {
            match connecting.await {
                Ok(conn) => {
                    println!("✅ 客户端已连接: {}", conn.remote_address());

                    let conn = Arc::new(conn);

                    // 监听TCP端口并转发到客户端
                    let listen_addr = format!("0.0.0.0:{}", self.target_port);
                    let listener = TcpListener::bind(&listen_addr).await?;

                    println!("🔌 开始监听TCP端口: {}", listen_addr);
                    println!("🌍 准备接受连接...\n");

                    // 接受并处理连接
                    while let Ok((tcp_stream, addr)) = listener.accept().await {
                        println!("📥 新TCP连接来自: {}", addr);

                        let conn_clone = Arc::clone(&conn);
                        tokio::spawn(async move {
                            if let Err(e) = handle_tcp_to_quic(tcp_stream, conn_clone, addr).await {
                                eprintln!("❌ 处理连接错误 ({}): {}", addr, e);
                            }
                            println!("🔚 连接已关闭: {}", addr);
                        });
                    }
                    break;
                }
                Err(e) => {
                    eprintln!("❌ 连接接受失败: {}", e);
                    continue;
                }
            }
        }

        Ok(())
    }
}

async fn handle_tcp_to_quic(
    mut tcp_stream: TcpStream,
    conn: Arc<quinn::Connection>,
    addr: SocketAddr,
) -> Result<()> {
    // 打开双向QUIC流
    let (mut quic_send, mut quic_recv) = conn.open_bi().await?;

    println!("🔗 QUIC流已打开: {}", addr);

    // 获取TCP读写端
    let (mut tcp_read, mut tcp_write) = tcp_stream.split();

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

    tokio::select! {
        res = tcp_to_quic => {
            if let Err(e) = res {
                eprintln!("TCP->QUIC错误: {}", e);
            }
        }
        res = quic_to_tcp => {
            if let Err(e) = res {
                eprintln!("QUIC->TCP错误: {}", e);
            }
        }
    }

    // 关闭QUIC流
    quic_send.finish()?;

    Ok(())
}
