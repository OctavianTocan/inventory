//! Retained HTTP check assertions that require direct control of hub authorization.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use mm_core::http::Request;
use mm_core::hub::{Hub, Socket};
use mm_core::obj;
use mm_core::runtime::{self, Runtime, Task};

struct Idle;

impl Runtime for Idle {
    fn spawn(&self, _task: Task) {}

    fn sleep(&self, _ms: u64) -> Task {
        Box::pin(std::future::pending())
    }

    fn log(&self, _line: &str) {}
}

static IDLE: Idle = Idle;

#[derive(Default)]
struct MemorySocket {
    sent: RefCell<Vec<Vec<u8>>>,
    destroyed: Cell<bool>,
}

impl Socket for MemorySocket {
    fn write(&self, bytes: &[u8]) {
        self.sent.borrow_mut().push(bytes.to_vec());
    }

    fn end(&self) {
        self.destroyed.set(true);
    }

    fn destroy(&self) {
        self.destroyed.set(true);
    }

    fn is_destroyed(&self) -> bool {
        self.destroyed.get()
    }
}

// Match a payload without interpreting the production encoder's WebSocket frame layout.
fn contains(socket: &MemorySocket, text: &str) -> bool {
    socket.sent.borrow().iter().any(|bytes| {
        bytes
            .windows(text.len())
            .any(|part| part == text.as_bytes())
    })
}

// Complete the SignalR handshake through a masked client WebSocket frame.
fn handshake() -> Vec<u8> {
    let payload = b"{\"protocol\":\"json\",\"version\":1}\x1e";
    let mut bytes = vec![0x81, 0x80 | payload.len() as u8, 0, 0, 0, 0];
    bytes.extend_from_slice(payload);
    bytes
}

#[test]
fn ended_session_disconnects_before_the_next_hub_event_or_ping() {
    runtime::install(&IDLE);
    for ping in [false, true] {
        let allowed = Rc::new(Cell::new(true));
        let permission = allowed.clone();
        let hub = Hub::new(Rc::new(move |_| Ok(permission.get())));
        let mut request = Request::new("GET", "http://localhost/hub").unwrap();
        request
            .headers
            .set("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==");
        let socket = Rc::new(MemorySocket::default());
        let connection = hub
            .upgrade(&request, socket.clone(), &[])
            .expect("signed-in upgrade");
        connection.data(&handshake());
        hub.broadcast("object:updated", &obj! { "id" => "while-signed-in" }.into())
            .unwrap();
        assert!(
            contains(&socket, "while-signed-in"),
            "signed-in connection receives its event"
        );
        assert!(!socket.is_destroyed());

        allowed.set(false);
        let before = socket.sent.borrow().len();
        if ping {
            connection.session().ping();
        } else {
            hub.broadcast("object:updated", &obj! { "id" => "after-sign-out" }.into())
                .unwrap();
        }
        assert!(socket.is_destroyed(), "ended session closes its connection");
        assert_eq!(
            socket.sent.borrow().len(),
            before,
            "ended session receives no event or ping"
        );
        assert!(
            !contains(&socket, "after-sign-out"),
            "no event after sign-out"
        );
        connection.closed();
    }
}
