use anyhow::Result;
use quinn::{ClientConfig, Endpoint, crypto::rustls::QuicClientConfig, TransportConfig};
use rustls::pki_types::ServerName;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Client {
    server_addr: SocketAddr,
    target_addr: String,
}

impl Client {
    pub fn new(server_addr: SocketAddr, target_addr: String) -> Result<Self> {
        Ok(Self {
            server_addr,
            target_addr,
        })
    }

    pub async fn run(&self) -> Result<()> {
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

        println!("🔧 QUIC客户端配置完成");
        println!("🌐 连接到服务器: {}", self.server_addr);
        println!("🎯 目标服务: {}", self.target_addr);

        // 连接到服务器
        let conn = endpoint
            .connect(self.server_addr, "rfrp")?
            .await?;

        println!("✅ 已连接到服务器: {}", self.server_addr);
        println!("⏳ 等待QUIC流...\n");

        let conn = Arc::new(conn);

        // 循环接受来自服务器的QUIC流
        loop {
            match conn.accept_bi().await {
                Ok((quic_send, quic_recv)) => {
                    println!("📨 收到新的QUIC流");

                    let target_addr = self.target_addr.clone();

                    tokio::spawn(async move {
                        match TcpStream::connect(&target_addr).await {
                            Ok(tcp_stream) => {
                                println!("🔗 已连接到目标服务: {}", target_addr);

                                if let Err(e) =
                                    handle_quic_to_tcp(quic_send, quic_recv, tcp_stream).await
                                {
                                    eprintln!("❌ 处理流错误: {}", e);
                                }
                                println!("🔚 流已关闭");
                            }
                            Err(e) => {
                                eprintln!(
                                    "❌ 连接目标服务失败 ({}): {}",
                                    target_addr, e
                                );
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("❌ 接受流失败: {}", e);
                    eprintln!("⚠️  连接已断开，5秒后重连...");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    // 重连逻辑会在外层循环处理
                    return Err(e.into());
                }
            }
        }

        // 保持endpoint不被drop（实际上不会到达这里，因为loop是无限的）
        std::mem::forget(endpoint);
        Ok(())
    }
}

async fn handle_quic_to_tcp(
    mut quic_send: quinn::SendStream,
    mut quic_recv: quinn::RecvStream,
    mut tcp_stream: TcpStream,
) -> Result<()> {
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
                eprintln!("QUIC->TCP错误: {}", e);
            }
        }
        res = tcp_to_quic => {
            if let Err(e) = res {
                eprintln!("TCP->QUIC错误: {}", e);
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
