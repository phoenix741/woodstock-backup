//! Layer tracing qui écrit les logs de chaque job dans des fichiers dédiés.
//!
//! Pour les backups et restores, les fichiers sont créés dans le répertoire du backup :
//!   - `<hosts_path>/<host>/<backup_id>/backup.log`       (tous les niveaux)
//!   - `<hosts_path>/<host>/<backup_id>/backup-error.log` (WARN et ERROR uniquement)
//!   - `<hosts_path>/<host>/<backup_id>/restore.log`
//!   - `<hosts_path>/<host>/<backup_id>/restore-error.log`
//!
//! Pour remove, refcnt, fsck : dans `jobs_path` :
//!   - `<jobs_path>/<job_type>-<task_id>.log`
//!   - `<jobs_path>/<job_type>-<task_id>-error.log`
//!
//! Le layer est activé par les champs `job_type`, `host`, `backup_id` et `task_id`
//! déclarés dans les macros `#[instrument(fields(...))]` des handlers.

use chrono::Utc;
use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

// ---------------------------------------------------------------------------
// Extracteur de champs de span
// ---------------------------------------------------------------------------

/// Visite les attributs d'un span et extrait les champs job_type / host /
/// backup_id / task_id positionnés par `#[instrument(fields(...))]`.
#[derive(Default)]
struct SpanFieldExtractor {
    job_type: Option<String>,
    host: Option<String>,
    backup_id: Option<String>,
    task_id: Option<String>,
}

impl SpanFieldExtractor {
    fn accept(&mut self, name: &str, value: &str) {
        match name {
            "job_type" => self.job_type = Some(value.to_string()),
            "host" => self.host = Some(value.to_string()),
            "backup_id" => self.backup_id = Some(value.to_string()),
            "task_id" => self.task_id = Some(value.to_string()),
            _ => {}
        }
    }
}

impl Visit for SpanFieldExtractor {
    // Champs string littéraux : job_type="backup"
    fn record_str(&mut self, field: &Field, value: &str) {
        self.accept(field.name(), value);
    }
    // Champs Display (%host, %backup_number, %task_id) → Debug délègue à Display
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.accept(field.name(), &format!("{value:?}"));
    }
}

// ---------------------------------------------------------------------------
// Extracteur du message d'un Event
// ---------------------------------------------------------------------------

struct EventMessageExtractor(String);

impl Visit for EventMessageExtractor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.0 = value.to_string();
        }
    }
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }
}

/// Formate un événement tracing en ligne de log lisible.
fn format_event(event: &Event<'_>) -> String {
    let meta = event.metadata();
    let ts = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
    let level = meta.level();
    let target = meta.target();

    let mut v = EventMessageExtractor(String::new());
    event.record(&mut v);

    format!("{ts} {level:5} {target}: {}\n", v.0)
}

// ---------------------------------------------------------------------------
// Entrée de log par span (deux writers : full et error)
// ---------------------------------------------------------------------------

struct SpanLogEntry {
    writer_full: Mutex<BufWriter<File>>,
    writer_error: Mutex<BufWriter<File>>,
}

impl SpanLogEntry {
    fn write(&self, event: &Event<'_>) {
        let line = format_event(event);
        let bytes = line.as_bytes();

        if let Ok(mut w) = self.writer_full.lock() {
            let _ = w.write_all(bytes);
        }
        // WARN et ERROR vont aussi dans le fichier erreur
        if *event.metadata().level() <= Level::WARN {
            if let Ok(mut w) = self.writer_error.lock() {
                let _ = w.write_all(bytes);
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut w) = self.writer_full.lock() {
            let _ = w.flush();
        }
        if let Ok(mut w) = self.writer_error.lock() {
            let _ = w.flush();
        }
    }
}

// ---------------------------------------------------------------------------
// Layer principal
// ---------------------------------------------------------------------------

/// Layer tracing qui route les événements de chaque job vers des fichiers dédiés.
pub struct JobLogLayer {
    hosts_path: PathBuf,
    jobs_path: PathBuf,
    /// Associe l'identifiant numérique d'un span à son entrée de log.
    span_map: RwLock<HashMap<u64, SpanLogEntry>>,
}

impl JobLogLayer {
    /// Crée le layer. `jobs_path` est créé si inexistant.
    pub fn new(hosts_path: PathBuf, jobs_path: PathBuf) -> Result<Self, std::io::Error> {
        fs::create_dir_all(&jobs_path)?;
        Ok(Self {
            hosts_path,
            jobs_path,
            span_map: RwLock::new(HashMap::new()),
        })
    }

    /// Ouvre (ou crée) les deux fichiers de log pour une entrée de span.
    fn open_entry(
        &self,
        path_full: &Path,
        path_error: &Path,
    ) -> Result<SpanLogEntry, std::io::Error> {
        if let Some(parent) = path_full.parent() {
            fs::create_dir_all(parent)?;
        }
        let file_full = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path_full)?;
        let file_error = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path_error)?;
        Ok(SpanLogEntry {
            writer_full: Mutex::new(BufWriter::new(file_full)),
            writer_error: Mutex::new(BufWriter::new(file_error)),
        })
    }
}

impl<S> Layer<S> for JobLogLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    /// Appelé à la création de chaque span.
    /// Si le span porte les champs `job_type` (obligatoire), `host` et
    /// `backup_number` (backup/restore) ou `task_id` (remove/refcnt/fsck),
    /// ouvre les fichiers de log correspondants.
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, _ctx: Context<'_, S>) {
        let mut ext = SpanFieldExtractor::default();
        attrs.record(&mut ext);

        let job_type = match ext.job_type.as_deref() {
            Some(jt) => jt.to_string(),
            None => return,
        };

        let result: Result<SpanLogEntry, std::io::Error> = match job_type.as_str() {
            "backup" => {
                if let (Some(host), Some(backup_id)) = (&ext.host, &ext.backup_id) {
                    let dir = self.hosts_path.join(host).join(backup_id);
                    self.open_entry(&dir.join("backup.log"), &dir.join("backup-error.log"))
                } else {
                    return;
                }
            }
            "restore" => {
                if let (Some(host), Some(backup_id)) = (&ext.host, &ext.backup_id) {
                    let dir = self.hosts_path.join(host).join(backup_id);
                    self.open_entry(&dir.join("restore.log"), &dir.join("restore-error.log"))
                } else {
                    return;
                }
            }
            "remove" | "refcnt" | "fsck" => {
                let task_id = ext.task_id.as_deref().unwrap_or("unknown");
                let base = format!("{job_type}-{task_id}");
                self.open_entry(
                    &self.jobs_path.join(format!("{base}.log")),
                    &self.jobs_path.join(format!("{base}-error.log")),
                )
            }
            _ => return,
        };

        match result {
            Ok(entry) => {
                if let Ok(mut map) = self.span_map.write() {
                    map.insert(id.into_u64(), entry);
                }
            }
            Err(e) => {
                eprintln!("JobLogLayer: impossible d'ouvrir le fichier de log: {e}");
            }
        }
    }

    /// Appelé pour chaque événement tracing.
    /// Cherche l'ancêtre de span le plus proche (innermost) qui a un fichier de log
    /// et y écrit l'événement.
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let map = match self.span_map.read() {
            Ok(m) => m,
            Err(_) => return,
        };

        // Parcourt la pile de spans du plus proche au plus éloigné
        let mut found_id: Option<u64> = None;
        if let Some(scope) = ctx.event_scope(event) {
            for span in scope {
                let sid = span.id().into_u64();
                if map.contains_key(&sid) {
                    found_id = Some(sid);
                    break; // ancêtre le plus proche (innermost)
                }
            }
        }

        if let Some(sid) = found_id {
            if let Some(entry) = map.get(&sid) {
                entry.write(event);
            }
        }
    }

    /// Appelé à la fermeture d'un span (fin du handler).
    /// Flush et libère les fichiers de log associés.
    fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
        if let Ok(mut map) = self.span_map.write() {
            if let Some(entry) = map.remove(&id.into_u64()) {
                entry.flush();
                // entry est drop ici → les File sont fermés automatiquement
            }
        }
    }
}
