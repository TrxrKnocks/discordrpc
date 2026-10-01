//! Discord IPC client.
//!
//! One worker thread owns the socket. Everything is request/response on a
//! single handle, which avoids the Windows problem where a pending read on a
//! synchronous pipe blocks writes from another thread.

use std::io::{self, Read, Write};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::{json, Value};

const OP_HANDSHAKE: u32 = 0;
const OP_FRAME: u32 = 1;
const OP_CLOSE: u32 = 2;
const OP_PING: u32 = 3;
const OP_PONG: u32 = 4;

const RETRY_EVERY: Duration = Duration::from_secs(3);
const MIN_SET_GAP: Duration = Duration::from_secs(4);
const KEEPALIVE: Duration = Duration::from_secs(30);
const MAX_FRAME: usize = 1 << 20;

const READ_TIMEOUT: Duration = Duration::from_secs(5);

trait Pipe: Read + Write + Send {
    /// Blocks until data can be read, or fails with `TimedOut`.
    fn wait_readable(&mut self, timeout: Duration) -> io::Result<()>;
}

#[cfg(windows)]
struct WinPipe(std::fs::File);

#[cfg(windows)]
impl Read for WinPipe {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

#[cfg(windows)]
impl Write for WinPipe {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

#[cfg(windows)]
impl Pipe for WinPipe {
    /// A synchronous read on a named pipe can't be given a timeout, so poll
    /// the pipe for pending bytes instead of blocking in `read`.
    fn wait_readable(&mut self, timeout: Duration) -> io::Result<()> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::Pipes::PeekNamedPipe;

        let deadline = Instant::now() + timeout;
        loop {
            let mut avail: u32 = 0;
            let ok = unsafe {
                PeekNamedPipe(
                    self.0.as_raw_handle() as _,
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null_mut(),
                    &mut avail,
                    std::ptr::null_mut(),
                )
            };
            if ok == 0 {
                return Err(io::Error::last_os_error());
            }
            if avail > 0 {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "Discord did not answer"));
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

#[cfg(unix)]
impl Pipe for std::os::unix::net::UnixStream {
    fn wait_readable(&mut self, _timeout: Duration) -> io::Result<()> {
        // The socket carries a read timeout, set when it was opened.
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub connected: bool,
    pub user: Option<DiscordUser>,
    pub error: Option<String>,
}

enum Cmd {
    Set { client_id: String, activity: Value },
    Clear,
    Shutdown(Sender<()>),
}

#[derive(Clone)]
pub struct Rpc {
    tx: Sender<Cmd>,
}

impl Rpc {
    pub fn spawn(client_id: String, on_status: impl Fn(Status) + Send + 'static) -> Rpc {
        let (tx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("discord-ipc".into())
            .spawn(move || Worker::new(client_id, Box::new(on_status)).run(rx))
            .expect("failed to start ipc thread");
        Rpc { tx }
    }

    pub fn set(&self, client_id: &str, activity: Value) {
        let _ = self.tx.send(Cmd::Set { client_id: client_id.to_owned(), activity });
    }

    pub fn clear(&self) {
        let _ = self.tx.send(Cmd::Clear);
    }

    /// Clears the presence and waits briefly for the worker to finish.
    pub fn shutdown(&self) {
        let (ack_tx, ack_rx) = mpsc::channel();
        if self.tx.send(Cmd::Shutdown(ack_tx)).is_ok() {
            let _ = ack_rx.recv_timeout(Duration::from_secs(2));
        }
    }
}

enum RpcError {
    /// The connection is unusable and has to be reopened.
    Io(String),
    /// Discord rejected the request but the connection is fine.
    Api(String),
}

impl From<io::Error> for RpcError {
    fn from(e: io::Error) -> Self {
        RpcError::Io(e.to_string())
    }
}

#[cfg(windows)]
fn open_pipe(i: u8) -> io::Result<Box<dyn Pipe>> {
    use std::fs::OpenOptions;
    let f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(format!(r"\\.\pipe\discord-ipc-{i}"))?;
    Ok(Box::new(WinPipe(f)))
}

#[cfg(unix)]
fn open_pipe(i: u8) -> io::Result<Box<dyn Pipe>> {
    use std::os::unix::net::UnixStream;
    use std::path::PathBuf;

    let mut dirs: Vec<PathBuf> = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"]
        .iter()
        .filter_map(|v| std::env::var_os(v))
        .map(PathBuf::from)
        .collect();
    dirs.push(PathBuf::from("/tmp"));

    for dir in dirs {
        let candidates = [
            dir.join(format!("discord-ipc-{i}")),
            dir.join(format!("app/com.discordapp.Discord/discord-ipc-{i}")),
            dir.join(format!("snap.discord/discord-ipc-{i}")),
        ];
        for path in candidates {
            if let Ok(s) = UnixStream::connect(&path) {
                s.set_read_timeout(Some(Duration::from_secs(5)))?;
                s.set_write_timeout(Some(Duration::from_secs(5)))?;
                return Ok(Box::new(s));
            }
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "no discord socket"))
}

fn write_frame(pipe: &mut dyn Pipe, op: u32, body: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(body).expect("json value serializes");
    let mut buf = Vec::with_capacity(8 + body.len());
    buf.extend_from_slice(&op.to_le_bytes());
    buf.extend_from_slice(&(body.len() as u32).to_le_bytes());
    buf.extend_from_slice(&body);
    pipe.write_all(&buf)?;
    pipe.flush()
}

fn read_frame(pipe: &mut dyn Pipe) -> io::Result<(u32, Value)> {
    let mut head = [0u8; 8];
    pipe.wait_readable(READ_TIMEOUT)?;
    pipe.read_exact(&mut head)?;
    let op = u32::from_le_bytes(head[0..4].try_into().unwrap());
    let len = u32::from_le_bytes(head[4..8].try_into().unwrap()) as usize;
    if len > MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "oversized frame"));
    }
    let mut body = vec![0u8; len];
    if len > 0 {
        pipe.wait_readable(READ_TIMEOUT)?;
    }
    pipe.read_exact(&mut body)?;
    Ok((op, serde_json::from_slice(&body).unwrap_or(Value::Null)))
}

struct Conn {
    pipe: Box<dyn Pipe>,
    client_id: String,
    user: Option<DiscordUser>,
    nonce: u64,
    last_io: Instant,
}

impl Conn {
    fn open(client_id: &str) -> Result<Conn, String> {
        let mut last_err = String::from("Discord is not running");
        for i in 0..10 {
            let Ok(mut pipe) = open_pipe(i) else { continue };
            let hello = json!({ "v": 1, "client_id": client_id });
            if let Err(e) = write_frame(pipe.as_mut(), OP_HANDSHAKE, &hello) {
                last_err = e.to_string();
                continue;
            }
            match read_frame(pipe.as_mut()) {
                Ok((OP_FRAME, v)) if v["evt"] == "READY" => {
                    let u = &v["data"]["user"];
                    let user = u["id"].as_str().map(|id| DiscordUser {
                        id: id.to_owned(),
                        username: u["username"].as_str().unwrap_or_default().to_owned(),
                        global_name: u["global_name"].as_str().map(str::to_owned),
                        avatar: u["avatar"].as_str().map(str::to_owned),
                    });
                    return Ok(Conn {
                        pipe,
                        client_id: client_id.to_owned(),
                        user,
                        nonce: 0,
                        last_io: Instant::now(),
                    });
                }
                Ok((OP_CLOSE, v)) => {
                    last_err = v["message"].as_str().unwrap_or("Discord closed the connection").to_owned();
                }
                Ok(_) => last_err = "unexpected reply from Discord".into(),
                Err(e) => last_err = e.to_string(),
            }
        }
        Err(last_err)
    }

    /// Reads frames until the reply to our request arrives, answering pings on the way.
    fn read_reply(&mut self) -> Result<Value, RpcError> {
        loop {
            let (op, v) = read_frame(self.pipe.as_mut())?;
            self.last_io = Instant::now();
            match op {
                OP_PING => write_frame(self.pipe.as_mut(), OP_PONG, &v)?,
                OP_CLOSE => {
                    let msg = v["message"].as_str().unwrap_or("connection closed");
                    return Err(RpcError::Io(msg.to_owned()));
                }
                OP_FRAME => return Ok(v),
                _ => {}
            }
        }
    }

    fn set_activity(&mut self, activity: Option<&Value>) -> Result<(), RpcError> {
        self.nonce += 1;
        let mut args = json!({ "pid": std::process::id() });
        if let Some(a) = activity {
            args["activity"] = a.clone();
        }
        let req = json!({ "cmd": "SET_ACTIVITY", "args": args, "nonce": self.nonce.to_string() });
        write_frame(self.pipe.as_mut(), OP_FRAME, &req)?;
        let reply = self.read_reply()?;
        if reply["evt"] == "ERROR" {
            let msg = reply["data"]["message"].as_str().unwrap_or("Discord rejected the activity");
            return Err(RpcError::Api(msg.to_owned()));
        }
        Ok(())
    }

    fn keepalive(&mut self) -> Result<(), RpcError> {
        write_frame(self.pipe.as_mut(), OP_PING, &json!({}))?;
        loop {
            let (op, v) = read_frame(self.pipe.as_mut())?;
            self.last_io = Instant::now();
            match op {
                OP_PONG => return Ok(()),
                OP_PING => write_frame(self.pipe.as_mut(), OP_PONG, &v)?,
                OP_CLOSE => return Err(RpcError::Io("connection closed".into())),
                _ => {}
            }
        }
    }
}

struct Worker {
    client_id: String,
    desired: Option<Value>,
    /// What Discord currently has. `None` means we haven't told it anything yet.
    sent: Option<Option<Value>>,
    conn: Option<Conn>,
    status: Status,
    on_status: Box<dyn Fn(Status) + Send>,
    next_try: Instant,
    last_set: Instant,
    stop: Option<Sender<()>>,
}

impl Worker {
    fn new(client_id: String, on_status: Box<dyn Fn(Status) + Send>) -> Worker {
        let now = Instant::now();
        Worker {
            client_id,
            desired: None,
            sent: None,
            conn: None,
            status: Status::default(),
            on_status,
            next_try: now,
            last_set: now - MIN_SET_GAP,
            stop: None,
        }
    }

    fn publish(&mut self, status: Status) {
        if status != self.status {
            self.status = status.clone();
            (self.on_status)(status);
        }
    }

    fn run(mut self, rx: mpsc::Receiver<Cmd>) {
        // Emit the initial state so the UI doesn't have to guess.
        (self.on_status)(self.status.clone());

        while self.stop.is_none() {
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(cmd) => {
                    self.apply(cmd);
                    while self.stop.is_none() {
                        match rx.try_recv() {
                            Ok(cmd) => self.apply(cmd),
                            Err(_) => break,
                        }
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            if self.stop.is_none() {
                self.tick();
            }
        }

        if let Some(conn) = self.conn.as_mut() {
            let _ = conn.set_activity(None);
        }
        if let Some(ack) = self.stop.take() {
            let _ = ack.send(());
        }
    }

    fn apply(&mut self, cmd: Cmd) {
        match cmd {
            Cmd::Set { client_id, activity } => {
                self.client_id = client_id;
                self.desired = Some(activity);
            }
            Cmd::Clear => self.desired = None,
            Cmd::Shutdown(ack) => self.stop = Some(ack),
        }
    }

    fn tick(&mut self) {
        if self.conn.as_ref().is_some_and(|c| c.client_id != self.client_id) {
            self.conn = None;
            self.sent = None;
        }

        if self.conn.is_none() {
            if Instant::now() < self.next_try {
                return;
            }
            self.next_try = Instant::now() + RETRY_EVERY;
            match Conn::open(&self.client_id) {
                Ok(conn) => {
                    let user = conn.user.clone();
                    self.conn = Some(conn);
                    self.sent = None;
                    self.publish(Status { connected: true, user, error: None });
                }
                Err(e) => {
                    self.publish(Status { connected: false, user: None, error: Some(e) });
                    return;
                }
            }
        }

        let due = self.sent.as_ref() != Some(&self.desired) && self.last_set.elapsed() >= MIN_SET_GAP;
        let Some(conn) = self.conn.as_mut() else { return };

        let result = if due {
            self.last_set = Instant::now();
            conn.set_activity(self.desired.as_ref()).map(|_| true)
        } else if conn.last_io.elapsed() >= KEEPALIVE {
            conn.keepalive().map(|_| false)
        } else {
            return;
        };

        match result {
            Ok(was_set) => {
                if was_set {
                    self.sent = Some(self.desired.clone());
                    let user = self.status.user.clone();
                    self.publish(Status { connected: true, user, error: None });
                }
            }
            Err(RpcError::Api(msg)) => {
                // Don't resend the same rejected payload in a loop.
                self.sent = Some(self.desired.clone());
                let user = self.status.user.clone();
                self.publish(Status { connected: true, user, error: Some(msg) });
            }
            Err(RpcError::Io(msg)) => {
                self.conn = None;
                self.sent = None;
                self.next_try = Instant::now() + RETRY_EVERY;
                self.publish(Status { connected: false, user: None, error: Some(msg) });
            }
        }
    }
}
