//! Small reqwest-compatible surface for isideload on rootshell. No JS, native
//! binaries, blocking threads, TLS bypass, or credentials in error strings.
use header::{HeaderMap, HeaderName, HeaderValue};
pub use http::{header, Method, StatusCode};
use rootshell_runtime::Socket;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::time::{Duration, Instant};
use url::Url;

const MAX_BODY: usize = 16 * 1024 * 1024;
const MAX_HEADERS: usize = 64 * 1024;

#[derive(Debug)]
pub struct Error(String, Option<io::ErrorKind>, bool);
impl Error {
    pub fn is_timeout(&self) -> bool {
        matches!(
            self.1,
            Some(io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock)
        )
    }
    pub fn is_connect(&self) -> bool {
        self.2
    }
    fn connect(e: io::Error) -> Self {
        Self(format!("connection failed: {e}"), Some(e.kind()), true)
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self(format!("network I/O: {e}"), Some(e.kind()), false)
    }
}
fn invalid(message: &str) -> Error {
    Error(message.to_string(), None, false)
}

#[derive(Clone, Debug)]
pub struct Client {
    timeout: Duration,
}
impl Default for Client {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
        }
    }
}
impl Client {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }
    pub fn request(&self, method: Method, url: impl AsRef<str>) -> RequestBuilder {
        RequestBuilder {
            method,
            url: url.as_ref().into(),
            headers: HeaderMap::new(),
            body: Vec::new(),
            timeout: self.timeout,
            invalid_header: false,
        }
    }
    pub fn get(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request(Method::GET, url)
    }
    pub fn post(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request(Method::POST, url)
    }
    pub fn patch(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request(Method::PATCH, url)
    }
}

#[derive(Default)]
pub struct ClientBuilder {
    client: Client,
    invalid_tls: bool,
}
impl ClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_root_certificate(self, _: Certificate) -> Self {
        self
    }
    pub fn http1_title_case_headers(self) -> Self {
        self
    }
    pub fn danger_accept_invalid_certs(mut self, enabled: bool) -> Self {
        self.invalid_tls = enabled;
        self
    }
    pub fn connection_verbose(self, _: bool) -> Self {
        self
    }
    pub fn pool_max_idle_per_host(self, _: usize) -> Self {
        self
    }
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.client.timeout = timeout;
        self
    }
    pub fn build(self) -> Result<Client, Error> {
        if self.invalid_tls {
            Err(invalid("rootshell requires verified TLS"))
        } else {
            Ok(self.client)
        }
    }
}

pub struct Certificate;
impl Certificate {
    pub fn from_der(der: &[u8]) -> Result<Self, Error> {
        if der.is_empty() {
            Err(invalid("empty root certificate"))
        } else {
            Ok(Self)
        }
    }
}

#[derive(Clone)]
pub struct RequestBuilder {
    method: Method,
    url: String,
    headers: HeaderMap,
    body: Vec<u8>,
    timeout: Duration,
    invalid_header: bool,
}
impl RequestBuilder {
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers.extend(headers);
        self
    }
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: TryInto<HeaderName>,
        V: TryInto<HeaderValue>,
    {
        match (key.try_into(), value.try_into()) {
            (Ok(key), Ok(value)) => {
                self.headers.insert(key, value);
            }
            _ => self.invalid_header = true,
        }
        self
    }
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }
    pub async fn send(self) -> Result<Response, Error> {
        let span = rootshell_runtime::perf::span(
            "http.request",
            &[("request_bytes", self.body.len() as u64)],
        );
        rootshell_runtime::perf::checkpoint();
        let response = self.send_blocking();
        match &response {
            Ok(response) => span.finish(
                rootshell_runtime::perf::Outcome::Ok,
                &[
                    ("status", response.status.as_u16() as u64),
                    ("response_bytes", response.body.len() as u64),
                ],
            ),
            Err(error) => span.finish(
                rootshell_runtime::perf::Outcome::Error,
                &[
                    ("timeout", u64::from(error.is_timeout())),
                    ("connect_error", u64::from(error.is_connect())),
                ],
            ),
        }
        response
    }

    fn send_blocking(self) -> Result<Response, Error> {
        if self.invalid_header {
            return Err(invalid("invalid HTTP header"));
        }
        let url = Url::parse(&self.url).map_err(|_| invalid("invalid request URL"))?;
        let tls = match url.scheme() {
            "https" => true,
            "http" => false,
            _ => return Err(invalid("HTTP(S) required")),
        };
        if !url.username().is_empty() || url.password().is_some() {
            return Err(invalid("URL credentials are not supported"));
        }
        let host = url
            .host_str()
            .ok_or_else(|| invalid("request host missing"))?;
        // Apple login must never travel over unencrypted HTTP. Plain HTTP is
        // reserved for loopback fixtures and an explicitly configured local ADI.
        if !tls && !matches!(host, "127.0.0.1" | "localhost" | "[::1]") {
            return Err(invalid("remote HTTP requires TLS"));
        }
        let port = url
            .port_or_known_default()
            .ok_or_else(|| invalid("request port missing"))?;
        let mut socket =
            Socket::connect(host.trim_matches(['[', ']']), port, tls).map_err(Error::connect)?;
        socket.set_read_timeout(self.timeout)?;
        let mut path = url.path().to_string();
        if path.is_empty() {
            path.push('/');
        }
        if let Some(query) = url.query() {
            path.push('?');
            path.push_str(query);
        }
        write!(socket, "{} {path} HTTP/1.1\r\nHost: {}", self.method, host)?;
        if url.port().is_some() {
            write!(socket, ":{port}")?;
        }
        write!(
            socket,
            "\r\nConnection: close\r\nAccept-Encoding: gzip\r\nContent-Length: {}\r\n",
            self.body.len()
        )?;
        for (key, value) in &self.headers {
            if matches!(
                key.as_str(),
                "host" | "connection" | "content-length" | "accept-encoding" | "transfer-encoding"
            ) {
                continue;
            }
            // HTTP HeaderValue already rejects CR/LF; write bytes verbatim.
            let title = key
                .as_str()
                .split('-')
                .map(|part| {
                    let mut bytes = part.bytes();
                    match bytes.next() {
                        Some(first) => format!(
                            "{}{}",
                            (first as char).to_ascii_uppercase(),
                            bytes.map(char::from).collect::<String>()
                        ),
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join("-");
            write!(socket, "{title}: ")?;
            socket.write_all(value.as_bytes())?;
            socket.write_all(b"\r\n")?;
        }
        socket.write_all(b"\r\n")?;
        socket.write_all(&self.body)?;
        socket.flush()?;
        read_response(socket, self.timeout)
    }
}

pub struct Response {
    status: StatusCode,
    body: Vec<u8>,
}
impl Response {
    pub fn status(&self) -> StatusCode {
        self.status
    }
    pub fn error_for_status(self) -> Result<Self, Error> {
        if self.status.is_client_error()
            || self.status.is_server_error()
            || self.status.is_redirection()
        {
            Err(Error(format!("HTTP {}", self.status), None, false))
        } else {
            Ok(self)
        }
    }
    pub async fn text(self) -> Result<String, Error> {
        String::from_utf8(self.body).map_err(|_| invalid("response is not UTF-8"))
    }
    pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<T, Error> {
        serde_json::from_slice(&self.body).map_err(|_| invalid("invalid JSON response"))
    }
    pub async fn bytes(self) -> Result<Vec<u8>, Error> {
        Ok(self.body)
    }
}

fn read_response(socket: Socket, timeout: Duration) -> Result<Response, Error> {
    let deadline = Instant::now() + timeout;
    let mut input = BufReader::new(socket);
    let mut consumed = 0;
    let (status, headers) = loop {
        let line = bounded_line(&mut input, &mut consumed, deadline)?;
        let mut fields = line.split_ascii_whitespace();
        if !matches!(fields.next(), Some("HTTP/1.1" | "HTTP/1.0")) {
            return Err(invalid("invalid HTTP response"));
        }
        let status = fields
            .next()
            .and_then(|s| s.parse::<u16>().ok())
            .and_then(|s| StatusCode::from_u16(s).ok())
            .ok_or_else(|| invalid("invalid HTTP status"))?;
        let mut headers = HeaderMap::new();
        loop {
            let line = bounded_line(&mut input, &mut consumed, deadline)?;
            if line.is_empty() {
                break;
            }
            let (name, value) = line
                .split_once(':')
                .ok_or_else(|| invalid("malformed HTTP header"))?;
            let name = HeaderName::from_bytes(name.as_bytes())
                .map_err(|_| invalid("invalid response header name"))?;
            let value = HeaderValue::from_str(value.trim())
                .map_err(|_| invalid("invalid response header value"))?;
            if name == header::CONTENT_LENGTH && headers.contains_key(&name) {
                return Err(invalid("duplicate Content-Length"));
            }
            headers.append(name, value);
        }
        if status.is_informational() {
            if status == StatusCode::SWITCHING_PROTOCOLS {
                return Err(invalid("unexpected protocol switch"));
            }
            continue;
        }
        break (status, headers);
    };
    let mut body = Vec::new();
    if status != StatusCode::NO_CONTENT && status != StatusCode::NOT_MODIFIED {
        if let Some(encoding) = headers.get(header::TRANSFER_ENCODING) {
            if headers.contains_key(header::CONTENT_LENGTH)
                || !encoding.as_bytes().eq_ignore_ascii_case(b"chunked")
            {
                return Err(invalid("ambiguous HTTP body framing"));
            }
            loop {
                let line = bounded_line(&mut input, &mut consumed, deadline)?;
                let size = usize::from_str_radix(line.split(';').next().unwrap_or("").trim(), 16)
                    .map_err(|_| invalid("invalid chunk size"))?;
                if size == 0 {
                    while !bounded_line(&mut input, &mut consumed, deadline)?.is_empty() {}
                    break;
                }
                read_exact_bounded(&mut input, &mut body, size, deadline)?;
                let mut ending = [0; 2];
                input.read_exact(&mut ending)?;
                if ending != *b"\r\n" {
                    return Err(invalid("invalid chunk ending"));
                }
            }
        } else if let Some(length) = headers.get(header::CONTENT_LENGTH) {
            let length = length
                .to_str()
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or_else(|| invalid("invalid Content-Length"))?;
            read_exact_bounded(&mut input, &mut body, length, deadline)?;
        } else {
            loop {
                check_deadline(deadline)?;
                let mut block = [0; 8192];
                let count = input.read(&mut block)?;
                if count == 0 {
                    break;
                }
                if body.len() + count > MAX_BODY {
                    return Err(invalid("HTTP body exceeds 16 MiB"));
                }
                body.extend_from_slice(&block[..count]);
            }
        }
    }
    if let Some(encoding) = headers.get(header::CONTENT_ENCODING) {
        if encoding.as_bytes().eq_ignore_ascii_case(b"gzip") {
            let mut decoded = Vec::new();
            flate2::read::GzDecoder::new(body.as_slice())
                .take(MAX_BODY as u64 + 1)
                .read_to_end(&mut decoded)?;
            if decoded.len() > MAX_BODY {
                return Err(invalid("decoded HTTP body exceeds 16 MiB"));
            }
            body = decoded;
        } else if !encoding.as_bytes().eq_ignore_ascii_case(b"identity") {
            return Err(invalid("unsupported Content-Encoding"));
        }
    }
    Ok(Response { status, body })
}

fn check_deadline(deadline: Instant) -> Result<(), Error> {
    if Instant::now() >= deadline {
        Err(Error(
            "HTTP response timed out".into(),
            Some(io::ErrorKind::TimedOut),
            false,
        ))
    } else {
        Ok(())
    }
}
fn bounded_line(
    input: &mut impl BufRead,
    consumed: &mut usize,
    deadline: Instant,
) -> Result<String, Error> {
    let mut line = Vec::new();
    loop {
        check_deadline(deadline)?;
        let available = input.fill_buf()?;
        if available.is_empty() {
            return Err(invalid("truncated HTTP headers"));
        }
        let count = available
            .iter()
            .position(|b| *b == b'\n')
            .map(|i| i + 1)
            .unwrap_or(available.len());
        *consumed += count;
        if *consumed > MAX_HEADERS {
            return Err(invalid("HTTP headers exceed 64 KiB"));
        }
        line.extend_from_slice(&available[..count]);
        input.consume(count);
        if line.last() == Some(&b'\n') {
            break;
        }
    }
    if !line.ends_with(b"\r\n") {
        return Err(invalid("HTTP requires CRLF"));
    }
    line.truncate(line.len() - 2);
    String::from_utf8(line).map_err(|_| invalid("invalid HTTP header encoding"))
}
fn read_exact_bounded(
    input: &mut impl Read,
    body: &mut Vec<u8>,
    length: usize,
    deadline: Instant,
) -> Result<(), Error> {
    if length > MAX_BODY.saturating_sub(body.len()) {
        return Err(invalid("HTTP body exceeds 16 MiB"));
    }
    let mut left = length;
    while left != 0 {
        check_deadline(deadline)?;
        let mut block = [0; 8192];
        let count = input.read(&mut block[..left.min(8192)])?;
        if count == 0 {
            return Err(invalid("truncated HTTP body"));
        }
        body.extend_from_slice(&block[..count]);
        left -= count;
    }
    Ok(())
}

pub mod websocket;

#[cfg(test)]
mod tests {
    use super::*;

    fn request(response: Vec<u8>) -> Result<Response, Error> {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut head = Vec::new();
            while !head.ends_with(b"\r\n\r\n") {
                let mut byte = [0];
                stream.read_exact(&mut byte).unwrap();
                head.push(byte[0]);
            }
            assert!(head.starts_with(b"GET /bag?test=1 HTTP/1.1\r\n"));
            stream.write_all(&response).unwrap();
        });
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        let result = runtime.block_on(
            Client::new()
                .get(format!("http://{address}/bag?test=1"))
                .send(),
        );
        server.join().unwrap();
        result
    }

    #[test]
    fn bounded_chunked_gzip_and_interim_responses() {
        let mut gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        gzip.write_all(b"{\"ok\":true}").unwrap();
        let gzip = gzip.finish().unwrap();
        let mut response = b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nTransfer-Encoding: chunked\r\n\r\n".to_vec();
        response.extend(format!("{:x};test=yes\r\n", gzip.len()).bytes());
        response.extend(gzip);
        response.extend(b"\r\n0\r\nX-Trailer: yes\r\n\r\n");
        let response = request(response).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&response.body).unwrap()["ok"],
            true
        );
    }

    #[test]
    fn rejects_truncation_smuggling_oversized_and_malformed_responses() {
        for wire in [
            "HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\nabc",
            "HTTP/1.1 200 OK\r\nContent-Length: 1\r\nContent-Length: 2\r\n\r\nx",
            "HTTP/1.1 200 OK\r\nContent-Length: 1\r\nTransfer-Encoding: chunked\r\n\r\nx",
            "HTTP/1.1 200 OK\r\nContent-Length: 999999999\r\n\r\n",
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nwrong\r\n",
            "HTTP/1.1 200 OK\n\n",
        ] {
            assert!(request(wire.as_bytes().to_vec()).is_err());
        }
    }

    #[test]
    fn fails_closed_on_redirects_and_tls_bypass_without_leaking_credentials() {
        assert!(request(
            b"HTTP/1.1 302 Found\r\nLocation: https://other.invalid\r\nContent-Length: 0\r\n\r\n"
                .to_vec()
        )
        .unwrap()
        .error_for_status()
        .is_err());
        assert!(Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .is_err());
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let error = runtime
            .block_on(
                Client::new()
                    .get("http://apple.example/secret?password=must-not-appear")
                    .send(),
            )
            .err()
            .unwrap();
        assert!(!error.to_string().contains("must-not-appear"));
        assert!(!error.to_string().contains("secret"));
    }
}
