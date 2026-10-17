//! An upgraded connection as the hub's raw [`Socket`]: writes queue to a writer task, reads feed the hub's frame
//! decoder, and `destroy` drops the connection at once, as Node's `socket.destroy()` does.

use std::cell::Cell;
use std::rc::Rc;

use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use mm_core::http::Request;
use mm_core::hub::{Hub, Socket};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Notify, mpsc};

enum Write {
    Bytes(Vec<u8>),
    End,
    Stop,
}

struct Upgrade {
    writes: mpsc::UnboundedSender<Write>,
    destroyed: Cell<bool>,
    kill: Rc<Notify>,
}

impl Socket for Upgrade {
    fn write(&self, bytes: &[u8]) {
        if !self.destroyed.get() {
            let _ = self.writes.send(Write::Bytes(bytes.to_vec()));
        }
    }

    fn end(&self) {
        let _ = self.writes.send(Write::End);
    }

    fn destroy(&self) {
        self.destroyed.set(true);
        self.kill.notify_one();
    }

    fn is_destroyed(&self) -> bool {
        self.destroyed.get()
    }
}

/// Run a hub connection on an upgraded connection until either side closes it.
///
/// # Arguments
/// * `hub` - The hub that admitted the request.
/// * `request` - The upgrade request, without its body.
/// * `io` - The upgraded connection; bytes the client sent after the request headers are read first.
pub async fn run(hub: Rc<Hub>, request: Request, io: TokioIo<Upgraded>) {
    let (writes, mut queued) = mpsc::unbounded_channel();
    let kill = Rc::new(Notify::new());
    let socket = Rc::new(Upgrade { writes: writes.clone(), destroyed: Cell::new(false), kill: Rc::clone(&kill) });
    let connection = hub.attach(&request, socket.clone());
    let (mut reader, mut writer) = tokio::io::split(io);
    let reading = async {
        let mut buffer = vec![0; 64 * 1024];
        loop {
            match reader.read(&mut buffer).await {
                Ok(0) | Err(_) => break,
                Ok(read) => connection.data(&buffer[..read]),
            }
        }
        let _ = writes.send(Write::Stop);
    };
    let writing = async {
        while let Some(write) = queued.recv().await {
            match write {
                Write::Bytes(bytes) => {
                    if writer.write_all(&bytes).await.is_err() {
                        break;
                    }
                }
                Write::End => {
                    let _ = writer.shutdown().await;
                }
                Write::Stop => break,
            }
        }
    };
    tokio::select! {
        () = kill.notified() => {}
        _ = async { tokio::join!(reading, writing) } => {}
    }
    socket.destroyed.set(true);
    connection.closed();
}
