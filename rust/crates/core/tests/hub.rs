//! `lib/hub.mjs` against the Rust port: the bytes each side writes to a fake upgraded socket for the handshake, pushes,
//! pings, closes and malformed frames.

mod common;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use mm_core::http::Request;
use mm_core::hub::{Hub, RawConnection, Socket};
use mm_core::runtime::{self, Runtime, Task};
use mm_core::{Error, Value, obj};
use mm_js::base64;

const DRIVER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/drivers/hub.mjs");

struct Idle;

impl Runtime for Idle {
    fn spawn(&self, _task: Task) {}

    fn sleep(&self, _ms: u64) -> Task {
        Box::pin(std::future::pending())
    }

    fn log(&self, _line: &str) {}
}

static IDLE: Idle = Idle;

type Events = Rc<RefCell<Vec<Value>>>;

struct Fake {
    id: String,
    events: Events,
    destroyed: Cell<bool>,
}

impl Socket for Fake {
    fn write(&self, bytes: &[u8]) {
        self.events.borrow_mut().push(Value::Array(vec![s(&self.id), s("write"), s(&base64::hex(bytes))]));
    }

    fn end(&self) {
        self.events.borrow_mut().push(Value::Array(vec![s(&self.id), s("end")]));
    }

    fn destroy(&self) {
        self.destroyed.set(true);
        self.events.borrow_mut().push(Value::Array(vec![s(&self.id), s("destroy")]));
    }

    fn is_destroyed(&self) -> bool {
        self.destroyed.get()
    }
}

fn s(text: &str) -> Value {
    Value::String(text.to_owned())
}

fn hex(text: &str) -> Vec<u8> {
    (0..text.len() / 2).map(|i| u8::from_str_radix(&text[2 * i..2 * i + 2], 16).unwrap()).collect()
}

fn client_frame(opcode: u8, payload: &[u8], masked: bool) -> Vec<u8> {
    let mut out = vec![0x80 | opcode];
    let mask_bit = if masked { 0x80 } else { 0 };
    if payload.len() < 126 {
        out.push(mask_bit | payload.len() as u8);
    } else if payload.len() <= 65535 {
        out.push(mask_bit | 126);
        out.extend((payload.len() as u16).to_be_bytes());
    } else {
        out.push(mask_bit | 127);
        out.extend((payload.len() as u64).to_be_bytes());
    }
    let mask = [0x12, 0x34, 0x56, 0x78];
    if masked {
        out.extend(mask);
    }
    out.extend(payload.iter().enumerate().map(|(i, byte)| if masked { byte ^ mask[i % 4] } else { *byte }));
    out
}

fn text_frame(text: &str) -> Vec<u8> {
    client_frame(1, text.as_bytes(), true)
}

fn upgrade(id: &str, path: &str, headers: &[(&str, &str)], head: &[u8]) -> Value {
    let mut map = mm_core::Object::new();
    for (name, value) in headers {
        map.insert(*name, *value);
    }
    obj! { "op" => "upgrade", "id" => id, "path" => path, "headers" => map, "head" => base64::hex(head) }.into()
}

fn keyed(id: &str, user: &str) -> Value {
    upgrade(id, "/hub", &[("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ=="), ("x-user", user)], &[])
}

fn data(id: &str, bytes: &[u8]) -> Value {
    obj! { "op" => "data", "id" => id, "hex" => base64::hex(bytes) }.into()
}

fn simple(op: &str, id: &str) -> Value {
    obj! { "op" => op, "id" => id }.into()
}

fn allow(mode: Value) -> Value {
    obj! { "op" => "allow", "mode" => mode }.into()
}

fn broadcast(target: &str, payload: Value) -> Value {
    obj! { "op" => "broadcast", "target" => target, "payload" => payload }.into()
}

const HELLO: &str = "{\"protocol\":\"json\",\"version\":1}\x1e";

fn steps() -> Vec<Value> {
    let long = "é".repeat(40_000);
    let mut steps = vec![
        keyed("a", "u1"),
        keyed("b", "u2"),
        upgrade("c", "/hub", &[("x-user", "u1")], &[]),
        upgrade("d", "/nothub", &[("sec-websocket-key", "k"), ("x-user", "u1")], &[]),
        upgrade("e", "/hub?x=1", &[("sec-websocket-key", "abc"), ("x-user", "u1")], &[]),
        upgrade("f", "/hub", &[("sec-websocket-key", ""), ("x-user", "u1")], &[]),
        upgrade("g", "/hub", &[("sec-websocket-key", "k"), ("x-user", "nobody")], &[]),
        upgrade("h", "/hub", &[("sec-websocket-key", "k"), ("x-user", "u1")], &text_frame(HELLO)),
        broadcast("early", obj! { "n" => 1 }.into()),
        data("a", &text_frame(HELLO)),
        broadcast("pushed", obj! { "id" => "abc", "list" => vec![Value::from(1.0), s("é\u{1F600}\u{2028}")] }.into()),
        data("a", &text_frame(HELLO)),
        broadcast("again", Value::Null),
    ];
    let hello = text_frame(HELLO);
    for chunk in hello.chunks(5) {
        steps.push(data("b", chunk));
    }
    steps.push(broadcast("split", s("x")));
    steps.push(simple("ping", "a"));
    steps.push(simple("ping", "b"));
    steps.push(simple("ping", "d"));
    steps.push(allow(Value::Array(vec![s("u1")])));
    steps.push(simple("ping", "b"));
    steps.push(broadcast("after-denied", s("y")));
    steps.push(allow(s("throw")));
    steps.push(simple("ping", "a"));
    steps.push(allow(s("all")));
    steps.push(broadcast("big", s(&long)));
    steps.push(broadcast("big2", s(&"z".repeat(70_000))));
    steps.push(allow(s("none")));
    steps.push(broadcast("none", s("n")));
    steps.push(allow(s("all")));

    for (id, user) in [("p", "u1"), ("q", "u1"), ("r", "u1"), ("s", "u1"), ("t", "u1"), ("u", "u1"), ("v", "u1"), ("w", "u1"), ("x", "u1"), ("y", "u1"), ("z", "u1")] {
        steps.push(keyed(id, user));
    }
    steps.push(data("p", &client_frame(9, b"hi", true)));
    steps.push(data("p", &client_frame(9, b"", true)));
    steps.push(data("p", &client_frame(10, b"pong", true)));
    steps.push(data("p", &client_frame(2, b"\x00\x01", true)));
    steps.push(data("p", &text_frame("{\"protocol\":\"json\"}\x1e{\"protocol\":\"xml\",\"version\":1}\x1e{\"protocol\":\"json\",\"version\":2}\x1e5\x1e\"str\"\x1e[]\x1e{}\x1e")));
    steps.push(broadcast("p-none", s("none yet")));
    steps.push(data("p", &text_frame(HELLO)));
    steps.push(broadcast("p-yes", s("registered")));
    steps.push(data("q", &client_frame(8, b"\x03\xe8bye", true)));
    steps.push(simple("close", "q"));
    steps.push(broadcast("after-close", s("x")));
    steps.push(data("r", &client_frame(1, HELLO.as_bytes(), false)));
    steps.push(data("r", &text_frame(HELLO)));
    steps.push(data("s", &text_frame("not json\x1e")));
    steps.push(data("s", &text_frame(HELLO)));
    steps.push(data("t", &text_frame("\x1e")));
    steps.push(data("u", &[&client_frame(0, b"{\"protocol\"", true)[..], &client_frame(0, b":\"json\",\"version\":1}\x1e", true)[..]].concat()));
    steps.push(data("u", &{
        let mut first = client_frame(1, b"{\"protocol\"", true);
        first[0] &= 0x7f;
        first
    }));
    steps.push(data("u", &client_frame(0, b":\"json\",\"version\":1}\x1e", true)));
    steps.push(broadcast("u-yes", s("registered")));
    steps.push(data("v", &client_frame(1, &[b'a'; 200], true)));
    steps.push(data("v", &client_frame(1, &vec![b'a'; 70_000], true)));
    steps.push(data("v", &text_frame(HELLO)));
    steps.push(broadcast("v-yes", s("registered")));
    let mut huge = vec![0x81, 0xff];
    huge.extend((1_048_577u64).to_be_bytes());
    steps.push(data("w", &huge));
    steps.push(data("w", &text_frame(HELLO)));
    let mut edge = vec![0x81, 0xff];
    edge.extend((1_048_576u64).to_be_bytes());
    steps.push(data("x", &edge));
    steps.push(data("x", &[0x12, 0x34, 0x56, 0x78]));
    steps.push(data("x", &text_frame(HELLO)));
    steps.push(data("y", &[0x81]));
    steps.push(data("y", &[0xfe, 0x00]));
    let accented = "{\"protocol\":\"json\",\"version\":1,\"x\":\"é\"}\x1e".as_bytes().to_vec();
    steps.push(data("y", &client_frame(1, &accented[..accented.len() - 1], true)));
    steps.push(data("y", &client_frame(0, &accented[accented.len() - 1..], true)));
    steps.push(broadcast("y-yes", s("registered")));
    steps.push(data("z", &client_frame(1, &[0xc3], true)));
    steps.push(data("z", &client_frame(0, &[0xa9, 0x1e], true)));
    steps.push(data("z", &text_frame(HELLO)));
    steps.push(data("z", &client_frame(8, b"", true)));
    steps.push(data("z", &client_frame(9, b"late", true)));
    steps
}

#[test]
fn hub_bytes_match_js() {
    runtime::install(&IDLE);
    let steps = steps();
    let Value::Array(js) = common::call(DRIVER, "run", vec![Value::Array(steps.clone())]).unwrap() else { panic!("no answers") };

    let users: Rc<RefCell<Value>> = Rc::new(RefCell::new(s("all")));
    let mode = users.clone();
    let hub = Hub::new(Rc::new(move |req: &Request| match &*mode.borrow() {
        Value::String(text) if text == "all" => Ok(true),
        Value::String(text) if text == "none" => Ok(false),
        Value::String(text) if text == "throw" => Err(Error::internal("boom")),
        Value::Array(list) => Ok(list.iter().any(|user| Some(user.to_js_string().as_str()) == req.header("x-user"))),
        _ => Ok(false),
    }));
    let events: Events = Rc::default();
    let mut connections: HashMap<String, Rc<RawConnection>> = HashMap::new();
    let mut ended: Vec<String> = Vec::new();
    let mut rust = Vec::new();
    for step in &steps {
        events.borrow_mut().clear();
        let op = step.get("op").to_js_string();
        let id = step.get("id").to_js_string();
        match op.as_str() {
            "upgrade" => {
                let mut request = Request::new("GET", &format!("http://localhost{}", step.get("path").to_js_string())).unwrap();
                if let Value::Object(headers) = step.get("headers") {
                    for (name, value) in headers.iter() {
                        request.headers.set(name.into_owned(), value.to_js_string());
                    }
                }
                let socket = Rc::new(Fake { id: id.clone(), events: events.clone(), destroyed: Cell::new(false) });
                if let Some(connection) = hub.upgrade(&request, socket, &hex(&step.get("head").to_js_string())) {
                    connections.insert(id, connection);
                }
            }
            "data" => {
                if let Some(connection) = connections.get(&id) {
                    connection.data(&hex(&step.get("hex").to_js_string()));
                }
            }
            "ping" => {
                if let Some(connection) = connections.get(&id).filter(|_| !ended.contains(&id)) {
                    connection.session().ping();
                }
            }
            "allow" => *users.borrow_mut() = step.get("mode").clone(),
            "broadcast" => hub.broadcast(&step.get("target").to_js_string(), step.get("payload")).unwrap(),
            "close" => {
                if let Some(connection) = connections.get(&id) {
                    connection.closed();
                    ended.push(id);
                }
            }
            other => panic!("unknown step {other}"),
        }
        rust.push(obj! { "op" => op.as_str(), "events" => events.borrow().clone() }.into());
    }
    assert_eq!(rust.len(), js.len());
    let mut written = 0;
    for (index, (rust, js)) in rust.iter().zip(&js).enumerate() {
        assert_eq!(common::text(rust), common::text(js), "step {index}: {}", common::text(&steps[index]).chars().take(300).collect::<String>());
        if let Value::Array(list) = js.get("events") {
            written += list.len();
        }
    }
    assert!(written > 40, "the scenario must produce traffic");
    println!("hub: {} steps compared, {written} events", steps.len());
}
