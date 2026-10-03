//! Exact files produced by the application, bound to their account and identity.
use crate::workspace::{store::Store, AccountGuard};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::{Component, Path, PathBuf},
};

#[derive(Deserialize, Serialize, PartialEq, Eq)]
struct Identity {
    length: u64,
    modified_nanos: u128,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
}
#[derive(Deserialize, Serialize)]
struct ProducedFile {
    canonical: PathBuf,
    identity: Identity,
    digest: String,
}
fn location(path: &Path) -> Result<(PathBuf, PathBuf, Identity), String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    if absolute
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return Err("FILE_OPEN_REFUSED: Parent traversal".into());
    }
    let metadata = std::fs::symlink_metadata(&absolute).map_err(|e| e.to_string())?;
    if !metadata.file_type().is_file() {
        return Err("FILE_OPEN_REFUSED: File is not a regular produced file".into());
    }
    let canonical = absolute.canonicalize().map_err(|e| e.to_string())?;
    let identity = file_identity(&metadata)?;
    Ok((absolute, canonical, identity))
}
fn file_identity(metadata: &std::fs::Metadata) -> Result<Identity, String> {
    Ok(Identity {
        length: metadata.len(),
        modified_nanos: metadata
            .modified()
            .map_err(|e| e.to_string())?
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos(),
        #[cfg(unix)]
        device: {
            use std::os::unix::fs::MetadataExt;
            metadata.dev()
        },
        #[cfg(unix)]
        inode: {
            use std::os::unix::fs::MetadataExt;
            metadata.ino()
        },
    })
}
fn key(path: &Path) -> String {
    format!("{:x}", Sha256::digest(path.to_string_lossy().as_bytes()))
}
fn digest(path: &Path) -> Result<String, String> {
    #[cfg(feature = "native-e2e")]
    let gate = FILE_GATE.lock().unwrap_or_else(|e| e.into_inner()).take();
    #[cfg(feature = "native-e2e")]
    if let Some((started, release)) = gate {
        std::fs::write(started, b"ready").map_err(|e| e.to_string())?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while !release.is_file() {
            if std::time::Instant::now() >= deadline {
                return Err("File gate deadline".into());
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
        #[cfg(feature = "native-e2e")]
        HASHED_BYTES.fetch_add(count as u64, std::sync::atomic::Ordering::SeqCst);
    }
    Ok(format!("{:x}", hash.finalize()))
}
/// Call only after successful production/publication and existing protection checks.
pub(crate) fn register(account: &AccountGuard, path: &Path) -> Result<(), String> {
    account.validate()?;
    let (absolute, canonical, identity) = location(path)?;
    let store = Store::open(&account.root, account.owner)?;
    let record_key = key(&absolute);
    if store
        .record::<ProducedFile>("external-produced-v1", &record_key)?
        .is_some_and(|saved| saved.canonical == canonical && saved.identity == identity)
    {
        return account.validate();
    }
    let digest = digest(&canonical)?;
    let (_, after_path, after_identity) = location(&absolute)?;
    if after_path != canonical || after_identity != identity {
        return Err("FILE_OPEN_REFUSED: File changed during registration".into());
    }
    account.validate()?;
    store.put_record(
        "external-produced-v1",
        &record_key,
        &ProducedFile {
            canonical,
            identity,
            digest,
        },
    )?;
    account.validate()
}
fn validate_lease(account: &AccountGuard, path: &Path) -> Result<ValidatedFile, String> {
    account.validate()?;
    let (absolute, canonical, identity) = location(path)?;
    let record = Store::open(&account.root, account.owner)?
        .record::<ProducedFile>("external-produced-v1", &key(&absolute))?
        .ok_or("FILE_OPEN_REFUSED: File was not produced by this application")?;
    if record.canonical != canonical
        || record.identity != identity
        || record.digest != digest(&canonical)?
    {
        return Err("FILE_OPEN_REFUSED: Produced file was replaced or modified".into());
    }
    let (_, after_path, after_identity) = location(&absolute)?;
    if after_path != canonical || after_identity != identity {
        return Err("FILE_OPEN_REFUSED: File changed before opening".into());
    }
    account.validate()?;
    #[cfg(feature = "native-e2e")]
    wait_gate(&VALIDATED_GATE)?;
    Ok(ValidatedFile {
        path: canonical,
        identity,
    })
}

static FILE_IO_SLOTS: std::sync::LazyLock<std::sync::Arc<tokio::sync::Semaphore>> =
    std::sync::LazyLock::new(|| std::sync::Arc::new(tokio::sync::Semaphore::new(2)));
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    let permit = FILE_IO_SLOTS
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| "File workers stopped")?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        work()
    })
    .await
    .map_err(|e| e.to_string())?
}
pub(crate) async fn register_async(account: AccountGuard, path: PathBuf) -> Result<(), String> {
    blocking(move || register(&account, &path)).await
}
pub(crate) struct ValidatedFile {
    path: PathBuf,
    identity: Identity,
}
impl ValidatedFile {
    pub(crate) fn checked(self, account: &AccountGuard) -> Result<PathBuf, String> {
        account.validate()?;
        let (_, canonical, identity) = location(&self.path)?;
        if canonical != self.path || identity != self.identity {
            return Err("FILE_OPEN_REFUSED: File changed before launch".into());
        }
        Ok(self.path)
    }
}
pub(crate) async fn validate_async(
    account: AccountGuard,
    path: PathBuf,
) -> Result<ValidatedFile, String> {
    blocking(move || validate_lease(&account, &path)).await
}

/// Reuse only a previously verified produced file. Older unregistered cache
/// entries remain available internally but gain no external-opening permission.
pub(crate) async fn reuse_cached(account: AccountGuard, path: PathBuf) -> Result<bool, String> {
    blocking(move || {
        let (absolute, _, _) = location(&path)?;
        if Store::open(&account.root, account.owner)?
            .record::<ProducedFile>("external-produced-v1", &key(&absolute))?
            .is_none()
        {
            account.validate()?;
            return Ok(false);
        }
        let authenticated = validate_lease(&account, &path)?;
        let (absolute, canonical, before) = location(&path)?;
        if canonical != authenticated.path || before != authenticated.identity {
            return Err("FILE_OPEN_REFUSED: Cache changed after validation".into());
        }
        let store = Store::open(&account.root, account.owner)?;
        let record_key = key(&absolute);
        let mut record = store
            .record::<ProducedFile>("external-produced-v1", &record_key)?
            .ok_or("FILE_OPEN_REFUSED")?;
        let handle = std::fs::OpenOptions::new()
            .write(true)
            .open(&canonical)
            .map_err(|e| e.to_string())?;
        if file_identity(&handle.metadata().map_err(|e| e.to_string())?)? != authenticated.identity
        {
            return Err("FILE_OPEN_REFUSED: Opened cache changed".into());
        }
        handle
            .set_times(std::fs::FileTimes::new().set_modified(std::time::SystemTime::now()))
            .map_err(|e| e.to_string())?;
        let (_, after_path, after) = location(&path)?;
        let touched = file_identity(&handle.metadata().map_err(|e| e.to_string())?)?;
        if after_path != canonical || after != touched || after.length != before.length {
            return Err("FILE_OPEN_REFUSED: Cache changed".into());
        }
        record.identity = after;
        account.validate()?;
        store.put_record("external-produced-v1", &record_key, &record)?;
        account.validate()?;
        Ok(true)
    })
    .await
}
#[cfg(feature = "native-e2e")]
type FileGate = Option<(PathBuf, PathBuf)>;
#[cfg(feature = "native-e2e")]
static FILE_GATE: std::sync::Mutex<FileGate> = std::sync::Mutex::new(None);
#[cfg(feature = "native-e2e")]
static HASHED_BYTES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
#[cfg(feature = "native-e2e")]
pub(crate) fn test_hold(started: PathBuf, release: PathBuf) {
    *FILE_GATE.lock().unwrap_or_else(|e| e.into_inner()) = Some((started, release));
}
#[cfg(feature = "native-e2e")]
pub(crate) fn test_status() -> (usize, u64) {
    (
        FILE_IO_SLOTS.available_permits(),
        HASHED_BYTES.load(std::sync::atomic::Ordering::SeqCst),
    )
}

#[cfg(feature = "native-e2e")]
static VALIDATED_GATE: std::sync::Mutex<FileGate> = std::sync::Mutex::new(None);
#[cfg(feature = "native-e2e")]
fn wait_gate(slot: &std::sync::Mutex<FileGate>) -> Result<(), String> {
    let gate = slot.lock().unwrap_or_else(|e| e.into_inner()).take();
    if let Some((started, release)) = gate {
        std::fs::write(started, b"ready").map_err(|e| e.to_string())?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while !release.is_file() {
            if std::time::Instant::now() >= deadline {
                return Err("File gate deadline".into());
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    Ok(())
}
#[cfg(feature = "native-e2e")]
pub(crate) fn test_hold_validated(started: PathBuf, release: PathBuf) {
    *VALIDATED_GATE.lock().unwrap_or_else(|e| e.into_inner()) = Some((started, release));
}
