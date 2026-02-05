use anyhow::Result;
use std::net::SocketAddr;
use socket2::{Socket, Domain, Type, Protocol};

#[cfg(windows)]
fn apply_windows_udp_fix(socket: &Socket) -> Result<()> {
    use std::os::windows::io::AsRawSocket;
    use windows_sys::Win32::Networking::WinSock::{WSAIoctl, SIO_UDP_CONNRESET, SOCKET_ERROR};
    use std::ptr;

    let handle = socket.as_raw_socket();
    let mut bytes_returned: u32 = 0;
    let mut enable: u32 = 0; // FALSE

    unsafe {
        let ret = WSAIoctl(
            handle as usize,
            SIO_UDP_CONNRESET,
            &mut enable as *mut _ as *mut _,
            std::mem::size_of::<u32>() as u32,
            ptr::null_mut(),
            0,
            &mut bytes_returned,
            ptr::null_mut(),
            None,
        );

        if ret == SOCKET_ERROR {
            return Err(anyhow::anyhow!("WSAIoctl failed"));
        }
    }
    Ok(())
}

pub async fn create_configured_udp_socket(addr: SocketAddr) -> Result<tokio::net::UdpSocket> {
    let domain = if addr.is_ipv4() { Domain::IPV4 } else { Domain::IPV6 };
    let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))?;

    socket.set_nonblocking(true)?;

    #[cfg(windows)]
    if let Err(e) = apply_windows_udp_fix(&socket) {
        // Log warning but don't fail? Or fail?
        // Since this is critical for stability on Windows, we should probably fail or at least log.
        // But we don't have tracing here easily unless we add it.
        // Let's just return error.
        return Err(e);
    }

    socket.bind(&addr.into())?;

    let std_socket: std::net::UdpSocket = socket.into();
    let tokio_socket = tokio::net::UdpSocket::from_std(std_socket)?;
    Ok(tokio_socket)
}
