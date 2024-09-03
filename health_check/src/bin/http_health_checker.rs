use std::env::VarError;
use std::net::{Shutdown, TcpStream};
use core::time::Duration;
use std::io::{Read, Write};
use std::process::exit;
//--------------------------------------------------------------------------------------------------


pub fn main() -> Result<(), Box<dyn std::error::Error>> {

    // let is_ok = health_check("www.google.com", 80, "/")?;
    let port = get_port() ?;
    let is_ok = health_check("localhost", port, "/healthcheck") ?;

    if !is_ok {
        exit(2);
    }

    Ok(())
}


fn get_port() -> Result<u16, Box<dyn std::error::Error>> {
    let port = std::env::var("SERVER_PORT");

    let port: u16 = match port {
        Ok(port) =>
            core::str::FromStr::from_str(&port)?,
        Err(VarError::NotPresent) =>
            8080,
        Err(VarError::NotUnicode(port)) => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other, format!("SERVER_PORT env is incorrect [{port:?}]")
            )));
        }
    };

    Ok(port)
}


fn health_check(host: &str, port: u16, url_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let msg = format!("\
        GET {url_path} HTTP/1.1\r\n\
        Host: {host}\r\n\
        Connection: close\r\n
        Accept: text/plain\r\n\
        Accept-Encoding: \r\n\
        \r\n\
        ");

    let timeout = Duration::from_secs(2);
    let mut stream = connect(host, port, timeout) ?;
    stream.set_read_timeout(Some(timeout)) ?;
    stream.set_write_timeout(Some(timeout)) ?;

    stream.write(msg.as_bytes())?;

    let mut out: [u8; 512] = [0; 512];
    stream.read(&mut out) ?;

    let str = String::from_utf8_lossy(&out);

    stream.shutdown(Shutdown::Both) ?;

    let is_ok = str.contains("200 OK");
    Ok(is_ok)
}

fn connect(host: &str, port: u16, timeout: Duration) -> Result<TcpStream, Box<dyn std::error::Error>> { // TODO: try to return std::io::Error
    use std::net::ToSocketAddrs;

    let host_with_port = format!("{host}:{port}");

    let addrs = match host_with_port.as_str().to_socket_addrs() {
        Ok(addrs) => addrs,
        Err(err) => return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other, format!("No IP for [{host}] ({err:?})")
        ))),
    };

    let mut last_err = None;
    for addr in addrs {
        match TcpStream::connect_timeout(&addr, timeout.clone()) {
            Ok(tcp_stream) =>
                return Ok(tcp_stream),
            Err(err) =>
                last_err = Some(err),
        }
    }

    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other, format!("Could not resolve to any addresses for [{host}] ({last_err:?})")
    )))
}
