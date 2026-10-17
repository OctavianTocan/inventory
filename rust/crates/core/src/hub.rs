//! Local SignalR JSON hub over WebSockets (`lib/hub.mjs`): the handshake, record splitting, pings and which clients
//! get a push, behind socket traits the native server and the Worker implement.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use mm_js::{Value, base64, json, obj, text};
use sha1::{Digest, Sha1};

use crate::Result;
use crate::http::Request;
use crate::runtime;

const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
const LIMIT: usize = 1_048_576;
const PING_MS: u64 = 15_000;
const RECORD_END: char = '\x1e';

/// A raw upgraded TCP socket, for hosts that receive bytes (Node's `socket`).
pub trait Socket {
    /// Write bytes to the client.
    ///
    /// # Arguments
    /// * `bytes` - Data to send.
    fn write(&self, bytes: &[u8]);

    /// Finish writing and close the connection (`socket.end()`).
    fn end(&self);

    /// Close the connection at once (`socket.destroy()`).
    fn destroy(&self);

    /// Whether the connection has been destroyed.
    fn is_destroyed(&self) -> bool;
}

/// A connected client that exchanges whole text messages, for hosts that decode WebSocket frames themselves.
pub trait Peer {
    /// Send one text message.
    ///
    /// # Arguments
    /// * `text` - Message text.
    fn send_text(&self, text: &str);

    /// Close the connection at once.
    fn destroy(&self);

    /// Whether the connection has been destroyed.
    fn is_destroyed(&self) -> bool;
}

/// The hub's list of connected SignalR clients (`createHub`).
/// Decides whether an upgrade request may connect.
pub type Allows = Rc<dyn Fn(&Request) -> Result<bool>>;

pub struct Hub {
    allows: Allows,
    clients: RefCell<Vec<Rc<Session>>>,
}

/// One WebSocket connection to the hub.
pub struct Session {
    hub: Weak<Hub>,
    head: Request,
    peer: Rc<dyn Peer>,
    pending: RefCell<String>,
    registered: Cell<bool>,
    closed: Cell<bool>,
}

/// A raw socket connection: decodes the client's WebSocket frames into a [`Session`].
pub struct RawConnection {
    session: Rc<Session>,
    socket: Rc<dyn Socket>,
    buffer: RefCell<Vec<u8>>,
}

struct RawPeer(Rc<dyn Socket>);

fn frame(opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 10);
    out.push(0x80 | opcode);
    if payload.len() < 126 {
        out.push(payload.len() as u8);
    } else if payload.len() <= 65535 {
        out.push(126);
        out.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    } else {
        out.push(127);
        out.extend_from_slice(&(payload.len() as u64).to_be_bytes());
    }
    out.extend_from_slice(payload);
    out
}

fn write_frame(socket: &dyn Socket, opcode: u8, payload: &[u8]) {
    if !socket.is_destroyed() {
        socket.write(&frame(opcode, payload));
    }
}

impl Peer for RawPeer {
    fn send_text(&self, text: &str) {
        write_frame(&*self.0, 1, text.as_bytes());
    }

    fn destroy(&self) {
        self.0.destroy();
    }

    fn is_destroyed(&self) -> bool {
        self.0.is_destroyed()
    }
}

/// The `Sec-WebSocket-Accept` value for a client key.
///
/// # Arguments
/// * `key` - The request's `Sec-WebSocket-Key`.
///
/// # Returns
/// Base64 of the SHA-1 of the key and the protocol GUID.
pub fn accept_key(key: &str) -> String {
    base64::encode(&Sha1::digest(format!("{key}{GUID}").as_bytes()))
}

impl Hub {
    /// A hub with no clients.
    ///
    /// # Arguments
    /// * `allows` - Whether an upgrade request may connect; asked again before each event and ping, so a connection
    ///   whose session ended is closed. An error counts as no.
    pub fn new(allows: Allows) -> Rc<Hub> {
        Rc::new(Hub { allows, clients: RefCell::new(Vec::new()) })
    }

    fn permitted(&self, req: &Request) -> bool {
        (self.allows)(req).unwrap_or(false)
    }

    /// Whether an upgrade request is one the hub accepts: path `/hub`, a WebSocket key and a permitted session.
    ///
    /// # Arguments
    /// * `req` - The upgrade request.
    pub fn admits(&self, req: &Request) -> bool {
        req.pathname() == "/hub" && req.header("sec-websocket-key").is_some_and(|key| !key.is_empty()) && self.permitted(req)
    }

    /// Accept an upgraded raw socket (`server.on('upgrade')`).
    ///
    /// # Arguments
    /// * `req` - The upgrade request.
    /// * `socket` - The upgraded connection.
    /// * `head` - Bytes the client sent after the request headers.
    ///
    /// # Returns
    /// The connection to feed with data, or `None` when the socket was refused and destroyed.
    pub fn upgrade(self: &Rc<Self>, req: &Request, socket: Rc<dyn Socket>, head: &[u8]) -> Option<Rc<RawConnection>> {
        if !self.admits(req) {
            socket.destroy();
            return None;
        }
        let accept = accept_key(req.header("sec-websocket-key").unwrap_or_default());
        socket.write(format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {accept}\r\n\r\n").as_bytes());
        let connection = self.attach(req, socket);
        if !head.is_empty() {
            connection.data(head);
        }
        Some(connection)
    }

    /// Accept a raw socket whose `101 Switching Protocols` answer the host's HTTP server already sent, for a request
    /// the hub [`admits`](Hub::admits).
    ///
    /// # Arguments
    /// * `req` - The upgrade request.
    /// * `socket` - The upgraded connection.
    ///
    /// # Returns
    /// The connection to feed with data.
    pub fn attach(self: &Rc<Self>, req: &Request, socket: Rc<dyn Socket>) -> Rc<RawConnection> {
        let session = self.session(req, Rc::new(RawPeer(socket.clone())));
        Rc::new(RawConnection { session, socket, buffer: RefCell::new(Vec::new()) })
    }

    /// Accept a client whose WebSocket frames the host decodes.
    ///
    /// # Arguments
    /// * `req` - The upgrade request.
    /// * `peer` - The connected client.
    ///
    /// # Returns
    /// The session to feed with text messages, or `None` when the client was refused and destroyed.
    pub fn connect(self: &Rc<Self>, req: &Request, peer: Rc<dyn Peer>) -> Option<Rc<Session>> {
        if !self.admits(req) {
            peer.destroy();
            return None;
        }
        Some(self.session(req, peer))
    }

    fn session(self: &Rc<Self>, req: &Request, peer: Rc<dyn Peer>) -> Rc<Session> {
        let session = Rc::new(Session {
            hub: Rc::downgrade(self),
            head: req.head(),
            peer,
            pending: RefCell::new(String::new()),
            registered: Cell::new(false),
            closed: Cell::new(false),
        });
        let weak = Rc::downgrade(&session);
        runtime::spawn(async move {
            loop {
                runtime::sleep(PING_MS).await;
                match weak.upgrade() {
                    Some(session) if !session.closed.get() => session.ping(),
                    _ => break,
                }
            }
        });
        session
    }

    fn remove(&self, session: &Rc<Session>) {
        self.clients.borrow_mut().retain(|client| !Rc::ptr_eq(client, session));
    }

    /// Send one invocation to every connected client that finished the handshake.
    ///
    /// # Arguments
    /// * `target` - Hub method name.
    /// * `payload` - The single argument.
    ///
    /// # Errors
    /// An error when the payload cannot be serialized.
    pub fn broadcast(&self, target: &str, payload: &Value) -> Result<()> {
        let message = format!("{}{RECORD_END}", json::to_string(&obj! { "type" => 1, "target" => target, "arguments" => vec![payload.clone()] }.into()));
        let clients: Vec<Rc<Session>> = self.clients.borrow().clone();
        for session in clients {
            if self.permitted(&session.head) {
                session.send(&message);
            } else {
                self.remove(&session);
                session.peer.destroy();
            }
        }
        Ok(())
    }
}

impl Session {
    fn send(&self, message: &str) {
        if !self.peer.is_destroyed() {
            self.peer.send_text(message);
        }
    }

    /// Send the keep-alive ping, or close the connection when its session ended.
    pub fn ping(self: &Rc<Self>) {
        match self.hub.upgrade() {
            Some(hub) if hub.permitted(&self.head) => self.send("{\"type\":6}\x1e"),
            _ => self.peer.destroy(),
        }
    }

    /// The connection ended; stop pinging and forget the client.
    pub fn closed(self: &Rc<Self>) {
        self.closed.set(true);
        if let Some(hub) = self.hub.upgrade() {
            hub.remove(self);
        }
    }

    /// Feed text received from the client.
    ///
    /// # Arguments
    /// * `chunk` - Text of one WebSocket message or fragment.
    ///
    /// # Returns
    /// Whether the connection is still open; `false` after it was destroyed for a protocol violation.
    pub fn on_text(self: &Rc<Self>, chunk: &str) -> bool {
        self.pending.borrow_mut().push_str(chunk);
        if text::len(&self.pending.borrow()) > LIMIT {
            self.peer.destroy();
            return false;
        }
        loop {
            let record = {
                let mut pending = self.pending.borrow_mut();
                let Some(end) = pending.find(RECORD_END) else { break };
                let record = pending[..end].to_owned();
                pending.drain(..end + RECORD_END.len_utf8());
                record
            };
            let message = match json::parse(&record) {
                Ok(Value::Null) | Err(_) => {
                    self.peer.destroy();
                    return false;
                }
                Ok(message) => message,
            };
            let json_v1 = matches!(message.get("protocol"), Value::String(protocol) if protocol == "json") && matches!(message.get("version"), Value::Number(version) if *version == 1.0);
            if !self.registered.get() && json_v1 {
                self.send("{}\x1e");
                if let Some(hub) = self.hub.upgrade() {
                    self.registered.set(true);
                    hub.clients.borrow_mut().push(self.clone());
                }
            }
        }
        true
    }
}

impl RawConnection {
    /// Feed bytes received from the socket.
    ///
    /// # Arguments
    /// * `chunk` - Bytes in the order they arrived.
    pub fn data(&self, chunk: &[u8]) {
        self.buffer.borrow_mut().extend_from_slice(chunk);
        loop {
            let (opcode, payload, consumed) = {
                let buffer = self.buffer.borrow();
                if buffer.len() < 2 {
                    return;
                }
                let opcode = buffer[0] & 15;
                if buffer[1] & 128 == 0 {
                    drop(buffer);
                    self.socket.destroy();
                    return;
                }
                let mut length = (buffer[1] & 127) as usize;
                let mut offset = 2;
                if length == 126 {
                    if buffer.len() < 4 {
                        return;
                    }
                    length = u16::from_be_bytes([buffer[2], buffer[3]]) as usize;
                    offset = 4;
                } else if length == 127 {
                    if buffer.len() < 10 {
                        return;
                    }
                    let large = u64::from_be_bytes(buffer[2..10].try_into().expect("eight bytes"));
                    if large > LIMIT as u64 {
                        drop(buffer);
                        self.socket.destroy();
                        return;
                    }
                    length = large as usize;
                    offset = 10;
                }
                if length > LIMIT {
                    drop(buffer);
                    self.socket.destroy();
                    return;
                }
                if buffer.len() < offset + 4 + length {
                    return;
                }
                let mask = [buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3]];
                offset += 4;
                let payload: Vec<u8> = buffer[offset..offset + length].iter().enumerate().map(|(index, byte)| byte ^ mask[index % 4]).collect();
                (opcode, payload, offset + length)
            };
            self.buffer.borrow_mut().drain(..consumed);
            match opcode {
                8 => {
                    write_frame(&*self.socket, 8, &payload);
                    self.socket.end();
                    return;
                }
                9 => write_frame(&*self.socket, 10, &payload),
                0 | 1 if !self.session.on_text(&String::from_utf8_lossy(&payload)) => return,
                _ => {}
            }
        }
    }

    /// The socket closed or errored.
    pub fn closed(&self) {
        self.session.closed();
    }

    /// The session this connection feeds.
    pub fn session(&self) -> &Rc<Session> {
        &self.session
    }
}
