use std::net::{SocketAddr, UdpSocket, IpAddr, Ipv4Addr};
use std::io;
use std::fmt;

/// Represents a network address (IP + port)
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArkAddress {
    pub ip: [u8; 4],
    pub port: u16,
}

impl ArkAddress {
    /// Create a new address
    pub fn new(ip: [u8; 4], port: u16) -> Self {
        Self { ip, port }
    }

    /// Create from SocketAddr
    pub fn from_socket_addr(addr: SocketAddr) -> Self {
        match addr.ip() {
            IpAddr::V4(ipv4) => Self {
                ip: ipv4.octets(),
                port: addr.port(),
            },
            IpAddr::V6(_) => panic!("IPv6 not supported"),
        }
    }

    /// Convert to SocketAddr
    pub fn to_socket_addr(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(self.ip[0], self.ip[1], self.ip[2], self.ip[3])), self.port)
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}.{}:{}", self.ip[0], self.ip[1], self.ip[2], self.ip[3], self.port)
    }
}

impl fmt::Debug for ArkAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Display for ArkAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Network packet structure
pub struct ArkPacket {
    pub data: Vec<u8>,
}

impl ArkPacket {
    /// Create empty packet
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }

    /// Create packet with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Create packet from data
    pub fn from_data(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Get packet size
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Clear packet data
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Create a sub-packet from range
    pub fn sub_packet(&self, start: usize, len: usize) -> ArkPacket {
        ArkPacket::from_data(self.data[start..start + len].to_vec())
    }
}

impl Clone for ArkPacket {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

/// UDP Socket wrapper
pub struct ArkSocket {
    socket: UdpSocket,
}

impl ArkSocket {
    /// Create new UDP socket bound to address
    pub fn new(bind_addr: SocketAddr) -> io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self { socket })
    }

    /// Send data to address
    pub fn send_to(&self, addr: &ArkAddress, data: &[u8]) -> io::Result<usize> {
        self.socket.send_to(data, addr.to_socket_addr())
    }

    /// Receive data from address
    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, ArkAddress)> {
        let (size, addr) = self.socket.recv_from(buf)?;
        Ok((size, ArkAddress::from_socket_addr(addr)))
    }

    /// Set blocking mode
    pub fn set_blocking(&self, blocking: bool) -> io::Result<()> {
        self.socket.set_nonblocking(!blocking)
    }

    /// Get local address
    pub fn local_addr(&self) -> io::Result<ArkAddress> {
        let addr = self.socket.local_addr()?;
        Ok(ArkAddress::from_socket_addr(addr))
    }
}

/// Stream for building network packets
pub struct ArkStream {
    data: Vec<u8>,
}

impl ArkStream {
    /// Create new stream
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }

    /// Write u8
    pub fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    /// Write u16 (big endian)
    pub fn write_u16(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    /// Write u32 (big endian)
    pub fn write_u32(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    /// Write i32 (big endian)
    pub fn write_i32(&mut self, value: i32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    /// Write f32 (big endian)
    pub fn write_f32(&mut self, value: f32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    /// Write string (length-prefixed)
    pub fn write_string(&mut self, value: &str) {
        let bytes = value.as_bytes();
        self.write_u32(bytes.len() as u32);
        self.data.extend_from_slice(bytes);
    }

    /// Write raw bytes
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// Read u8
    pub fn read_u8(&mut self) -> Option<u8> {
        if self.data.is_empty() {
            None
        } else {
            Some(self.data.remove(0))
        }
    }

    /// Read u16 (big endian)
    pub fn read_u16(&mut self) -> Option<u16> {
        if self.data.len() < 2 {
            None
        } else {
            let bytes = [self.data[0], self.data[1]];
            self.data.drain(0..2);
            Some(u16::from_be_bytes(bytes))
        }
    }

    /// Read u32 (big endian)
    pub fn read_u32(&mut self) -> Option<u32> {
        if self.data.len() < 4 {
            None
        } else {
            let bytes = [self.data[0], self.data[1], self.data[2], self.data[3]];
            self.data.drain(0..4);
            Some(u32::from_be_bytes(bytes))
        }
    }

    /// Read i32 (big endian)
    pub fn read_i32(&mut self) -> Option<i32> {
        if self.data.len() < 4 {
            None
        } else {
            let bytes = [self.data[0], self.data[1], self.data[2], self.data[3]];
            self.data.drain(0..4);
            Some(i32::from_be_bytes(bytes))
        }
    }

    /// Read f32 (big endian)
    pub fn read_f32(&mut self) -> Option<f32> {
        if self.data.len() < 4 {
            None
        } else {
            let bytes = [self.data[0], self.data[1], self.data[2], self.data[3]];
            self.data.drain(0..4);
            Some(f32::from_be_bytes(bytes))
        }
    }

    /// Read string (length-prefixed)
    pub fn read_string(&mut self) -> Option<String> {
        if let Some(len) = self.read_u32() {
            if self.data.len() < len as usize {
                None
            } else {
                let bytes: Vec<u8> = self.data.drain(0..len as usize).collect();
                Some(String::from_utf8_lossy(&bytes).to_string())
            }
        } else {
            None
        }
    }

    /// Get remaining data
    pub fn remaining_data(&self) -> &[u8] {
        &self.data
    }

    /// Clear stream
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get data as packet
    pub fn to_packet(self) -> ArkPacket {
        ArkPacket::from_data(self.data)
    }

    /// Create from packet
    pub fn from_packet(packet: ArkPacket) -> Self {
        Self {
            data: packet.data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ark_address() {
        let addr = ArkAddress::new([127, 0, 0, 1], 53640);
        assert_eq!(addr.to_string(), "127.0.0.1:53640");
    }

    #[test]
    fn test_stream_primitives() {
        let mut stream = ArkStream::new();

        stream.write_u8(42);
        stream.write_u16(1337);
        stream.write_u32(0xDEADBEEF);
        stream.write_string("Hello World");

        assert_eq!(stream.read_u8(), Some(42));
        assert_eq!(stream.read_u16(), Some(1337));
        assert_eq!(stream.read_u32(), Some(0xDEADBEEF));
        assert_eq!(stream.read_string(), Some("Hello World".to_string()));
    }
}
