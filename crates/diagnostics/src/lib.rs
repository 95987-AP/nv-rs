//! Opt-in evidence capture. No gameplay rules, external libraries or disk I/O on producers.
use std::cell::Cell;
use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub enum Value {
    Str(String),
    U64(u64),
    F64(f64),
    Bool(bool),
}
impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::Str(s.into())
    }
}
impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::Str(s)
    }
}
pub fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => {
                use std::fmt::Write;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
pub fn object(fields: &[(&str, Value)]) -> String {
    let mut out = String::from("{");
    for (i, (key, value)) in fields.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&quote(key));
        out.push(':');
        out.push_str(&match value {
            Value::Str(s) => quote(s),
            Value::U64(v) => v.to_string(),
            Value::F64(v) if v.is_finite() => v.to_string(),
            Value::F64(_) => "null".into(),
            Value::Bool(v) => v.to_string(),
        });
    }
    out.push('}');
    out
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub queue_events: usize,
    pub queue_bytes: usize,
    pub file_bytes: u64,
    pub session_bytes: u64,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            queue_events: 8192,
            queue_bytes: 16 * 1024 * 1024,
            file_bytes: 16 * 1024 * 1024,
            session_bytes: 512 * 1024 * 1024,
        }
    }
}
#[derive(Debug)]
struct Encoded {
    line: String,
    configuration: Option<Vec<(String, Value)>>,
}
#[derive(Debug)]
struct Manifest {
    prefix: String,
    suffix: String,
    configuration: Vec<(String, Value)>,
}
#[derive(Default, Debug)]
struct Queue {
    lines: VecDeque<Encoded>,
    bytes: usize,
    sequence: u64,
    closing: bool,
}
#[derive(Debug)]
struct Shared {
    queue: Mutex<Queue>,
    wake: Condvar,
    start: Instant,
    next: AtomicU64,
    dropped: AtomicU64,
    written: AtomicU64,
    active: AtomicBool,
    limits: Limits,
    path: PathBuf,
    manifest: Manifest,
}
#[derive(Clone, Default, Debug)]
pub struct Observer(Option<Arc<Shared>>);
static CURRENT: OnceLock<Observer> = OnceLock::new();
thread_local! { static PARENT: Cell<u64> = const { Cell::new(0) }; }
pub struct ContextGuard(u64);
impl Drop for ContextGuard {
    fn drop(&mut self) {
        PARENT.with(|v| v.set(self.0));
    }
}
impl Observer {
    pub fn current() -> Self {
        CURRENT.get().cloned().unwrap_or_default()
    }
    pub fn install(&self) {
        let _ = CURRENT.set(self.clone());
    }
    pub fn enabled(&self) -> bool {
        self.0
            .as_ref()
            .is_some_and(|s| s.active.load(Ordering::Relaxed))
    }
    pub fn id(&self) -> u64 {
        self.0
            .as_ref()
            .map_or(0, |s| s.next.fetch_add(1, Ordering::Relaxed))
    }
    pub fn parent(&self) -> u64 {
        PARENT.with(Cell::get)
    }
    pub fn context(&self, id: u64) -> ContextGuard {
        ContextGuard(PARENT.with(|v| v.replace(id)))
    }
    pub fn time_us(&self) -> u64 {
        self.0
            .as_ref()
            .map_or(0, |s| s.start.elapsed().as_micros() as u64)
    }
    pub fn dropped(&self) -> u64 {
        self.0
            .as_ref()
            .map_or(0, |s| s.dropped.load(Ordering::Relaxed))
    }
    pub fn path(&self) -> Option<&Path> {
        self.0.as_ref().map(|s| s.path.as_path())
    }
    pub fn emit(&self, kind: &str, id: u64, parent_id: u64, fields: &[(&str, Value)]) {
        let Some(s) = self.0.as_ref().filter(|s| s.active.load(Ordering::Relaxed)) else {
            return;
        };
        let payload = object(fields);
        let Ok(mut q) = s.queue.try_lock() else {
            s.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        };
        if q.closing {
            return;
        }
        q.sequence += 1;
        let line = format!("{{\"version\":1,\"sequence\":{},\"time_us\":{},\"thread\":{},\"type\":{},\"id\":{id},\"parent_id\":{parent_id},\"fields\":{payload}}}\n",
            q.sequence, s.start.elapsed().as_micros(), quote(&format!("{:?}", thread::current().id())), quote(kind));
        let queued_bytes = if kind == "configuration" {
            line.len().saturating_mul(2)
        } else {
            line.len()
        };
        if q.lines.len() >= s.limits.queue_events
            || q.bytes.saturating_add(queued_bytes) > s.limits.queue_bytes
        {
            s.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        }
        q.bytes += queued_bytes;
        let configuration = (kind == "configuration").then(|| {
            fields
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect()
        });
        q.lines.push_back(Encoded {
            line,
            configuration,
        });
        s.wake.notify_one();
    }
    pub fn span(&self, name: &str, id: u64, parent_id: u64) -> Span {
        let start = self.enabled().then(Instant::now);
        if start.is_some() {
            self.emit("span_start", id, parent_id, &[("name", name.into())]);
        }
        Span {
            observer: self.clone(),
            name: start.map(|_| name.to_owned()),
            start,
            id,
            parent_id,
        }
    }
}
pub struct Span {
    observer: Observer,
    name: Option<String>,
    start: Option<Instant>,
    id: u64,
    parent_id: u64,
}
impl Drop for Span {
    fn drop(&mut self) {
        if let (Some(name), Some(start)) = (&self.name, self.start) {
            self.observer.emit(
                "span_end",
                self.id,
                self.parent_id,
                &[
                    ("name", name.clone().into()),
                    (
                        "duration_us",
                        Value::U64(start.elapsed().as_micros() as u64),
                    ),
                ],
            );
        }
    }
}
pub struct Recorder {
    observer: Observer,
    worker: Option<JoinHandle<()>>,
}
impl Recorder {
    pub fn start(root: &Path, configuration: &[(&str, Value)]) -> io::Result<Self> {
        Self::with_limits(root, configuration, Limits::default())
    }
    pub fn with_limits(
        root: &Path,
        configuration: &[(&str, Value)],
        limits: Limits,
    ) -> io::Result<Self> {
        if limits.file_bytes == 0 || limits.session_bytes == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "zero capture limit",
            ));
        }
        fs::create_dir_all(root)?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros();
        let mut suffix = 0;
        let (session, path) = loop {
            let session = format!("{stamp}-{}-{suffix}", std::process::id());
            let path = root.join(&session);
            match fs::create_dir(&path) {
                Ok(()) => break (session, path),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => suffix += 1,
                Err(e) => return Err(e),
            }
        };
        fs::write(path.join("manifest.json"), format!("{{\"version\":1,\"session_id\":{},\"configuration\":{},\"started_unix_us\":{stamp},\"limits\":{{\"queue_events\":{},\"queue_bytes\":{},\"file_bytes\":{},\"session_bytes\":{}}}}}\n", quote(&session), object(configuration), limits.queue_events,limits.queue_bytes,limits.file_bytes,limits.session_bytes))?;
        let manifest = Manifest { prefix: format!("{{\"version\":1,\"session_id\":{},\"started_unix_us\":{stamp},\"limits\":{{\"queue_events\":{},\"queue_bytes\":{},\"file_bytes\":{},\"session_bytes\":{}}},\"configuration\":",quote(&session),limits.queue_events,limits.queue_bytes,limits.file_bytes,limits.session_bytes), suffix: "}\n".into(), configuration: configuration.iter().map(|(k,v)|(k.to_string(),v.clone())).collect() };
        let initial = File::create(path.join("events-0000.jsonl"))?;
        let shared = Arc::new(Shared {
            queue: Mutex::new(Queue::default()),
            wake: Condvar::new(),
            start: Instant::now(),
            next: AtomicU64::new(1),
            dropped: AtomicU64::new(0),
            written: AtomicU64::new(0),
            active: AtomicBool::new(true),
            limits,
            path,
            manifest,
        });
        let writer = shared.clone();
        let worker = thread::Builder::new()
            .name("nv-diagnostics".into())
            .spawn(move || writer_loop(writer, initial))?;
        Ok(Self {
            observer: Observer(Some(shared)),
            worker: Some(worker),
        })
    }
    pub fn observer(&self) -> Observer {
        self.observer.clone()
    }
}
impl Drop for Recorder {
    fn drop(&mut self) {
        if let Some(s) = &self.observer.0 {
            if let Ok(mut q) = s.queue.lock() {
                q.closing = true;
                s.wake.notify_one();
            }
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
fn status(s: &Shared, complete: bool, reason: &str) -> io::Result<()> {
    // Atomic replacement leaves either the previous readable snapshot or this one.
    let temp = s.path.join("status.tmp");
    fs::write(
        &temp,
        object(&[
            ("complete", Value::Bool(complete)),
            ("reason", reason.into()),
            (
                "dropped_events",
                Value::U64(s.dropped.load(Ordering::Relaxed)),
            ),
            (
                "recorded_bytes",
                Value::U64(s.written.load(Ordering::Relaxed)),
            ),
            (
                "duration_us",
                Value::U64(s.start.elapsed().as_micros() as u64),
            ),
        ]),
    )?;
    fs::rename(temp, s.path.join("status.json"))
}
fn writer_loop(s: Arc<Shared>, initial: File) {
    let mut out = BufWriter::new(initial);
    let mut index = 0;
    let mut file_bytes = 0;
    let mut configuration: std::collections::BTreeMap<String, Value> =
        s.manifest.configuration.iter().cloned().collect();
    let mut last_flush = Instant::now();
    let mut reason = "shutdown";
    let result: io::Result<()> = (|| {
        loop {
            let (lines, closing) = {
                let mut q = s
                    .queue
                    .lock()
                    .map_err(|_| io::Error::other("capture queue poisoned"))?;
                if q.lines.is_empty() && !q.closing {
                    q = s
                        .wake
                        .wait_timeout(q, Duration::from_secs(1))
                        .map_err(|_| io::Error::other("capture queue poisoned"))?
                        .0;
                }
                let lines = std::mem::take(&mut q.lines);
                q.bytes = 0;
                (lines, q.closing)
            };
            let mut lines = lines.into_iter();
            while let Some(encoded) = lines.next() {
                let line = encoded.line;
                if let Some(fields) = encoded.configuration {
                    configuration.extend(fields);
                }
                let size = line.len() as u64;
                if s.written.load(Ordering::Relaxed) + size > s.limits.session_bytes {
                    s.dropped
                        .fetch_add(1 + lines.count() as u64, Ordering::Relaxed);
                    s.active.store(false, Ordering::Relaxed);
                    reason = "session_limit";
                    eprintln!("Diagnostics session limit reached: recording stopped.");
                    break;
                }
                if file_bytes > 0 && file_bytes + size > s.limits.file_bytes {
                    out.flush()?;
                    index += 1;
                    out = BufWriter::new(File::create(
                        s.path.join(format!("events-{index:04}.jsonl")),
                    )?);
                    file_bytes = 0;
                }
                if size > s.limits.file_bytes {
                    s.dropped.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
                out.write_all(line.as_bytes())?;
                file_bytes += size;
                s.written.fetch_add(size, Ordering::Relaxed);
            }
            if closing || !s.active.load(Ordering::Relaxed) {
                break;
            }
            if last_flush.elapsed() >= Duration::from_secs(1) {
                out.flush()?;
                status(&s, false, "recording")?;
                write_manifest(&s, &configuration)?;
                last_flush = Instant::now();
            }
        }
        out.flush()?;
        write_manifest(&s, &configuration)?;
        Ok(())
    })();
    s.active.store(false, Ordering::Relaxed);
    if let Err(e) = result {
        reason = "disk_error";
        eprintln!("Diagnostics disabled after disk error: {e}");
    }
    if let Ok(mut q) = s.queue.lock() {
        s.dropped.fetch_add(q.lines.len() as u64, Ordering::Relaxed);
        q.lines.clear();
        q.bytes = 0;
    }
    let _ = status(&s, reason == "shutdown", reason);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "nvdiag-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
    #[test]
    fn escaping() {
        assert_eq!(quote("a\"\\\n\u{0}😀"), "\"a\\\"\\\\\\n\\u0000😀\"");
        assert_eq!(object(&[("x", Value::F64(f64::NAN))]), "{\"x\":null}");
    }
    #[test]
    fn disabled() {
        let o = Observer::default();
        assert!(!o.enabled());
        assert_eq!(o.id(), 0);
        o.emit("ignored", 0, 0, &[]);
        assert!(o.path().is_none());
    }
    #[test]
    fn correlation_and_shutdown() {
        let r = Recorder::start(&root(), &[]).unwrap();
        let o = r.observer();
        let parent = o.id();
        let c = o.clone();
        thread::spawn(move || {
            let _ctx = c.context(parent);
            c.emit("worker", c.id(), c.parent(), &[("path", "x\\y".into())]);
        })
        .join()
        .unwrap();
        let path = o.path().unwrap().to_owned();
        drop(r);
        let text = fs::read_to_string(path.join("events-0000.jsonl")).unwrap();
        assert!(text.contains(&format!("\"parent_id\":{parent}")));
        assert!(text.ends_with('\n'));
        assert!(fs::read_to_string(path.join("status.json"))
            .unwrap()
            .contains("\"complete\":true"));
    }
    #[test]
    fn saturation_is_nonblocking() {
        let r = Recorder::with_limits(
            &root(),
            &[],
            Limits {
                queue_events: 0,
                ..Limits::default()
            },
        )
        .unwrap();
        let o = r.observer();
        o.emit("drop", 0, 0, &[]);
        assert_eq!(o.dropped(), 1);
    }
    #[test]
    fn rotation_and_limit() {
        let r = Recorder::with_limits(
            &root(),
            &[],
            Limits {
                file_bytes: 250,
                session_bytes: 500,
                ..Limits::default()
            },
        )
        .unwrap();
        let o = r.observer();
        for _ in 0..100 {
            o.emit("test", o.id(), 0, &[]);
        }
        let path = o.path().unwrap().to_owned();
        drop(r);
        let total: u64 = fs::read_dir(&path)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().ends_with("jsonl"))
            .map(|e| e.metadata().unwrap().len())
            .sum();
        assert!(total <= 500);
        assert!(path.join("events-0001.jsonl").exists());
        assert!(fs::read_to_string(path.join("status.json"))
            .unwrap()
            .contains("session_limit"));
    }
    #[test]
    fn disk_failure_continues() {
        let r = Recorder::start(&root(), &[]).unwrap();
        let o = r.observer();
        fs::create_dir(o.path().unwrap().join("status.tmp")).unwrap();
        o.emit("first", 1, 0, &[]);
        thread::sleep(Duration::from_millis(1200));
        assert!(!o.enabled());
        drop(r);
    }
}

fn write_manifest(
    s: &Shared,
    configuration: &std::collections::BTreeMap<String, Value>,
) -> io::Result<()> {
    let fields: Vec<(&str, Value)> = configuration
        .iter()
        .map(|(k, v)| (k.as_str(), v.clone()))
        .collect();
    let temp = s.path.join("manifest.tmp");
    fs::write(
        &temp,
        format!(
            "{}{}{}",
            s.manifest.prefix,
            object(&fields),
            s.manifest.suffix
        ),
    )?;
    fs::rename(temp, s.path.join("manifest.json"))
}
