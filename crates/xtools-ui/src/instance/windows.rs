use std::io::{self, Error, ErrorKind};
use std::sync::{Arc, Mutex};
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS,
    ERROR_PIPE_BUSY, ERROR_PIPE_CONNECTED, ERROR_PIPE_LISTENING, HANDLE,
    INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_GENERIC_READ, FILE_GENERIC_WRITE, OPEN_EXISTING,
};
use windows_sys::Win32::System::IO::{ReadFile, WriteFile};
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PeekNamedPipe,
    PIPE_ACCESS_DUPLEX, PIPE_NOWAIT, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS,
    PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES,
};
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::System::WindowsProgramming::GetUserNameW;

struct Inner {
    handle: HANDLE,
    mutex: HANDLE,
    pipe_name: Vec<u16>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
            CloseHandle(self.mutex);
        }
    }
}

#[derive(Clone)]
pub struct InstanceListener(Arc<Mutex<Inner>>);

fn current_user() -> io::Result<String> {
    let mut user = [0u16; 257];
    let mut len = user.len() as u32;
    if unsafe { GetUserNameW(user.as_mut_ptr(), &mut len) } == 0 {
        return Err(Error::last_os_error());
    }
    Ok(String::from_utf16_lossy(&user[..len.saturating_sub(1) as usize]))
}

fn endpoint_name(name: &str) -> io::Result<String> {
    Ok(format!("{}-{name}", current_user()?))
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn pipe_name(endpoint: &str) -> Vec<u16> {
    wide(&format!(r"\\.\pipe\xtools-{endpoint}"))
}

fn create_pipe(name: &[u16]) -> io::Result<HANDLE> {
    let handle = unsafe {
        CreateNamedPipeW(
            name.as_ptr(),
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_NOWAIT | PIPE_REJECT_REMOTE_CLIENTS,
            PIPE_UNLIMITED_INSTANCES,
            4096,
            4096,
            0,
            std::ptr::null(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        Err(Error::last_os_error())
    } else {
        Ok(handle)
    }
}

pub fn claim_instance(name: &str) -> io::Result<Option<InstanceListener>> {
    let endpoint = endpoint_name(name)?;
    let pipe_name = pipe_name(&endpoint);
    let mutex_name = wide(&format!(r"Local\xtools-{endpoint}-instance"));
    let mutex = unsafe { CreateMutexW(std::ptr::null(), 0, mutex_name.as_ptr()) };
    if mutex == 0 {
        return Err(Error::last_os_error());
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe { CloseHandle(mutex) };
        return Ok(None);
    }
    match create_pipe(&pipe_name) {
        Ok(handle) => Ok(Some(InstanceListener(Arc::new(Mutex::new(Inner {
            handle,
            mutex,
            pipe_name,
        }))))),
        Err(err) => {
            unsafe { CloseHandle(mutex) };
            Err(err)
        }
    }
}

fn connect(name: &[u16]) -> io::Result<Option<HANDLE>> {
    let handle = unsafe {
        CreateFileW(
            name.as_ptr(),
            FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            0,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            0,
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        let code = unsafe { GetLastError() };
        if code == ERROR_PIPE_BUSY || code == ERROR_ACCESS_DENIED {
            Ok(None)
        } else {
            Err(Error::from_raw_os_error(code as i32))
        }
    } else {
        Ok(Some(handle))
    }
}

fn send(name: &str, bytes: &[u8]) -> io::Result<bool> {
    let name = pipe_name(&endpoint_name(name)?);
    let Some(handle) = connect(&name)? else {
        return Ok(false);
    };
    let mut offset = 0;
    while offset < bytes.len() {
        let mut written = 0;
        let result = unsafe {
            WriteFile(
                handle,
                bytes[offset..].as_ptr(),
                (bytes.len() - offset) as u32,
                &mut written,
                std::ptr::null(),
            )
        };
        if result == 0 {
            let err = Error::last_os_error();
            unsafe { CloseHandle(handle) };
            return Err(err);
        }
        if written == 0 {
            unsafe { CloseHandle(handle) };
            return Err(Error::new(ErrorKind::WriteZero, "named-pipe write made no progress"));
        }
        offset += written as usize;
    }
    unsafe { CloseHandle(handle) };
    Ok(true)
}

pub fn terminate_instance(name: &str) -> io::Result<bool> {
    send(name, b"QUIT\n")
}

pub fn raise_instance(name: &str, token: Option<&str>) -> io::Result<bool> {
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
    send(name, line.as_bytes())
}

pub fn accept_command(listener: &InstanceListener) -> Option<super::InstanceCommand> {
    let mut inner = listener.0.lock().ok()?;
    if unsafe { ConnectNamedPipe(inner.handle, std::ptr::null_mut()) } == 0 {
        let code = unsafe { GetLastError() };
        if code != ERROR_PIPE_CONNECTED {
            if code == ERROR_PIPE_LISTENING {
                return None;
            }
            return None;
        }
    }
    let mut available = 0;
    if unsafe {
        PeekNamedPipe(
            inner.handle,
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            &mut available,
            std::ptr::null_mut(),
        )
    } == 0
        || available == 0
    {
        return None;
    }
    let mut buf = [0u8; 4096];
    let mut read = 0;
    if unsafe {
        ReadFile(
            inner.handle,
            buf.as_mut_ptr(),
            buf.len() as u32,
            &mut read,
            std::ptr::null_mut(),
        )
    } == 0
    {
        return None;
    }
    unsafe {
        DisconnectNamedPipe(inner.handle);
    }
    if let Ok(handle) = create_pipe(&inner.pipe_name) {
        unsafe { CloseHandle(inner.handle) };
        inner.handle = handle;
    }
    let line = std::str::from_utf8(&buf[..read as usize])
        .ok()?
        .lines()
        .next()?
        .trim_end();
    if line == "QUIT" {
        return Some(super::InstanceCommand::Quit);
    }
    if line == "RAISE" {
        return Some(super::InstanceCommand::Raise(None));
    }
    let rest = line.strip_prefix("RAISE ")?;
    if rest.is_empty() || rest.contains(' ') || rest.contains('\0') {
        return None;
    }
    Some(super::InstanceCommand::Raise(Some(rest.to_string())))
}
