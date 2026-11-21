//! Opt-in performance telemetry. Only code-defined operation names and numeric
//! measurements are accepted: no payloads, paths, input, URLs, keys or errors.
use serde_json::{json, Map, Value};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, OnceLock,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const MAX_BYTES: u64 = 16 * 1024 * 1024;
const BUFFER: usize = 64 * 1024;
static ACTIVE: AtomicBool = AtomicBool::new(false);
static LOGGER: OnceLock<Mutex<Option<Recorder>>> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ok,
    Error,
    Cancelled,
    Interrupted,
}
impl Outcome {
    fn name(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
            Self::Cancelled => "cancelled",
            Self::Interrupted => "interrupted",
        }
    }
}
pub fn enabled() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}
fn state() -> &'static Mutex<Option<Recorder>> {
    LOGGER.get_or_init(|| Mutex::new(None))
}
fn micros(duration: Duration) -> u64 {
    duration.as_micros().min(u64::MAX as u128) as u64
}
fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or(0)
}
fn clock_resolution_us() -> Option<u64> {
    // Integer microseconds describe the output unit, not the resolution of
    // the user's emulated monotonic clock. Leave unknown native resolution null.
    if cfg!(target_os = "wasi") {
        Some(1000)
    } else {
        None
    }
}

fn linear_memory_bytes() -> Option<u64> {
    #[cfg(target_arch = "wasm32")]
    {
        Some(core::arch::wasm32::memory_size::<0>() as u64 * 65536)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}
fn with_recorder(action: impl FnOnce(&mut Recorder)) {
    if !enabled() {
        return;
    }
    if let Ok(mut guard) = state().lock() {
        if let Some(recorder) = guard.as_mut() {
            action(recorder);
        }
    }
}

#[derive(Default)]
struct Stats {
    calls: u64,
    bytes: u64,
    total_us: u64,
    max_us: u64,
    errors: u64,
    idle: u64,
    buckets: [u64; 32],
}
impl Stats {
    fn add(&mut self, duration: Duration, bytes: u64, error: bool, idle: bool) {
        let us = micros(duration);
        self.calls += 1;
        self.bytes = self.bytes.saturating_add(bytes);
        self.total_us = self.total_us.saturating_add(us);
        self.max_us = self.max_us.max(us);
        self.errors += u64::from(error);
        self.idle += u64::from(idle);
        let bucket = if us <= 1 {
            0
        } else {
            (64 - (us - 1).leading_zeros()).min(31) as usize
        };
        self.buckets[bucket] += 1;
    }
    fn percentile_upper_us(&self, percent: u64) -> u64 {
        let target = self.calls.saturating_mul(percent).div_ceil(100);
        let mut count = 0;
        for (i, &n) in self.buckets.iter().enumerate() {
            count += n;
            if count >= target {
                let bound = if i == 31 {
                    (1u64 << i).max(self.max_us)
                } else {
                    1u64 << i
                };
                return bound.max(clock_resolution_us().unwrap_or(1));
            }
        }
        self.max_us
    }
    fn json(&self) -> Value {
        json!({"calls":self.calls,"bytes":self.bytes,"total_us":self.total_us,"max_us":self.max_us,
            "p50_upper_us":self.percentile_upper_us(50),"p95_upper_us":self.percentile_upper_us(95),
            "errors":self.errors,"idle_timeouts":self.idle})
    }
}

struct Recorder {
    writer: BufWriter<File>,
    started: Instant,
    last_snapshot: Instant,
    seq: u64,
    span_id: u64,
    written: u64,
    write_us: u64,
    dropped: u64,
    failure: Option<io::Error>,
    stats: BTreeMap<&'static str, Stats>,
    limit: u64,
}
impl Recorder {
    fn new(file: File) -> Self {
        let now = Instant::now();
        Self {
            writer: BufWriter::with_capacity(BUFFER, file),
            started: now,
            last_snapshot: now,
            seq: 0,
            span_id: 0,
            written: 0,
            write_us: 0,
            dropped: 0,
            failure: None,
            stats: BTreeMap::new(),
            limit: MAX_BYTES,
        }
    }
    fn write(&mut self, mut record: Value, reserved: bool) {
        if self.failure.is_some() {
            return;
        }
        let started = Instant::now();
        self.seq += 1;
        record["schema"] = 1.into();
        record["seq"] = self.seq.into();
        record["time_unix_ms"] = unix_ms().into();
        record["elapsed_us"] = micros(self.started.elapsed()).into();
        record["linear_memory_bytes"] = linear_memory_bytes().into();
        let result = (|| {
            let mut line = serde_json::to_vec(&record).map_err(io::Error::other)?;
            line.push(b'\n');
            if !reserved && self.written.saturating_add(line.len() as u64) > self.limit {
                self.dropped += 1;
                return Ok(());
            }
            self.writer.write_all(&line)?;
            self.written += line.len() as u64;
            Ok(())
        })();
        if let Err(error) = result {
            self.failure = Some(error);
        }
        self.write_us = self.write_us.saturating_add(micros(started.elapsed()));
    }
    fn flush(&mut self) {
        if self.failure.is_none() {
            let started = Instant::now();
            if let Err(error) = self.writer.flush() {
                self.failure = Some(error);
            }
            self.write_us = self.write_us.saturating_add(micros(started.elapsed()));
        }
    }
    fn stats_json(&self) -> Value {
        Value::Object(
            self.stats
                .iter()
                .map(|(&name, stats)| (name.into(), stats.json()))
                .collect(),
        )
    }
    fn snapshot_if_due(&mut self) {
        if self.last_snapshot.elapsed() >= Duration::from_secs(1) {
            self.write(
                json!({"event":"counters","metrics":self.stats_json()}),
                false,
            );
            self.flush();
            self.last_snapshot = Instant::now();
        }
    }
    fn measurement(
        &mut self,
        operation: &'static str,
        duration: Duration,
        bytes: u64,
        error: bool,
        idle: bool,
    ) {
        self.stats
            .entry(operation)
            .or_default()
            .add(duration, bytes, error, idle);
        self.snapshot_if_due();
    }
    fn finish(mut self, outcome: Outcome) -> io::Result<()> {
        self.write(json!({"event":"summary","outcome":outcome.name(),"metrics":self.stats_json(),
            "log_bytes_before_summary":self.written,"log_write_us":self.write_us,"dropped_events":self.dropped}), true);
        self.flush();
        if let Some(error) = self.failure {
            return Err(error);
        }
        self.writer.get_ref().sync_all()
    }
}

/// One owner finalizes the log; early returns produce an interrupted summary.
pub struct Session {
    path: PathBuf,
    finished: bool,
}
impl Session {
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn finish(mut self, outcome: Outcome) -> io::Result<()> {
        self.finished = true;
        ACTIVE.store(false, Ordering::Relaxed);
        let recorder = state()
            .lock()
            .map_err(|_| io::Error::other("performance logger lock failed"))?
            .take();
        match recorder {
            Some(recorder) => recorder.finish(outcome),
            None => Ok(()),
        }
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        if !self.finished {
            ACTIVE.store(false, Ordering::Relaxed);
            if let Ok(mut state) = state().lock() {
                if let Some(recorder) = state.take() {
                    let _ = recorder.finish(Outcome::Interrupted);
                }
            }
        }
    }
}

/// Terminal/page stop handlers may exit immediately. Write a safe summary
/// before process::exit, which intentionally does not run Session's Drop.
pub fn cancel_active() {
    ACTIVE.store(false, Ordering::Relaxed);
    if let Ok(mut state) = state().lock() {
        if let Some(recorder) = state.take() {
            let _ = recorder.finish(Outcome::Cancelled);
        }
    }
}

pub fn start(
    directory: &Path,
    version: &'static str,
    command: &'static str,
) -> io::Result<Session> {
    let mut guard = state()
        .lock()
        .map_err(|_| io::Error::other("performance logger lock failed"))?;
    if guard.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "performance log already active",
        ));
    }
    let stamp = unix_ms();
    for suffix in 0..1000 {
        let path = directory.join(format!("siboot-perf-{stamp}-{suffix:03}.jsonl"));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = match options.open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        let mut recorder = Recorder::new(file);
        recorder.write(json!({"event":"run_start","version":version,"command":command,"target_os":std::env::consts::OS,
            "time_kind":"wall_elapsed","timestamp_unit_us":1,"clock_resolution_us":clock_resolution_us(),"log_limit_bytes":MAX_BYTES}), false);
        recorder.flush();
        if let Some(error) = recorder.failure.take() {
            return Err(error);
        }
        *guard = Some(recorder);
        ACTIVE.store(true, Ordering::Relaxed);
        return Ok(Session {
            path,
            finished: false,
        });
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "no unused performance log filename",
    ))
}

fn fields(values: &[(&'static str, u64)]) -> Value {
    Value::Object(
        values
            .iter()
            .map(|(name, value)| ((*name).into(), (*value).into()))
            .collect::<Map<_, _>>(),
    )
}
pub fn event(name: &'static str, values: &[(&'static str, u64)]) {
    with_recorder(|recorder| recorder.write(json!({"event":name,"data":fields(values)}), false));
}
pub fn checkpoint() {
    with_recorder(Recorder::flush);
}

pub struct Span {
    id: u64,
    operation: &'static str,
    started: Option<Instant>,
    finished: bool,
}
pub fn span(operation: &'static str, values: &[(&'static str, u64)]) -> Span {
    let mut result = Span {
        id: 0,
        operation,
        started: None,
        finished: false,
    };
    with_recorder(|recorder| {
        recorder.span_id += 1;
        result.id = recorder.span_id;
        recorder.write(json!({"event":"begin","span_id":result.id,"operation":operation,"data":fields(values)}), false);
        result.started = Some(Instant::now());
    });
    result
}
impl Span {
    pub fn finish(mut self, outcome: Outcome, values: &[(&'static str, u64)]) {
        self.end(outcome, values);
    }
    fn end(&mut self, outcome: Outcome, values: &[(&'static str, u64)]) {
        if self.finished {
            return;
        }
        self.finished = true;
        if let Some(started) = self.started {
            let duration = started.elapsed();
            with_recorder(|recorder| {
                recorder.measurement(self.operation, duration, 0, outcome != Outcome::Ok, false);
                recorder.write(
                    json!({"event":"end","span_id":self.id,"operation":self.operation,
                    "duration_us":micros(duration),"outcome":outcome.name(),"data":fields(values)}),
                    false,
                );
            });
        }
    }
}
impl Drop for Span {
    fn drop(&mut self) {
        self.end(Outcome::Interrupted, &[]);
    }
}

pub fn io_bytes(
    operation: &'static str,
    call: impl FnOnce() -> io::Result<usize>,
) -> io::Result<usize> {
    io_bytes_with_idle(operation, false, call)
}

pub fn io_bytes_with_idle(
    operation: &'static str,
    expected_idle: bool,
    call: impl FnOnce() -> io::Result<usize>,
) -> io::Result<usize> {
    measured_io(operation, expected_idle, |size| *size as u64, call)
}

pub fn io_result<T>(
    operation: &'static str,
    size: impl FnOnce(&T) -> u64,
    call: impl FnOnce() -> io::Result<T>,
) -> io::Result<T> {
    measured_io(operation, false, size, call)
}

fn measured_io<T>(
    operation: &'static str,
    expected_idle: bool,
    size: impl FnOnce(&T) -> u64,
    call: impl FnOnce() -> io::Result<T>,
) -> io::Result<T> {
    if !enabled() {
        return call();
    }
    let started = Instant::now();
    let result = call();
    let duration = started.elapsed();
    let idle = expected_idle
        && result.as_ref().err().is_some_and(|error| {
            matches!(
                error.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::Interrupted
            )
        });
    let bytes = result.as_ref().map(size).unwrap_or(0);
    with_recorder(|recorder| {
        recorder.measurement(operation, duration, bytes, result.is_err() && !idle, idle)
    });
    result
}

pub fn work<T>(operation: &'static str, bytes: u64, call: impl FnOnce() -> T) -> T {
    if !enabled() {
        return call();
    }
    let started = Instant::now();
    let result = call();
    let duration = started.elapsed();
    with_recorder(|recorder| recorder.measurement(operation, duration, bytes, false, false));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary() -> PathBuf {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        std::env::temp_dir().join(format!(
            "siboot-perf-test-{}-{}-{}",
            std::process::id(),
            unix_ms(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn counters_distinguish_idle_polls_from_errors_and_bucket_latency() {
        let mut stats = Stats::default();
        stats.add(Duration::from_micros(1000), 400, false, false);
        stats.add(Duration::from_micros(8000), 800, true, false);
        stats.add(Duration::from_micros(2000), 0, false, true);
        let data = stats.json();
        assert_eq!(data["calls"], 3);
        assert_eq!(data["bytes"], 1200);
        assert_eq!(data["total_us"], 11000);
        assert_eq!(data["max_us"], 8000);
        assert_eq!(data["idle_timeouts"], 1);
        assert_eq!(data["errors"], 1);
        assert_eq!(data["p50_upper_us"], 2048);
        assert_eq!(data["p95_upper_us"], 8192);
    }

    #[test]
    fn bounded_logs_keep_a_parseable_summary_after_the_detail_limit() {
        let path = temporary();
        let mut recorder = Recorder::new(File::create(&path).unwrap());
        recorder.limit = 256;
        for _ in 0..100 {
            recorder.write(json!({"event":"test"}), false);
        }
        recorder.measurement("socket.send", Duration::from_millis(5), 1234, false, false);
        recorder.finish(Outcome::Error).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<Value> = text
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let summary = lines.last().unwrap();
        assert_eq!(summary["event"], "summary");
        assert_eq!(summary["outcome"], "error");
        assert!(summary["dropped_events"].as_u64().unwrap() > 0);
        assert_eq!(summary["metrics"]["socket.send"]["bytes"], 1234);
        assert!(text.len() < 4096);
        assert!(lines
            .windows(2)
            .all(|pair| pair[0]["seq"].as_u64() < pair[1]["seq"].as_u64()));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn periodic_snapshots_capture_progress_before_finalization() {
        let path = temporary();
        let mut recorder = Recorder::new(File::create(&path).unwrap());
        recorder.last_snapshot = Instant::now() - Duration::from_secs(2);
        recorder.measurement("socket.recv", Duration::from_millis(2), 9, false, true);
        let text = std::fs::read_to_string(&path).unwrap();
        let snapshot: Value = serde_json::from_str(text.trim()).unwrap();
        assert_eq!(snapshot["event"], "counters");
        assert_eq!(snapshot["metrics"]["socket.recv"]["idle_timeouts"], 1);
        recorder.finish(Outcome::Ok).unwrap();
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn logging_write_errors_are_reported_when_finishing() {
        let path = temporary();
        std::fs::write(&path, b"").unwrap();
        let mut recorder = Recorder::new(File::open(&path).unwrap());
        recorder.write(json!({"event":"test"}), false);
        assert!(recorder.finish(Outcome::Ok).is_err());
        std::fs::remove_file(path).unwrap();
    }
}
