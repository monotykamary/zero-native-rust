use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::Command as StdCommand;
use std::time::Duration;

use super::manifest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    MissingFrontend,
    MissingDevConfig,
    MissingBinary,
    InvalidUrl,
    Timeout,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub metadata: manifest::Metadata,
    pub binary_path: Option<String>,
    pub url_override: Option<String>,
    pub command_override: Option<Vec<String>>,
    pub timeout_ms: Option<u32>,
}

pub fn run(options: &Options) -> Result<(), Error> {
    let frontend = options.metadata.frontend.as_ref().ok_or(Error::MissingFrontend)?;
    let dev = frontend.dev.as_ref().ok_or(Error::MissingDevConfig)?;
    let url = options.url_override.as_deref().unwrap_or(&dev.url);
    let command = options.command_override.as_deref().unwrap_or(&dev.command);
    let timeout_ms = options.timeout_ms.unwrap_or(dev.timeout_ms);

    let mut dev_child = if !command.is_empty() {
        let mut cmd = StdCommand::new(&command[0]);
        cmd.args(&command[1..]);
        Some(cmd.spawn().map_err(|_| Error::MissingBinary)?)
    } else {
        None
    };

    wait_until_ready(url, &dev.ready_path, timeout_ms)?;

    let binary_path = options.binary_path.as_deref().ok_or(Error::MissingBinary)?;

    let mut app_cmd = StdCommand::new(binary_path);
    app_cmd.env("ZERO_NATIVE_FRONTEND_URL", url);
    app_cmd.env("ZERO_NATIVE_MODE", "dev");
    app_cmd.env("ZERO_NATIVE_HMR", "1");

    let result = app_cmd.status();

    if let Some(ref mut child) = dev_child {
        let _ = child.kill();
    }

    match result {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                eprintln!("app exited with status: {}", status);
                Ok(())
            }
        }
        Err(e) => {
            eprintln!("failed to launch app: {}", e);
            Err(Error::MissingBinary)
        }
    }
}

#[derive(Debug, Clone)]
pub struct UrlParts {
    pub host: String,
    pub port: u16,
    pub path: String,
}

pub fn parse_http_url(url: &str) -> Result<UrlParts, Error> {
    let (default_port, prefix_len) = if url.starts_with("http://") {
        (80, 7)
    } else if url.starts_with("https://") {
        (443, 8)
    } else {
        return Err(Error::InvalidUrl);
    };

    let rest = &url[prefix_len..];
    let slash_idx = rest.find('/').unwrap_or(rest.len());
    let host_port = &rest[..slash_idx];
    if host_port.is_empty() {
        return Err(Error::InvalidUrl);
    }
    let path = if slash_idx < rest.len() {
        rest[slash_idx..].to_string()
    } else {
        "/".to_string()
    };

    if let Some(colon) = host_port.rfind(':') {
        if colon == 0 || colon + 1 >= host_port.len() {
            return Err(Error::InvalidUrl);
        }
        let host = host_port[..colon].to_string();
        let port = host_port[colon + 1..]
            .parse::<u16>()
            .map_err(|_| Error::InvalidUrl)?;
        Ok(UrlParts { host, port, path })
    } else {
        Ok(UrlParts {
            host: host_port.to_string(),
            port: default_port,
            path,
        })
    }
}

fn wait_until_ready(url: &str, ready_path: &str, timeout_ms: u32) -> Result<(), Error> {
    let parts = parse_http_url(url)?;
    let host = if parts.host == "localhost" {
        "127.0.0.1"
    } else {
        &parts.host
    };
    let path = if !ready_path.is_empty() {
        ready_path
    } else {
        &parts.path
    };

    let mut waited_ms: u32 = 0;
    while waited_ms <= timeout_ms {
        let addr = format!("{}:{}", host, parts.port);
        if let Ok(mut stream) = TcpStream::connect_timeout(
            &addr.parse().map_err(|_| Error::InvalidUrl)?,
            Duration::from_millis(100),
        ) {
            if http_ready(&mut stream, host, path) {
                return Ok(());
            }
        }
        std::thread::sleep(Duration::from_millis(100));
        waited_ms += 100;
    }
    Err(Error::Timeout)
}

fn http_ready(stream: &mut TcpStream, host: &str, path: &str) -> bool {
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, host
    );
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }
    let _ = stream.flush();

    let mut response = [0u8; 64];
    match stream.read(&mut response) {
        Ok(len) if len > 0 => {
            let resp = String::from_utf8_lossy(&response[..len]);
            resp.starts_with("HTTP/1.1 2")
                || resp.starts_with("HTTP/1.0 2")
                || resp.starts_with("HTTP/1.1 3")
                || resp.starts_with("HTTP/1.0 3")
        }
        _ => false,
    }
}
