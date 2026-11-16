//! The documented rootshell ABI. TLS peer verification stays in Network.framework.
use std::io::{self, Read, Write};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// Replies share a 64 KiB JSON mailbox. Base64 expands binary data by 4/3;
/// 32 KiB leaves room for JSON fields and call IDs without truncating replies.
pub const MAX_HOST_READ: usize = 32 * 1024;

#[cfg(target_os = "wasi")]
#[link(wasm_import_module = "rootshell_socket")]
unsafe extern "C" {
    fn rootshell_socket_socket(domain: i32, kind: i32, fd_out: *mut i32) -> i32;
    fn rootshell_socket_connect_host(fd: i32, host: *const u8, len: i32, port: i32) -> i32;
    fn rootshell_socket_tls_connect_host(fd: i32, host: *const u8, len: i32, port: i32) -> i32;
    fn rootshell_socket_send(fd: i32, buf: *const u8, len: i32, sent: *mut u32) -> i32;
    fn rootshell_socket_recv(fd: i32, buf: *mut u8, len: i32, received: *mut u32) -> i32;
    fn rootshell_socket_close(fd: i32) -> i32;
    // Required by the supported rootshell 1.0.16 (178) host ABI.
    fn rootshell_socket_set_recv_timeout(fd: i32, timeout_ms: u32) -> i32;
}

pub const RECEIVE_TIMEOUT_REQUIRED: &str = "This installer requires rootshell 1.0.16 (178) with working socket receive timeouts. Other builds are unsupported.";

#[cfg(any(target_os = "wasi", test))]
fn is_timeout_api(code: i32) -> bool {
    code == 8 // A working broker reports EBADF for our invalid descriptor.
}

pub fn receive_timeout_supported() -> bool {
    #[cfg(target_os = "wasi")]
    {
        static SUPPORTED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *SUPPORTED
            .get_or_init(|| is_timeout_api(unsafe { rootshell_socket_set_recv_timeout(-1, 1) }))
    }
    #[cfg(not(target_os = "wasi"))]
    {
        true
    }
}

pub fn require_receive_timeout() -> io::Result<()> {
    if receive_timeout_supported() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            RECEIVE_TIMEOUT_REQUIRED,
        ))
    }
}

#[cfg(target_os = "wasi")]
fn errno(code: i32) -> io::Result<()> {
    use io::ErrorKind::*;
    if code == 0 {
        return Ok(());
    }
    let kind = match code {
        6 => WouldBlock,
        73 => TimedOut,
        14 => ConnectionRefused,
        2 => PermissionDenied,
        27 => Interrupted,
        64 => BrokenPipe,
        28 => InvalidInput,
        50 => NetworkUnreachable,
        _ => Other,
    };
    Err(io::Error::new(
        kind,
        format!("rootshell socket errno {code}"),
    ))
}

#[derive(Debug)]
pub struct Socket {
    #[cfg(target_os = "wasi")]
    fd: i32,
    #[cfg(not(target_os = "wasi"))]
    stream: std::net::TcpStream,
    cooperative: bool,
}

impl Socket {
    pub fn connect(host: &str, port: u16, tls: bool) -> io::Result<Self> {
        let span = perf::span(
            if tls {
                "socket.tls_connect"
            } else {
                "socket.connect"
            },
            &[],
        );
        perf::checkpoint();
        let result = Self::connect_inner(host, port, tls, true);
        span.finish(
            if result.is_ok() {
                perf::Outcome::Ok
            } else {
                perf::Outcome::Error
            },
            &[],
        );
        result
    }

    /// Check TCP reachability without reading or starting a device tunnel.
    pub fn probe_connection(host: &str, port: u16) -> io::Result<()> {
        let span = perf::span("socket.probe_connect", &[]);
        perf::checkpoint();
        let result = Self::connect_inner(host, port, false, false).map(drop);
        span.finish(
            if result.is_ok() {
                perf::Outcome::Ok
            } else {
                perf::Outcome::Error
            },
            &[],
        );
        result
    }

    fn connect_inner(host: &str, port: u16, tls: bool, read_timeout: bool) -> io::Result<Self> {
        if read_timeout {
            require_receive_timeout()?;
        }
        #[cfg(target_os = "wasi")]
        {
            let mut fd = -1;
            unsafe {
                errno(rootshell_socket_socket(2, 1, &mut fd))?;
            }
            let socket = Self {
                fd,
                cooperative: false,
            };
            let code = unsafe {
                if tls {
                    rootshell_socket_tls_connect_host(
                        fd,
                        host.as_ptr(),
                        host.len() as i32,
                        port as i32,
                    )
                } else {
                    rootshell_socket_connect_host(fd, host.as_ptr(), host.len() as i32, port as i32)
                }
            };
            errno(code)?;
            if read_timeout {
                socket.set_read_timeout(Duration::from_secs(30))?;
            }
            Ok(socket)
        }
        #[cfg(not(target_os = "wasi"))]
        {
            if tls {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "host-side TLS requires rootshell",
                ));
            }
            use std::net::ToSocketAddrs;
            let addresses = (host, port).to_socket_addrs()?;
            let mut error = io::Error::new(io::ErrorKind::NotFound, "no address for host");
            for address in addresses {
                match std::net::TcpStream::connect_timeout(&address, Duration::from_secs(10)) {
                    Ok(stream) => {
                        stream.set_nodelay(true)?;
                        let socket = Self {
                            stream,
                            cooperative: false,
                        };
                        if read_timeout {
                            socket.set_read_timeout(Duration::from_secs(30))?;
                        }
                        return Ok(socket);
                    }
                    Err(e) => error = e,
                }
            }
            Err(error)
        }
    }

    pub fn set_read_timeout(&self, duration: Duration) -> io::Result<()> {
        require_receive_timeout()?;
        #[cfg(target_os = "wasi")]
        unsafe {
            errno(rootshell_socket_set_recv_timeout(
                self.fd,
                duration.as_millis().clamp(1, u32::MAX as u128) as u32,
            ))
        }
        #[cfg(not(target_os = "wasi"))]
        {
            self.stream.set_read_timeout(Some(duration))
        }
    }
}

#[cfg(target_os = "wasi")]
impl Drop for Socket {
    fn drop(&mut self) {
        unsafe {
            rootshell_socket_close(self.fd);
        }
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        perf::io_bytes_with_idle("socket.recv", self.cooperative, || {
            if buf.is_empty() {
                return Ok(0);
            }
            #[cfg(target_os = "wasi")]
            {
                let mut count = 0;
                let requested = buf.len().min(MAX_HOST_READ);
                unsafe {
                    errno(rootshell_socket_recv(
                        self.fd,
                        buf.as_mut_ptr(),
                        requested as i32,
                        &mut count,
                    ))?;
                }
                if count as usize > requested {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid socket receive length",
                    ));
                }
                Ok(count as usize)
            }
            #[cfg(not(target_os = "wasi"))]
            {
                self.stream.read(buf)
            }
        })
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        perf::io_bytes("socket.send", || {
            if buf.is_empty() {
                return Ok(0);
            }
            #[cfg(target_os = "wasi")]
            {
                let mut count = 0;
                unsafe {
                    errno(rootshell_socket_send(
                        self.fd,
                        buf.as_ptr(),
                        buf.len().min(i32::MAX as usize) as i32,
                        &mut count,
                    ))?;
                }
                if count as usize > buf.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid socket send length",
                    ));
                }
                Ok(count as usize)
            }
            #[cfg(not(target_os = "wasi"))]
            {
                self.stream.write(buf)
            }
        })
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl AsyncRead for Socket {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if buf.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }
        let this = self.get_mut();
        if !this.cooperative {
            // A blocking, unbounded recv would prevent the TCP adapter and its
            // caller from ever running together on WASI's single thread.
            if let Err(e) = this.set_read_timeout(Duration::from_millis(2)) {
                return Poll::Ready(Err(e));
            }
            this.cooperative = true;
        }
        match Read::read(this, buf.initialize_unfilled()) {
            Ok(n) => {
                buf.advance(n);
                Poll::Ready(Ok(()))
            }
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::Interrupted
                ) =>
            {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl AsyncWrite for Socket {
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(Write::write(self.get_mut(), buf))
    }
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

/// WASI files are already synchronous host calls; no unavailable blocking pool.
pub mod fs {
    use std::{io, path::Path};
    pub async fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
        super::vfs::write(path, contents)
    }
    pub async fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        super::vfs::read(path)
    }
    pub async fn remove_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
        std::fs::remove_dir_all(path)
    }
}

pub mod directory;
pub mod vfs;

pub mod perf;

#[cfg(test)]
mod compatibility_tests {
    #[test]
    fn missing_function_stub_is_not_a_capability() {
        assert!(super::is_timeout_api(8));
        for code in [0, 1, 28, 52, 76, -1] {
            assert!(!super::is_timeout_api(code), "unexpected host errno {code}");
        }
    }

    #[test]
    fn reachability_probe_closes_without_waiting_for_data() {
        use std::io::Read;
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        super::Socket::probe_connection("127.0.0.1", listener.local_addr().unwrap().port())
            .unwrap();
        let (mut peer, _) = listener.accept().unwrap();
        peer.set_read_timeout(Some(std::time::Duration::from_secs(1)))
            .unwrap();
        assert_eq!(peer.read(&mut [0]).unwrap(), 0);
    }
}
