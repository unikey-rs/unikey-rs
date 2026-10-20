//! Minimal dependency-free Wayland client transport.
//!
//! The project deliberately avoids generated bindings so the AppImage remains
//! small and the composition engine can be tested without a compositor. The
//! transport only implements the small subset needed by the GUI and the
//! input-method v1 bridge.

pub mod input_method;

use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

pub const DISPLAY_OBJECT: u32 = 1;
pub const REGISTRY_OBJECT: u32 = 2;
pub const FIRST_CLIENT_OBJECT: u32 = 0xff00_0000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Global {
    pub name: u32,
    pub interface: String,
    pub version: u32,
}

#[derive(Clone, Debug)]
pub struct Event {
    pub object: u32,
    pub opcode: u16,
    pub body: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct Decoder<'a> {
    body: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(body: &'a [u8]) -> Self {
        Self { body, offset: 0 }
    }

    pub fn u32(&mut self) -> io::Result<u32> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes(bytes.try_into().expect("4 bytes")))
    }

    pub fn i32(&mut self) -> io::Result<i32> {
        Ok(self.u32()? as i32)
    }

    pub fn string(&mut self) -> io::Result<String> {
        let len = self.u32()? as usize;
        if len == 0 {
            return Ok(String::new());
        }
        let bytes = self.take(pad4(len))?;
        let bytes = &bytes[..len.saturating_sub(1)];
        String::from_utf8(bytes.to_vec())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Wayland string is not UTF-8"))
    }

    pub fn array(&mut self) -> io::Result<Vec<u8>> {
        let len = self.u32()? as usize;
        Ok(self.take(pad4(len))?[..len].to_vec())
    }

    pub fn fd(&mut self) -> io::Result<()> {
        Ok(())
    }

    pub fn remaining(&self) -> usize {
        self.body.len().saturating_sub(self.offset)
    }

    fn take(&mut self, len: usize) -> io::Result<&'a [u8]> {
        if self.offset + len > self.body.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "truncated Wayland message",
            ));
        }
        let value = &self.body[self.offset..self.offset + len];
        self.offset += len;
        Ok(value)
    }
}

#[derive(Default)]
pub struct Encoder {
    body: Vec<u8>,
    fds: Vec<i32>,
}

impl Encoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn u32(&mut self, value: u32) -> &mut Self {
        self.body.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub fn i32(&mut self, value: i32) -> &mut Self {
        self.u32(value as u32)
    }

    pub fn string(&mut self, value: &str) -> &mut Self {
        self.u32((value.len() + 1) as u32);
        self.body.extend_from_slice(value.as_bytes());
        self.body.push(0);
        self.pad();
        self
    }

    pub fn array(&mut self, value: &[u8]) -> &mut Self {
        self.u32(value.len() as u32);
        self.body.extend_from_slice(value);
        self.pad();
        self
    }

    pub fn fd(&mut self, fd: impl AsRawFd) -> &mut Self {
        self.fds.push(fd.as_raw_fd());
        self
    }

    pub fn object(&mut self, value: u32) -> &mut Self {
        self.u32(value)
    }

    fn pad(&mut self) {
        let padding = (4 - self.body.len() % 4) % 4;
        self.body.resize(self.body.len() + padding, 0);
    }
}

pub struct Connection {
    stream: UnixStream,
    next_object: u32,
}

impl Connection {
    pub fn connect() -> io::Result<Self> {
        let path = socket_path()?;
        let stream = UnixStream::connect(path)?;
        stream.set_read_timeout(None)?;
        Ok(Self {
            stream,
            next_object: FIRST_CLIENT_OBJECT,
        })
    }

    pub fn alloc_id(&mut self) -> u32 {
        let id = self.next_object;
        self.next_object = self.next_object.wrapping_add(1).max(FIRST_CLIENT_OBJECT);
        id
    }

    pub fn send(&mut self, object: u32, opcode: u16, encoder: &Encoder) -> io::Result<()> {
        let mut message = Vec::with_capacity(8 + encoder.body.len());
        message.extend_from_slice(&object.to_le_bytes());
        message.extend_from_slice(&opcode.to_le_bytes());
        let length = u16::try_from(8 + encoder.body.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Wayland message too large"))?;
        message.extend_from_slice(&length.to_le_bytes());
        message.extend_from_slice(&encoder.body);

        if std::env::var_os("UNIKEY_RS_DEBUG_WAYLAND").is_some() {
            eprintln!("send object={object} opcode={opcode} body_len={} fds={:?}", encoder.body.len(), encoder.fds);
        }
        let result = if encoder.fds.is_empty() {
            self.stream.write_all(&message)
        } else {
            send_with_fds(&self.stream, &message, &encoder.fds)
        };
        result.map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("Wayland send object={object} opcode={opcode}: {error}"),
            )
        })
    }

    pub fn recv(&mut self) -> io::Result<Event> {
        let mut header = [0u8; 8];
        self.stream.read_exact(&mut header)?;
        let object = u32::from_le_bytes(header[0..4].try_into().expect("object"));
        let opcode = u16::from_le_bytes(header[4..6].try_into().expect("opcode"));
        let length = u16::from_le_bytes(header[6..8].try_into().expect("length")) as usize;
        if length < 8 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid Wayland message length"));
        }
        let mut body = vec![0; length - 8];
        self.stream.read_exact(&mut body)?;
        if object == DISPLAY_OBJECT && opcode == 0 {
            let mut decoder = Decoder::new(&body);
            let error_object = decoder.u32()?;
            let code = decoder.u32()?;
            let message = decoder.string()?;
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Wayland protocol error on object {error_object}, code {code}: {message}"),
            ));
        }
        Ok(Event { object, opcode, body })
    }

    pub fn sync(&mut self) -> io::Result<()> {
        let callback = self.alloc_id();
        self.send(DISPLAY_OBJECT, 0, Encoder::new().u32(callback))?;
        loop {
            let event = self.recv()?;
            if event.object == callback && event.opcode == 0 {
                return Ok(());
            }
        }
    }

    pub fn registry(&mut self) -> io::Result<Vec<Global>> {
        self.send(DISPLAY_OBJECT, 1, Encoder::new().u32(REGISTRY_OBJECT))?;
        let callback = self.alloc_id();
        self.send(DISPLAY_OBJECT, 0, Encoder::new().u32(callback))?;
        let mut globals = Vec::new();
        loop {
            let event = self.recv()?;
            if event.object == REGISTRY_OBJECT && event.opcode == 0 {
                let mut decoder = Decoder::new(&event.body);
                let name = decoder.u32()?;
                let interface = decoder.string()?;
                let version = decoder.u32()?;
                globals.push(Global { name, interface, version });
            } else if event.object == callback && event.opcode == 0 {
                return Ok(globals);
            }
        }
    }

    pub fn bind(&mut self, global: &Global, version: u32, id: u32) -> io::Result<()> {
        let mut encoder = Encoder::new();
        encoder.u32(global.name).string(&global.interface).u32(version).u32(id);
        self.send(REGISTRY_OBJECT, 0, &encoder)
    }
}

pub fn socket_path() -> io::Result<PathBuf> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "XDG_RUNTIME_DIR is not set"))?;
    let display = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".to_owned());
    if display.starts_with('/') {
        Ok(PathBuf::from(display))
    } else {
        Ok(runtime.join(display))
    }
}

fn pad4(len: usize) -> usize {
    len.div_ceil(4) * 4
}

/// Read the registry with a synchronous round trip. Used by `doctor` and tests.
pub fn probe_globals() -> io::Result<Vec<Global>> {
    let mut connection = Connection::connect()?;
    connection.registry()
}

#[repr(C)]
struct Iovec {
    base: *const u8,
    len: usize,
}

#[repr(C)]
struct Msghdr {
    name: *mut u8,
    name_len: u32,
    iov: *const Iovec,
    iov_len: usize,
    control: *mut u8,
    control_len: usize,
    flags: i32,
}

extern "C" {
    fn sendmsg(fd: i32, message: *const Msghdr, flags: i32) -> isize;
}

fn send_with_fds(stream: &UnixStream, message: &[u8], fds: &[i32]) -> io::Result<()> {
    if fds.len() != 1 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "only one Wayland fd per message is supported"));
    }
    // `cmsghdr` is pointer-aligned on Linux; an unaligned byte buffer makes
    // `sendmsg` reject the control block with EINVAL.
    let mut control = [0usize; 8];
    let control_bytes = unsafe {
        std::slice::from_raw_parts_mut(control.as_mut_ptr().cast::<u8>(), std::mem::size_of::<[usize; 8]>())
    };
    let cmsg_len = 20usize;
    control_bytes[0..8].copy_from_slice(&cmsg_len.to_ne_bytes());
    control_bytes[8..12].copy_from_slice(&1i32.to_ne_bytes()); // SOL_SOCKET
    control_bytes[12..16].copy_from_slice(&1i32.to_ne_bytes()); // SCM_RIGHTS
    control_bytes[16..20].copy_from_slice(&fds[0].to_ne_bytes());

    let iov = Iovec {
        base: message.as_ptr(),
        len: message.len(),
    };
    let msghdr = Msghdr {
        name: std::ptr::null_mut(),
        name_len: 0,
        iov: &iov,
        iov_len: 1,
        control: control.as_mut_ptr().cast::<u8>(),
        control_len: 24,
        flags: 0,
    };

    let result = unsafe { sendmsg(stream.as_raw_fd(), &msghdr, 0) };
    if result < 0 {
        Err(io::Error::last_os_error())
    } else if result as usize == message.len() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::WriteZero, "partial Wayland message"))
    }
}
