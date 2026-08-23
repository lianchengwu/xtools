use std::io::{self, Read, Write};
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixListener, UnixStream};

/// Bind an abstract `AF_UNIX` name. `Ok(listener)` is the instance lock.
/// `Ok(None)` means another process already holds it.
pub fn claim_instance(name: &str) -> io::Result<Option<UnixListener>> {
    let addr = SocketAddr::from_abstract_name(name.as_bytes())?;
    match UnixListener::bind_addr(&addr) {
        Ok(listener) => {
            listener.set_nonblocking(true)?;
            Ok(Some(listener))
        }
        Err(err) if err.kind() == io::ErrorKind::AddrInUse => Ok(None),
        Err(err) => Err(err),
    }
}

/// Command received on single-instance socket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceCommand {
    Raise(Option<String>),
    Quit,
}

/// Connect to a live instance and write `QUIT\n` to instruct it to terminate.
pub fn terminate_instance(name: &str) -> io::Result<bool> {
    let addr = SocketAddr::from_abstract_name(name.as_bytes())?;
    match UnixStream::connect_addr(&addr) {
        Ok(mut stream) => {
            stream.write_all(b"QUIT\n")?;
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

/// Connect to a live instance and write `RAISE` or `RAISE <token>`.
/// `Ok(true)` wrote the line. `Ok(false)` means no live instance.
pub fn raise_instance(name: &str, token: Option<&str>) -> io::Result<bool> {
    let addr = SocketAddr::from_abstract_name(name.as_bytes())?;
    match UnixStream::connect_addr(&addr) {
        Ok(mut stream) => {
            let line = match token {
                Some(t)
                    if !t.is_empty()
                        && !t.contains('\0')
                        && !t.contains(' ')
                        && !t.contains('\n')
                        && t.len() < 4000 =>
                {
                    format!("RAISE {t}\n")
                }
                _ => "RAISE\n".to_string(),
            };
            stream.write_all(line.as_bytes())?;
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

/// Read one command line from a non-blocking listener. `None` if nothing ready or garbage.
pub fn accept_command(listener: &UnixListener) -> Option<InstanceCommand> {
    let (mut stream, _) = match listener.accept() {
        Ok(pair) => pair,
        Err(err) if err.kind() == io::ErrorKind::WouldBlock => return None,
        Err(_) => return None,
    };
    let _ = stream.set_nonblocking(true);
    let mut buf = [0u8; 4096];
    let n = match stream.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return None,
    };
    if n == 0 || n >= 4096 {
        return None;
    }
    let text = std::str::from_utf8(&buf[..n]).ok()?;
    let line = text.lines().next()?.trim_end();
    if line.contains('\0') {
        return None;
    }
    if line == "QUIT" {
        return Some(InstanceCommand::Quit);
    }
    if line == "RAISE" {
        return Some(InstanceCommand::Raise(None));
    }
    let rest = line.strip_prefix("RAISE ")?;
    if rest.is_empty() || rest.contains(' ') {
        return None;
    }
    Some(InstanceCommand::Raise(Some(rest.to_string())))
}

/// Read one RAISE line from a non-blocking listener. `None` if nothing ready or garbage.
pub fn accept_raise(listener: &UnixListener) -> Option<Option<String>> {
    match accept_command(listener) {
        Some(InstanceCommand::Raise(token)) => Some(token),
        _ => None,
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_and_raise_instance_flow() {
        let socket_name = "xtools-test-instance-flow";
        let lock = claim_instance(socket_name)
            .unwrap()
            .expect("should claim socket");

        // Second claim returns None
        assert!(claim_instance(socket_name).unwrap().is_none());

        // Raise without token
        assert!(raise_instance(socket_name, None).unwrap());
        let raised = accept_raise(&lock).expect("should accept raise");
        assert_eq!(raised, None);

        // Raise with token
        assert!(raise_instance(socket_name, Some("test-token-123")).unwrap());
        let raised_token = accept_raise(&lock).expect("should accept raise with token");
        assert_eq!(raised_token, Some("test-token-123".to_string()));

        // Terminate instance
        assert!(terminate_instance(socket_name).unwrap());
        let cmd = accept_command(&lock).expect("should accept quit command");
        assert_eq!(cmd, InstanceCommand::Quit);
    }
}
