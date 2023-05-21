use crossbeam_channel::{unbounded, Receiver, Sender};
use log::{Level, Metadata, Record};

use crate::woodstock::{LogEntry, LogLevel};

pub const CLOSE_LOG_STRING: &str = "close_log";

pub fn close_log() {
    log::info!("{}", CLOSE_LOG_STRING);
}

pub struct GrpcLogger {
    tx: Sender<LogEntry>,
    pub rx: Receiver<LogEntry>,
}

impl Default for GrpcLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl GrpcLogger {
    #[must_use]
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        GrpcLogger { tx, rx }
    }
}

impl log::Log for GrpcLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let entry = if record.args().to_string().eq(CLOSE_LOG_STRING) {
                LogEntry {
                    level: LogLevel::Verbose as i32,
                    context: CLOSE_LOG_STRING.to_string(),
                    line: CLOSE_LOG_STRING.to_string(),
                }
            } else {
                LogEntry {
                    level: match record.metadata().level() {
                        Level::Error => LogLevel::Error as i32,
                        Level::Warn => LogLevel::Warn as i32,
                        Level::Info => LogLevel::Log as i32,
                        Level::Debug => LogLevel::Debug as i32,
                        Level::Trace => LogLevel::Verbose as i32,
                    },
                    context: record.target().to_string(),
                    line: format!("{}", record.args()),
                }
            };

            let send = self.tx.send(entry);
            if send.is_err() {
                println!("Failed to send log entry: {:?}", send.err());
            }
        }
    }

    fn flush(&self) {}
}
