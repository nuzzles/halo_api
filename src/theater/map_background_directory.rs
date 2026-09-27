//! Host sidecar loading and metadata-invalidated background index cache.
use super::{MapBackgroundIndex, load_map_background};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{self, DirEntry},
    io,
    path::Path,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::UNIX_EPOCH,
};

fn entries(dir: &Path) -> io::Result<Vec<DirEntry>> {
    let mut entries = fs::read_dir(dir)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(DirEntry::file_name);
    Ok(entries)
}
fn sidecar(entry: &DirEntry) -> io::Result<Option<Vec<u8>>> {
    if entry.file_type()?.is_dir() {
        return Ok(None);
    }
    let name = entry.file_name();
    if !name.as_encoded_bytes().ends_with(b".json") {
        return Ok(None);
    }
    let name = name.as_encoded_bytes();
    Ok(Some(name[..name.len() - 5].to_vec()))
}
fn build(entries: &[DirEntry]) -> io::Result<MapBackgroundIndex> {
    let mut backgrounds = Vec::new();
    for entry in entries {
        let Some(key) = sidecar(entry)? else {
            continue;
        };
        match load_map_background(entry.path()) {
            Ok(background) => backgrounds.push((key, background)),
            Err(error) => tracing::warn!(path=?entry.path(), %error, "background sidecar skipped"),
        }
    }
    let index = MapBackgroundIndex::from_background_bytes(
        backgrounds.iter().map(|(key, b)| (key.as_slice(), b)),
    );
    for (identity, keys) in index.ambiguous() {
        tracing::warn!(%identity, ?keys, "ambiguous background identity excluded");
    }
    Ok(index)
}
/// Read published sidecars. Malformed or unreadable sidecars are logged and skipped.
/// Directory listing errors are returned; raw filename bytes are retained in keys.
pub fn build_map_background_index(dir: impl AsRef<Path>) -> io::Result<MapBackgroundIndex> {
    build(&entries(dir.as_ref())?)
}

static UNDATABLE: AtomicU64 = AtomicU64::new(0);
fn signature(entries: &[DirEntry]) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    for entry in entries {
        if sidecar(entry)?.is_none() {
            continue;
        }
        out.extend_from_slice(entry.file_name().as_encoded_bytes());
        out.push(b':');
        let metadata = entry.metadata().and_then(|m| Ok((m.len(), m.modified()?)));
        match metadata {
            Ok((size, time)) => {
                let nanos = match time.duration_since(UNIX_EPOCH) {
                    Ok(d) => d.as_nanos() as i64,
                    Err(e) => (e.duration().as_nanos() as i64).wrapping_neg(),
                };
                out.extend_from_slice(format!("{size}:{nanos};").as_bytes());
            }
            Err(_) => {
                let unique = UNDATABLE.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
                out.extend_from_slice(format!("indatable-{unique};").as_bytes());
            }
        }
    }
    Ok(out)
}
struct Cached {
    signature: Vec<u8>,
    index: Arc<MapBackgroundIndex>,
}
static CACHE: OnceLock<Mutex<BTreeMap<OsString, Cached>>> = OnceLock::new();
/// Reuse the same index allocation while sidecar names, sizes and mtimes agree.
/// Each call lists/stats the directory; this is not a constant-time disk lookup.
/// Signatures intentionally describe metadata, not file contents, as in native Go.
pub fn map_background_index_for(dir: impl AsRef<Path>) -> io::Result<Arc<MapBackgroundIndex>> {
    let dir = dir.as_ref();
    let entries = entries(dir)?;
    let signature = signature(&entries)?;
    let mut cache = CACHE
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let key = dir.as_os_str();
    if let Some(cached) = cache.get(key).filter(|c| c.signature == signature) {
        return Ok(cached.index.clone());
    }
    let index = Arc::new(build(&entries)?);
    cache.insert(
        key.to_owned(),
        Cached {
            signature,
            index: index.clone(),
        },
    );
    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn background_directory_undatable_rebuilds() {
        let root = std::env::temp_dir().join(format!(
            "halo-bg-undatable-{}-{}",
            std::process::id(),
            UNIX_EPOCH.elapsed().unwrap().as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let file = root.join("a.json");
        fs::write(&file, b"{\"schemaVersion\":1}").unwrap();
        let listed = entries(&root).unwrap();
        fs::remove_file(file).unwrap();
        let first = signature(&listed).unwrap();
        let second = signature(&listed).unwrap();
        assert_ne!(first, second);
        assert!(first.starts_with(b"a.json:indatable-"));
        assert_eq!(build(&listed).unwrap().keys(), 0);
        fs::remove_dir(root).unwrap();
    }
}
