use std::rc::Rc;
use std::cell::RefCell;
use crate::arknet::{ArkAddress, ArkPacket, ArkSocket, ArkStream};
use rnr_core::instance::Instance;

/// Listener for peer events
pub trait ArkPeerListener {
    fn on_packet_receiving(&mut self, peer: &ArkPeer, packet: &ArkPacket);
    fn on_connection_accepted(&mut self, peer: &ArkPeer) {}
    fn on_disconnected(&mut self, peer: &ArkPeer) {}
}

/// Network peer representing a connection
pub struct ArkPeer {
    remote_addr: ArkAddress,
    socket: Rc<RefCell<ArkSocket>>,
    listeners: Vec<Box<dyn ArkPeerListener>>,
    authorized: bool,
    user_data: Option<Box<dyn std::any::Any>>,
}

impl ArkPeer {
    /// Create new peer
    pub fn new(socket: Rc<RefCell<ArkSocket>>) -> Self {
        Self {
            remote_addr: ArkAddress::new([0, 0, 0, 0], 0),
            socket,
            listeners: Vec::new(),
            authorized: false,
            user_data: None,
        }
    }

    /// Create peer with remote address
    pub fn with_remote(remote: ArkAddress, socket: Rc<RefCell<ArkSocket>>) -> Self {
        Self {
            remote_addr: remote,
            socket,
            listeners: Vec::new(),
            authorized: false,
            user_data: None,
        }
    }

    /// Add event listener
    pub fn add_listener(&mut self, listener: Box<dyn ArkPeerListener>) {
        self.listeners.push(listener);
    }

    /// Send packet
    pub fn send_packet(&self, packet: &ArkPacket) -> std::io::Result<usize> {
        self.socket.borrow().send_to(&self.remote_addr, &packet.data)
    }

    /// Receive packet
    pub fn recv_packet(&self) -> std::io::Result<ArkPacket> {
        let mut buf = [0u8; 65535];
        let (size, addr) = self.socket.borrow().recv_from(&mut buf)?;

        if addr == self.remote_addr {
            Ok(ArkPacket::from_data(buf[..size].to_vec()))
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Packet from wrong address"))
        }
    }

    /// Authorize peer
    pub fn authorize(&mut self) {
        println!("ArkPeer::authorize: authorized {}", self.remote_addr.to_string());
        self.authorized = true;

        // Notify listeners - collect data first to avoid borrowing issues
        let remote_addr = self.remote_addr;
        let mut listeners: Vec<Box<dyn ArkPeerListener>> = self.listeners.drain(..).collect();

        // Create a temporary peer for notification
        let temp_peer = ArkPeer::with_remote(remote_addr, Rc::clone(&self.socket));

        for listener in &mut listeners {
            listener.on_connection_accepted(&temp_peer);
        }

        // Restore listeners
        self.listeners = listeners;
    }

    /// Disconnect peer
    pub fn disconnect(&mut self, reason: &str, silent: bool) {
        println!("Peer {} disconnected: {}", self.remote_addr.to_string(), reason);

        if !silent {
            // Send disconnect packet
            let mut stream = ArkStream::new();
            stream.write_u8(0xFF); // Disconnect packet type
            stream.write_string(reason);
            let packet = stream.to_packet();
            let _ = self.send_packet(&packet);
        }

        // Notify listeners - collect data first to avoid borrowing issues
        let remote_addr = self.remote_addr;
        let mut listeners: Vec<Box<dyn ArkPeerListener>> = self.listeners.drain(..).collect();

        // Create a temporary peer for notification
        let temp_peer = ArkPeer::with_remote(remote_addr, Rc::clone(&self.socket));

        for listener in &mut listeners {
            listener.on_disconnected(&temp_peer);
        }

        // Note: We don't restore listeners here since we're disconnecting
    }

    /// Check if authorized
    pub fn is_authorized(&self) -> bool {
        self.authorized
    }

    /// Get remote address
    pub fn remote_addr(&self) -> &ArkAddress {
        &self.remote_addr
    }

    /// Set remote address
    pub fn set_remote_addr(&mut self, addr: ArkAddress) {
        self.remote_addr = addr;
    }

    /// Get user data
    pub fn user_data(&self) -> Option<&Box<dyn std::any::Any>> {
        self.user_data.as_ref()
    }

    /// Set user data
    pub fn set_user_data(&mut self, data: Box<dyn std::any::Any>) {
        self.user_data = Some(data);
    }
}

/// Base network peer instance
pub struct NetworkPeer {
    instance: Rc<RefCell<Instance>>,
    peer: Option<Rc<RefCell<ArkPeer>>>,
    socket: Option<Rc<RefCell<ArkSocket>>>,
    running: bool,
}

impl NetworkPeer {
    /// Create new network peer
    pub fn new() -> Rc<RefCell<Self>> {
        let instance = Instance::new();
        instance.borrow_mut().set_name("NetworkPeer");
        instance.borrow_mut().set_class_name("NetworkPeer");

        Rc::new(RefCell::new(Self {
            instance,
            peer: None,
            socket: None,
            running: false,
        }))
    }

    /// Get instance
    pub fn instance(&self) -> &Rc<RefCell<Instance>> {
        &self.instance
    }

    /// Start peer
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Stop peer
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get ark peer
    pub fn ark_peer(&self) -> Option<&Rc<RefCell<ArkPeer>>> {
        self.peer.as_ref()
    }

    /// Set ark peer
    pub fn set_ark_peer(&mut self, peer: Rc<RefCell<ArkPeer>>) {
        self.peer = Some(peer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, IpAddr, Ipv4Addr};

    #[test]
    fn test_network_peer_creation() {
        let peer = NetworkPeer::new();
        assert_eq!(peer.borrow().instance().borrow().name(), "NetworkPeer");
        assert_eq!(peer.borrow().instance().borrow().class_name(), "NetworkPeer");
        assert!(!peer.borrow().is_running());
    }

    #[test]
    fn test_ark_peer_basic() {
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
        let socket = Rc::new(RefCell::new(ArkSocket::new(socket_addr).unwrap()));
        let mut peer = ArkPeer::with_remote(ArkAddress::new([127, 0, 0, 1], 53640), socket);

        assert!(!peer.is_authorized());
        peer.authorize();
        assert!(peer.is_authorized());
    }
}
