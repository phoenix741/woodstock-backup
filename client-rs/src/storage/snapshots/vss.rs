use std::ffi::OsString;
use std::mem::zeroed;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Component, Path, PathBuf, Prefix};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;

use eyre::{eyre, Result, WrapErr};
use tokio::task;
use tonic::async_trait;
use winapi::shared::guiddef::GUID;
use winapi::shared::minwindef::BOOL;
use winapi::shared::winerror::HRESULT;
use winapi::um::combaseapi::{CoInitializeEx, CoUninitialize};
use winapi::um::objbase::COINIT_MULTITHREADED;
use winapi::um::vsbackup::{
    CreateVssBackupComponents, IVssBackupComponents, VssFreeSnapshotProperties,
};
use winapi::um::vss::{
    IVssAsync, VSS_BACKUP_TYPE, VSS_BT_FULL, VSS_CTX_BACKUP, VSS_ID, VSS_OBJECT_SNAPSHOT,
    VSS_OBJECT_TYPE, VSS_SNAPSHOT_PROP,
};
use winapi::um::winbase::INFINITE;

use crate::storage::snapshots::{SnapshotCompletion, SnapshotManager, SnapshotReference};

const VSS_E_OBJECT_NOT_FOUND: HRESULT = 0x80042308u32 as i32;

#[derive(Debug, Clone)]
struct WindowsSnapshotTarget {
    volume_root: PathBuf,
    relative_path: PathBuf,
}

#[derive(Clone)]
pub struct VssSnapshotReference {
    redirection_path: PathBuf,
    snapshot_device_root: PathBuf,
    snapshot_id: String,
    snapshot_set_id: String,
    cleaned_up: Arc<AtomicBool>,
    session: Arc<Mutex<VssSessionController>>,
}

impl VssSnapshotReference {
    fn new(
        redirection_path: PathBuf,
        snapshot_device_root: PathBuf,
        snapshot_id: String,
        snapshot_set_id: String,
        session: Arc<Mutex<VssSessionController>>,
    ) -> Self {
        Self {
            redirection_path,
            snapshot_device_root,
            snapshot_id,
            snapshot_set_id,
            cleaned_up: Arc::new(AtomicBool::new(false)),
            session,
        }
    }
}

#[async_trait]
impl SnapshotReference for VssSnapshotReference {
    fn path(&self) -> &Path {
        &self.redirection_path
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn finalize_self(&self, completion: SnapshotCompletion) -> Result<()> {
        let cleanup_flag = Arc::clone(&self.cleaned_up);
        let session = Arc::clone(&self.session);
        let snapshot_path = self.snapshot_device_root.clone();
        let snapshot_id = self.snapshot_id.clone();
        let snapshot_set_id = self.snapshot_set_id.clone();

        spawn_blocking_vss(move || {
            tracing::info!(
                "WOODSTOCK_VSS finalize start snapshot_id={} snapshot_set_id={} device_root='{}' outcome={:?}",
                snapshot_id,
                snapshot_set_id,
                snapshot_path.display(),
                completion
            );
            let mut session = session
                .lock()
                .map_err(|_| eyre!("VSS session mutex was poisoned during finalization"))?;
            session.finalize(completion)?;
            cleanup_flag.store(true, Ordering::Release);
            tracing::info!(
                "WOODSTOCK_VSS finalize complete snapshot_id={} snapshot_set_id={} device_root='{}' outcome={:?}",
                snapshot_id,
                snapshot_set_id,
                snapshot_path.display(),
                completion
            );
            Ok(())
        })
        .await
    }
}

impl Drop for VssSnapshotReference {
    fn drop(&mut self) {
        if !self.cleaned_up.load(Ordering::Acquire) {
            tracing::warn!(
                "WOODSTOCK_VSS dropped without explicit finalization snapshot_id={} snapshot_set_id={} device_root='{}'. The session worker will abort cleanup as a fallback.",
                self.snapshot_id,
                self.snapshot_set_id,
                self.snapshot_device_root.display()
            );
        }
    }
}

#[derive(Clone)]
struct VssCreatedSnapshot {
    snapshot_id: VSS_ID,
    snapshot_set_id: VSS_ID,
    snapshot_device_root: PathBuf,
}

#[derive(Debug)]
enum VssSessionCommand {
    Finalize(SnapshotCompletion),
}

struct VssSessionController {
    finalize_tx: Option<SyncSender<VssSessionCommand>>,
    worker_handle: Option<thread::JoinHandle<Result<()>>>,
}

impl VssSessionController {
    fn finalize(&mut self, completion: SnapshotCompletion) -> Result<()> {
        let Some(worker_handle) = self.worker_handle.take() else {
            tracing::debug!(
                "VSS session finalization requested with outcome {:?}, but worker was already consumed",
                completion
            );
            return Ok(());
        };

        if let Some(finalize_tx) = self.finalize_tx.take() {
            tracing::debug!(
                "Sending VSS session finalization command {:?} to worker",
                completion
            );
            finalize_tx
                .send(VssSessionCommand::Finalize(completion))
                .map_err(|error| eyre!("Failed to notify VSS session worker: {}", error))?;
        }

        join_vss_worker(worker_handle)
    }
}

pub struct VssSnapshotManager;

impl VssSnapshotManager {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SnapshotManager for VssSnapshotManager {
    async fn create_snapshot(&self, source_path: &Path) -> Result<Box<dyn SnapshotReference>> {
        let source_path = source_path.to_path_buf();

        spawn_blocking_vss(move || {
            tracing::info!(
                "Attempting to create a VSS snapshot for '{}'",
                source_path.display()
            );
            let target = normalize_snapshot_target(&source_path)?;
            let (created_snapshot, session) = create_snapshot_for_volume(&target.volume_root)?;
            let redirection_path = if target.relative_path.as_os_str().is_empty() {
                created_snapshot.snapshot_device_root.clone()
            } else {
                created_snapshot
                    .snapshot_device_root
                    .join(&target.relative_path)
            };

            let snapshot_id = format_vss_id(&created_snapshot.snapshot_id);
            let snapshot_set_id = format_vss_id(&created_snapshot.snapshot_set_id);

            tracing::info!(
                "WOODSTOCK_VSS created snapshot_id={} snapshot_set_id={} source='{}' volume='{}' device_root='{}' redirection='{}'",
                snapshot_id,
                snapshot_set_id,
                source_path.display(),
                target.volume_root.display(),
                created_snapshot.snapshot_device_root.display(),
                redirection_path.display()
            );

            Ok(Box::new(VssSnapshotReference::new(
                redirection_path,
                created_snapshot.snapshot_device_root,
                snapshot_id,
                snapshot_set_id,
                session,
            )) as Box<dyn SnapshotReference>)
        })
        .await
    }

    async fn is_available(&self, source_path: &Path) -> Result<bool> {
        let source_path = source_path.to_path_buf();

        spawn_blocking_vss(move || {
            let target = match normalize_snapshot_target(&source_path) {
                Ok(target) => target,
                Err(error) => {
                    tracing::debug!(
                        "VSS is not available for '{}' because path normalization failed: {}",
                        source_path.display(),
                        error
                    );
                    return Ok(false);
                }
            };

            is_volume_supported(&target.volume_root)
        })
        .await
    }

    fn manager_name(&self) -> &'static str {
        "VSS"
    }

    fn priority(&self) -> u8 {
        110
    }
}

async fn spawn_blocking_vss<T, F>(operation: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    task::spawn_blocking(operation)
        .await
        .wrap_err("VSS background task failed")?
}

fn normalize_snapshot_target(source_path: &Path) -> Result<WindowsSnapshotTarget> {
    let source_path = strip_verbatim_prefix(source_path);

    if !source_path.is_absolute() {
        return Err(eyre!(
            "VSS requires an absolute local path, got '{}'",
            source_path.display()
        ));
    }

    let mut components = source_path.components();
    let drive_letter = match components.next() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::Disk(letter) => letter as char,
            _ => {
                return Err(eyre!(
                    "VSS currently supports only local drive-letter paths, got '{}'",
                    source_path.display()
                ));
            }
        },
        _ => {
            return Err(eyre!(
                "VSS currently supports only local drive-letter paths, got '{}'",
                source_path.display()
            ));
        }
    };

    match components.next() {
        Some(Component::RootDir) => {}
        _ => {
            return Err(eyre!(
                "VSS path '{}' must target a volume root or a path under it",
                source_path.display()
            ));
        }
    }

    let volume_root = PathBuf::from(format!("{}:\\", drive_letter.to_ascii_uppercase()));
    let relative_path = source_path
        .strip_prefix(&volume_root)
        .map(PathBuf::from)
        .map_err(|error| {
            eyre!(
                "Failed to compute VSS relative path from '{}' to '{}': {}",
                volume_root.display(),
                source_path.display(),
                error
            )
        })?;

    Ok(WindowsSnapshotTarget {
        volume_root,
        relative_path,
    })
}

fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    let raw = path.as_os_str().to_string_lossy();
    if let Some(stripped) = raw.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        path.to_path_buf()
    }
}

fn is_volume_supported(volume_root: &Path) -> Result<bool> {
    tracing::debug!(
        "Checking whether VSS supports volume '{}'",
        volume_root.display()
    );
    let _com = ComGuard::initialize()?;
    let backup = BackupComponentsGuard::create()?;
    let volume_name = wide_null(volume_root);
    let mut supported: BOOL = 0;
    let provider_id: GUID = unsafe { zeroed() };

    check_hresult(
        unsafe { (*backup.as_ptr()).InitializeForBackup(null_mut()) },
        "IVssBackupComponents::InitializeForBackup",
    )?;
    check_hresult(
        unsafe { (*backup.as_ptr()).SetContext(VSS_CTX_BACKUP as i32) },
        "IVssBackupComponents::SetContext",
    )?;

    check_hresult(
        unsafe {
            (*backup.as_ptr()).IsVolumeSupported(
                provider_id,
                volume_name.as_ptr() as *mut _,
                &mut supported,
            )
        },
        "IVssBackupComponents::IsVolumeSupported",
    )?;

    let supported = supported != 0;
    tracing::info!(
        "VSS support probe for volume '{}' returned {}",
        volume_root.display(),
        supported
    );

    Ok(supported)
}

fn create_snapshot_for_volume(
    volume_root: &Path,
) -> Result<(VssCreatedSnapshot, Arc<Mutex<VssSessionController>>)> {
    tracing::info!(
        "Starting VSS session worker for volume '{}'",
        volume_root.display()
    );
    let (created_tx, created_rx) = mpsc::sync_channel(1);
    let (finalize_tx, finalize_rx) = mpsc::sync_channel(1);
    let volume_root = volume_root.to_path_buf();
    let worker_volume_root = volume_root.clone();

    let worker_handle =
        thread::spawn(move || vss_session_worker(worker_volume_root, created_tx, finalize_rx));

    let created_snapshot = match created_rx.recv() {
        Ok(result) => result?,
        Err(error) => {
            let worker_result = join_vss_worker(worker_handle);
            if let Err(join_error) = worker_result {
                return Err(join_error.wrap_err(format!(
                    "VSS session worker did not return snapshot creation result: {error}"
                )));
            }

            return Err(eyre!(
                "VSS session worker did not return snapshot creation result: {}",
                error
            ));
        }
    };

    let session = Arc::new(Mutex::new(VssSessionController {
        finalize_tx: Some(finalize_tx),
        worker_handle: Some(worker_handle),
    }));

    tracing::info!(
        "WOODSTOCK_VSS worker registered snapshot_id={} snapshot_set_id={} volume='{}' device_root='{}'",
        format_vss_id(&created_snapshot.snapshot_id),
        format_vss_id(&created_snapshot.snapshot_set_id),
        volume_root.display(),
        created_snapshot.snapshot_device_root.display()
    );

    Ok((created_snapshot, session))
}

fn vss_session_worker(
    volume_root: PathBuf,
    created_tx: SyncSender<Result<VssCreatedSnapshot>>,
    finalize_rx: Receiver<VssSessionCommand>,
) -> Result<()> {
    tracing::debug!(
        "VSS session worker booting for volume '{}'",
        volume_root.display()
    );
    let _com = ComGuard::initialize()?;
    let backup = BackupComponentsGuard::create()?;

    let created_snapshot = match create_snapshot_in_worker(&backup, &volume_root) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            tracing::error!(
                "VSS snapshot creation failed for volume '{}': {}",
                volume_root.display(),
                error
            );
            let _ = created_tx.send(Err(error));
            return Ok(());
        }
    };

    tracing::info!(
        "WOODSTOCK_VSS worker created snapshot_id={} snapshot_set_id={} volume='{}' device_root='{}'",
        format_vss_id(&created_snapshot.snapshot_id),
        format_vss_id(&created_snapshot.snapshot_set_id),
        volume_root.display(),
        created_snapshot.snapshot_device_root.display()
    );

    if created_tx.send(Ok(created_snapshot.clone())).is_err() {
        tracing::warn!(
            "VSS snapshot session for '{}' was dropped before registration; aborting it immediately",
            volume_root.display()
        );
        return finalize_worker_session(
            &backup,
            created_snapshot.snapshot_id,
            SnapshotCompletion::Abort,
        );
    }

    match finalize_rx.recv() {
        Ok(VssSessionCommand::Finalize(completion)) => {
            finalize_worker_session(&backup, created_snapshot.snapshot_id, completion)
        }
        Err(_) => {
            tracing::warn!(
                "VSS snapshot session for '{}' lost its owner before finalization; aborting it as a fallback",
                volume_root.display()
            );
            finalize_worker_session(
                &backup,
                created_snapshot.snapshot_id,
                SnapshotCompletion::Abort,
            )
        }
    }
}

fn create_snapshot_in_worker(
    backup: &BackupComponentsGuard,
    volume_root: &Path,
) -> Result<VssCreatedSnapshot> {
    tracing::debug!(
        "Beginning VSS requester flow for volume '{}'",
        volume_root.display()
    );
    let volume_name = wide_null(volume_root);
    let provider_id: GUID = unsafe { zeroed() };
    let mut snapshot_set_id: VSS_ID = unsafe { zeroed() };
    let mut snapshot_id: VSS_ID = unsafe { zeroed() };
    let mut properties: VSS_SNAPSHOT_PROP = unsafe { zeroed() };

    check_hresult(
        unsafe { (*backup.as_ptr()).InitializeForBackup(null_mut()) },
        "IVssBackupComponents::InitializeForBackup",
    )?;
    check_hresult(
        unsafe { (*backup.as_ptr()).SetContext(VSS_CTX_BACKUP as i32) },
        "IVssBackupComponents::SetContext",
    )?;
    check_hresult(
        unsafe {
            (*backup.as_ptr()).SetBackupState(false, false, VSS_BT_FULL as VSS_BACKUP_TYPE, false)
        },
        "IVssBackupComponents::SetBackupState",
    )?;

    run_vss_async(
        unsafe {
            let mut async_handle: *mut IVssAsync = null_mut();
            check_hresult(
                (*backup.as_ptr()).GatherWriterMetadata(&mut async_handle),
                "IVssBackupComponents::GatherWriterMetadata",
            )?;
            Ok(async_handle)
        },
        "GatherWriterMetadata",
    )?;

    check_hresult(
        unsafe { (*backup.as_ptr()).StartSnapshotSet(&mut snapshot_set_id) },
        "IVssBackupComponents::StartSnapshotSet",
    )?;
    tracing::debug!(
        "WOODSTOCK_VSS start snapshot_set_id={} volume='{}'",
        format_vss_id(&snapshot_set_id),
        volume_root.display()
    );
    check_hresult(
        unsafe {
            (*backup.as_ptr()).AddToSnapshotSet(
                volume_name.as_ptr() as *mut _,
                provider_id,
                &mut snapshot_id,
            )
        },
        "IVssBackupComponents::AddToSnapshotSet",
    )?;
    tracing::debug!(
        "WOODSTOCK_VSS add volume='{}' snapshot_set_id={} snapshot_id={}",
        volume_root.display(),
        format_vss_id(&snapshot_set_id),
        format_vss_id(&snapshot_id)
    );

    run_vss_async(
        unsafe {
            let mut async_handle: *mut IVssAsync = null_mut();
            check_hresult(
                (*backup.as_ptr()).PrepareForBackup(&mut async_handle),
                "IVssBackupComponents::PrepareForBackup",
            )?;
            Ok(async_handle)
        },
        "PrepareForBackup",
    )?;

    run_vss_async(
        unsafe {
            let mut async_handle: *mut IVssAsync = null_mut();
            check_hresult(
                (*backup.as_ptr()).DoSnapshotSet(&mut async_handle),
                "IVssBackupComponents::DoSnapshotSet",
            )?;
            Ok(async_handle)
        },
        "DoSnapshotSet",
    )?;

    check_hresult(
        unsafe { (*backup.as_ptr()).GetSnapshotProperties(snapshot_id, &mut properties) },
        "IVssBackupComponents::GetSnapshotProperties",
    )?;

    let snapshot_device_root = unsafe { wide_ptr_to_path(properties.m_pwszSnapshotDeviceObject) }?;
    tracing::info!(
        "WOODSTOCK_VSS properties snapshot_id={} snapshot_set_id={} volume='{}' device_root='{}'",
        format_vss_id(&snapshot_id),
        format_vss_id(&snapshot_set_id),
        volume_root.display(),
        snapshot_device_root.display()
    );

    unsafe {
        VssFreeSnapshotProperties(&mut properties);
        let _ = (*backup.as_ptr()).FreeWriterMetadata();
    }

    Ok(VssCreatedSnapshot {
        snapshot_id,
        snapshot_set_id,
        snapshot_device_root,
    })
}

fn finalize_worker_session(
    backup: &BackupComponentsGuard,
    snapshot_id: VSS_ID,
    completion: SnapshotCompletion,
) -> Result<()> {
    tracing::info!(
        "Finalizing worker-managed VSS snapshot {} with outcome {:?}",
        format_vss_id(&snapshot_id),
        completion
    );
    let completion_result = match completion {
        SnapshotCompletion::Success => complete_backup_session(backup),
        SnapshotCompletion::Abort => abort_backup_session(backup),
    };
    let delete_result = delete_snapshot_with_backup(
        backup,
        snapshot_id,
        matches!(completion, SnapshotCompletion::Abort),
    );

    match (completion_result, delete_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Err(completion_error), Err(delete_error)) => Err(completion_error.wrap_err(delete_error)),
    }
}

fn complete_backup_session(backup: &BackupComponentsGuard) -> Result<()> {
    tracing::debug!("Requesting VSS BackupComplete");
    run_vss_async(
        unsafe {
            let mut async_handle: *mut IVssAsync = null_mut();
            check_hresult(
                (*backup.as_ptr()).BackupComplete(&mut async_handle),
                "IVssBackupComponents::BackupComplete",
            )?;
            Ok(async_handle)
        },
        "BackupComplete",
    )
}

fn abort_backup_session(backup: &BackupComponentsGuard) -> Result<()> {
    tracing::debug!("Requesting VSS AbortBackup");
    check_hresult(
        unsafe { (*backup.as_ptr()).AbortBackup() },
        "IVssBackupComponents::AbortBackup",
    )
}

fn delete_snapshot_with_backup(
    backup: &BackupComponentsGuard,
    snapshot_id: VSS_ID,
    allow_missing_snapshot: bool,
) -> Result<()> {
    tracing::debug!(
        "Requesting VSS DeleteSnapshots for snapshot {}",
        format_vss_id(&snapshot_id)
    );
    let mut deleted_snapshots = 0;
    let mut undeleted_snapshot_id: VSS_ID = unsafe { zeroed() };

    let delete_status = unsafe {
        (*backup.as_ptr()).DeleteSnapshots(
            snapshot_id,
            VSS_OBJECT_SNAPSHOT as VSS_OBJECT_TYPE,
            0,
            &mut deleted_snapshots,
            &mut undeleted_snapshot_id,
        )
    };

    if allow_missing_snapshot && delete_status == VSS_E_OBJECT_NOT_FOUND {
        tracing::warn!(
            "DeleteSnapshots reported VSS_E_OBJECT_NOT_FOUND for snapshot {} after abort; treating it as already cleaned up by VSS",
            format_vss_id(&snapshot_id)
        );
        return Ok(());
    }

    check_hresult(delete_status, "IVssBackupComponents::DeleteSnapshots")?;

    if deleted_snapshots == 0 {
        return Err(eyre!(
            "VSS did not delete snapshot {:08x}-{:04x}-{:04x}",
            snapshot_id.Data1,
            snapshot_id.Data2,
            snapshot_id.Data3
        ));
    }

    tracing::info!(
        "VSS deleted {} snapshot(s) while cleaning snapshot {}",
        deleted_snapshots,
        format_vss_id(&snapshot_id)
    );

    if !is_zero_guid(&undeleted_snapshot_id) {
        tracing::warn!(
            "VSS reported an undeleted snapshot id {} while deleting {}",
            format_vss_id(&undeleted_snapshot_id),
            format_vss_id(&snapshot_id)
        );
    }

    Ok(())
}

fn join_vss_worker(worker_handle: thread::JoinHandle<Result<()>>) -> Result<()> {
    worker_handle
        .join()
        .map_err(|_| eyre!("VSS session worker panicked"))?
}

fn run_vss_async(async_handle: Result<*mut IVssAsync>, operation: &str) -> Result<()> {
    tracing::debug!("Waiting for VSS async operation '{}'", operation);
    let async_handle = AsyncGuard::new(async_handle?)?;
    let mut operation_status: HRESULT = 0;
    let mut reserved = 0;

    check_hresult(
        unsafe { (*async_handle.as_ptr()).Wait(INFINITE) },
        &format!("IVssAsync::Wait for {operation}"),
    )?;
    check_hresult(
        unsafe { (*async_handle.as_ptr()).QueryStatus(&mut operation_status, &mut reserved) },
        &format!("IVssAsync::QueryStatus for {operation}"),
    )?;
    check_hresult(
        operation_status,
        &format!("VSS async operation {operation}"),
    )?;

    tracing::debug!(
        "VSS async operation '{}' completed with status 0x{:08X} (reserved={})",
        operation,
        operation_status as u32,
        reserved
    );

    Ok(())
}

fn check_hresult(status: HRESULT, operation: &str) -> Result<()> {
    if status < 0 {
        tracing::error!("{} failed with HRESULT 0x{:08X}", operation, status as u32);
        Err(eyre!(
            "{} failed with HRESULT 0x{:08X}",
            operation,
            status as u32
        ))
    } else {
        Ok(())
    }
}

fn format_vss_id(value: &VSS_ID) -> String {
    format!(
        "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        value.Data1,
        value.Data2,
        value.Data3,
        value.Data4[0],
        value.Data4[1],
        value.Data4[2],
        value.Data4[3],
        value.Data4[4],
        value.Data4[5],
        value.Data4[6],
        value.Data4[7]
    )
}

fn is_zero_guid(value: &VSS_ID) -> bool {
    value.Data1 == 0 && value.Data2 == 0 && value.Data3 == 0 && value.Data4 == [0; 8]
}

fn wide_null(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

unsafe fn wide_ptr_to_path(value: *mut u16) -> Result<PathBuf> {
    if value.is_null() {
        return Err(eyre!("VSS returned a null snapshot device path"));
    }

    let mut len = 0;
    while *value.add(len) != 0 {
        len += 1;
    }

    let wide = std::slice::from_raw_parts(value, len);
    Ok(PathBuf::from(OsString::from_wide(wide)))
}

struct ComGuard;

impl ComGuard {
    fn initialize() -> Result<Self> {
        check_hresult(
            unsafe { CoInitializeEx(null_mut(), COINIT_MULTITHREADED) },
            "CoInitializeEx",
        )?;
        Ok(Self)
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

struct BackupComponentsGuard(*mut IVssBackupComponents);

impl BackupComponentsGuard {
    fn create() -> Result<Self> {
        let mut backup: *mut IVssBackupComponents = null_mut();
        check_hresult(
            unsafe { CreateVssBackupComponents(&mut backup) },
            "CreateVssBackupComponents",
        )?;

        if backup.is_null() {
            return Err(eyre!("CreateVssBackupComponents returned a null interface"));
        }

        Ok(Self(backup))
    }

    fn as_ptr(&self) -> *mut IVssBackupComponents {
        self.0
    }
}

impl Drop for BackupComponentsGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                (*self.0).Release();
            }
        }
    }
}

struct AsyncGuard(*mut IVssAsync);

impl AsyncGuard {
    fn new(handle: *mut IVssAsync) -> Result<Self> {
        if handle.is_null() {
            return Err(eyre!("VSS returned a null async handle"));
        }

        Ok(Self(handle))
    }

    fn as_ptr(&self) -> *mut IVssAsync {
        self.0
    }
}

impl Drop for AsyncGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                (*self.0).Release();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::process::Command;
    use std::sync::Once;
    use tempfile::TempDir;
    use tracing_subscriber::EnvFilter;

    static TEST_TRACING: Once = Once::new();

    fn init_test_tracing() {
        TEST_TRACING.call_once(|| {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(
                    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
                )
                .with_test_writer()
                .try_init();
        });
    }

    #[test]
    fn parses_drive_letter_paths() {
        let target = normalize_snapshot_target(Path::new(r"C:\Users\phoenix\data")).unwrap();
        assert_eq!(target.volume_root, PathBuf::from(r"C:\"));
        assert_eq!(target.relative_path, PathBuf::from(r"Users\phoenix\data"));
    }

    #[test]
    fn rejects_unc_paths() {
        let result = normalize_snapshot_target(Path::new(r"\\server\share\data"));
        assert!(result.is_err());
    }

    #[tokio::test]
    #[cfg(windows)]
    #[ignore = "requires local Windows VSS support and sufficient privileges"]
    async fn test_vss_snapshot_manager_e2e_success() -> Result<()> {
        init_test_tracing();
        let fixture = create_vss_fixture()?;
        let manager = VssSnapshotManager::new();

        tracing::info!(
            "Starting VSS E2E success test on '{}'",
            fixture.root.path().display()
        );
        dump_shadow_copies("before success test")?;

        if !manager.is_available(fixture.root.path()).await? {
            eprintln!(
                "VSS is not available for '{}', skipping test.",
                fixture.root.path().display()
            );
            return Ok(());
        }

        let snapshot_ref = manager.create_snapshot(fixture.root.path()).await?;
        let snapshot_root = snapshot_ref.path().to_path_buf();
        let snapshot_file = snapshot_root.join(&fixture.relative_file);

        tracing::info!(
            "VSS E2E success test obtained snapshot root '{}' and snapshot file '{}'",
            snapshot_root.display(),
            snapshot_file.display()
        );
        dump_shadow_copies("after success snapshot creation")?;

        assert!(snapshot_root.exists(), "snapshot root should exist");
        assert!(snapshot_file.exists(), "snapshot file should exist");

        let snapshot_content = fs::read_to_string(&snapshot_file)?;
        assert_eq!(snapshot_content, fixture.content);

        snapshot_ref
            .finalize_self(SnapshotCompletion::Success)
            .await?;

        dump_shadow_copies("after success snapshot cleanup")?;
        assert_snapshot_is_gone(&snapshot_file)?;

        tracing::info!("VSS E2E success test completed successfully");

        Ok(())
    }

    #[tokio::test]
    #[cfg(windows)]
    #[ignore = "requires local Windows VSS support and sufficient privileges"]
    async fn test_vss_snapshot_manager_e2e_abort_cleanup() -> Result<()> {
        init_test_tracing();
        let fixture = create_vss_fixture()?;
        let manager = VssSnapshotManager::new();

        tracing::info!(
            "Starting VSS E2E abort test on '{}'",
            fixture.root.path().display()
        );
        dump_shadow_copies("before abort test")?;

        if !manager.is_available(fixture.root.path()).await? {
            eprintln!(
                "VSS is not available for '{}', skipping test.",
                fixture.root.path().display()
            );
            return Ok(());
        }

        let snapshot_ref = manager.create_snapshot(fixture.root.path()).await?;
        let snapshot_file = snapshot_ref.path().join(&fixture.relative_file);

        tracing::info!(
            "VSS E2E abort test obtained snapshot file '{}'",
            snapshot_file.display()
        );
        dump_shadow_copies("after abort snapshot creation")?;

        assert!(
            snapshot_file.exists(),
            "snapshot file should exist before abort cleanup"
        );

        snapshot_ref
            .finalize_self(SnapshotCompletion::Abort)
            .await?;

        dump_shadow_copies("after abort snapshot cleanup")?;
        assert_snapshot_is_gone(&snapshot_file)?;

        tracing::info!("VSS E2E abort test completed successfully");

        Ok(())
    }

    #[cfg(windows)]
    struct VssFixture {
        root: TempDir,
        relative_file: PathBuf,
        content: String,
    }

    #[cfg(windows)]
    fn create_vss_fixture() -> Result<VssFixture> {
        let root = tempfile::Builder::new()
            .prefix("woodstock-vss-e2e-")
            .tempdir()?;
        let nested_dir = root.path().join("nested");
        fs::create_dir_all(&nested_dir)?;

        let relative_file = PathBuf::from(r"nested\fixture.txt");
        let fixture_file = root.path().join(&relative_file);
        let content = format!(
            "woodstock-vss-e2e-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| eyre!("Failed to compute fixture timestamp: {}", error))?
                .as_nanos()
        );

        let mut handle = fs::File::create(&fixture_file)?;
        handle.write_all(content.as_bytes())?;
        handle.sync_all()?;

        Ok(VssFixture {
            root,
            relative_file,
            content,
        })
    }

    #[cfg(windows)]
    fn assert_snapshot_is_gone(snapshot_path: &Path) -> Result<()> {
        if snapshot_path.exists() {
            return Err(eyre!(
                "Snapshot path still exists after cleanup: {}",
                snapshot_path.display()
            ));
        }

        if let Ok(metadata) = fs::metadata(snapshot_path) {
            return Err(eyre!(
                "Snapshot metadata is still accessible after cleanup: {} ({:?})",
                snapshot_path.display(),
                metadata.file_type()
            ));
        }

        Ok(())
    }

    #[cfg(windows)]
    fn dump_shadow_copies(stage: &str) -> Result<()> {
        tracing::info!("Dumping shadow copies at stage '{}'", stage);

        let vssadmin_output = Command::new("vssadmin").args(["list", "shadows"]).output();

        match vssadmin_output {
            Ok(output) => {
                tracing::info!(
                    "[{}] vssadmin stdout:\n{}",
                    stage,
                    String::from_utf8_lossy(&output.stdout)
                );
                if !output.stderr.is_empty() {
                    tracing::warn!(
                        "[{}] vssadmin stderr:\n{}",
                        stage,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
            Err(error) => {
                tracing::warn!("[{}] Failed to run vssadmin list shadows: {}", stage, error);
            }
        }

        let cim_output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-CimInstance Win32_ShadowCopy | Select-Object ID, VolumeName, DeviceObject, InstallDate, State | Format-List | Out-String -Width 4096",
            ])
            .output();

        match cim_output {
            Ok(output) => {
                tracing::info!(
                    "[{}] Get-CimInstance Win32_ShadowCopy stdout:\n{}",
                    stage,
                    String::from_utf8_lossy(&output.stdout)
                );
                if !output.stderr.is_empty() {
                    tracing::warn!(
                        "[{}] Get-CimInstance Win32_ShadowCopy stderr:\n{}",
                        stage,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
            Err(error) => {
                tracing::warn!(
                    "[{}] Failed to run Get-CimInstance Win32_ShadowCopy: {}",
                    stage,
                    error
                );
            }
        }

        Ok(())
    }
}
