use std::io;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixListener};

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
