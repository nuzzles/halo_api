//! Native disk-cache access. Reading raw cache bytes is not native re-encoding.
use super::{FilmSource, FilmSourceError, FilmSourceMetadata};
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum FilmCacheError {
    #[error("film cache I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid film cache manifest: {0}")]
    Json(#[from] serde_json::Error),
    #[error("film cache source: {0}")]
    Source(#[from] FilmSourceError),
}

/// Native manifest integers are signed machine words (64-bit at the reference pin).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FilmCacheMetadata {
    pub index: i64,
    pub chunk_type: i64,
    pub start_ms: i64,
}
impl<'de> Deserialize<'de> for FilmCacheMetadata {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Fields;
        impl<'de> Visitor<'de> for Fields {
            type Value = FilmCacheMetadata;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a chunk metadata object or null")
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(Self::Value::default())
            }
            fn visit_map<M: MapAccess<'de>>(self, mut m: M) -> Result<Self::Value, M::Error> {
                let mut out = Self::Value::default();
                while let Some(k) = m.next_key::<String>()? {
                    let field = match k.to_ascii_lowercase().as_str() {
                        "index" => Some(&mut out.index),
                        "chunk_type" => Some(&mut out.chunk_type),
                        "start_ms" => Some(&mut out.start_ms),
                        _ => None,
                    };
                    if let Some(field) = field {
                        if let Some(value) = m.next_value::<Option<i64>>()? {
                            *field = value;
                        }
                    } else {
                        m.next_value::<serde::de::IgnoredAny>()?;
                    }
                }
                Ok(out)
            }
        }
        d.deserialize_any(Fields)
    }
}
#[derive(Default)]
struct Manifest(Vec<FilmCacheMetadata>);
impl<'de> Deserialize<'de> for Manifest {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Fields;
        impl<'de> Visitor<'de> for Fields {
            type Value = Manifest;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a manifest object or null")
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(Manifest::default())
            }
            fn visit_map<M: MapAccess<'de>>(self, mut m: M) -> Result<Self::Value, M::Error> {
                let mut out = Manifest::default();
                while let Some(k) = m.next_key::<String>()? {
                    if k.eq_ignore_ascii_case("chunks") {
                        out.0 = m
                            .next_value::<Option<Vec<FilmCacheMetadata>>>()?
                            .unwrap_or_default();
                    } else {
                        m.next_value::<serde::de::IgnoredAny>()?;
                    }
                }
                Ok(out)
            }
        }
        d.deserialize_any(Fields)
    }
}

// filepath.Join cleans lexical dot segments and appends later absolute-looking
// elements; Path::join would instead discard the earlier cache root.
fn cache_join(root: &Path, suffix: &str) -> PathBuf {
    let mut joined = root.as_os_str().to_os_string();
    if !joined.is_empty() {
        joined.push(std::path::MAIN_SEPARATOR_STR);
    }
    joined.push(suffix);
    cache_clean(Path::new(&joined))
}
fn cache_clean(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => (),
            Component::ParentDir => {
                if out.file_name().is_some_and(|n| n != "..") {
                    out.pop();
                } else if !out.has_root() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    if out.as_os_str().is_empty() {
        out.push(".");
    }
    out
}
fn cache_parent(path: &Path) -> &Path {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| {
            if path.has_root() {
                Path::new(std::path::MAIN_SEPARATOR_STR)
            } else {
                Path::new(".")
            }
        })
}

pub fn film_cache_chunks_root(root: impl AsRef<Path>) -> PathBuf {
    cache_join(root.as_ref(), "film_chunks")
}
pub fn film_cache_manifests_root(root: impl AsRef<Path>) -> PathBuf {
    cache_join(root.as_ref(), "film_manifests")
}
pub fn film_cache_chunk_dir(root: impl AsRef<Path>, short_id: &str) -> PathBuf {
    cache_join(&film_cache_chunks_root(root), short_id)
}
pub fn film_cache_manifest_path(root: impl AsRef<Path>, short_id: &str) -> PathBuf {
    cache_join(
        &film_cache_manifests_root(root),
        &format!("{short_id}.json"),
    )
}

/// Manifest-order source. A missing chunk is an error here; `load` instead uses
/// existing chunk files and joins metadata by native chunk number.
#[derive(Debug, Clone)]
pub struct FilmCacheSource {
    directory: PathBuf,
    metadata: Vec<FilmCacheMetadata>,
    /// Exact source JSON, including unknown keys and original formatting.
    pub manifest_bytes: Vec<u8>,
}
impl FilmCacheSource {
    pub fn open(root: impl AsRef<Path>, short_id: &str) -> Result<Option<Self>, FilmCacheError> {
        let bytes = match std::fs::read(film_cache_manifest_path(&root, short_id)) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let manifest: Manifest = serde_json::from_slice(&bytes)?;
        Ok(Some(Self {
            directory: film_cache_chunk_dir(root, short_id),
            metadata: manifest.0,
            manifest_bytes: bytes,
        }))
    }
    /// Native OpenChunkDir: clean the path, take its basename as the cache ID,
    /// and locate the cache root two directories above it.
    pub fn open_chunk_directory(
        directory: impl AsRef<Path>,
    ) -> Result<Option<Self>, FilmCacheError> {
        let directory = cache_clean(directory.as_ref());
        let name = directory
            .file_name()
            .unwrap_or(directory.as_os_str())
            .to_str()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "cache ID is not UTF-8")
            })?;
        Self::open(cache_parent(cache_parent(&directory)), name)
    }
    pub fn load_cache(
        root: impl AsRef<Path>,
        short_id: &str,
    ) -> Result<Option<FilmSource>, FilmCacheError> {
        Self::open(root, short_id)?
            .map(|source| source.load())
            .transpose()
    }
    pub fn load_chunk_directory(
        directory: impl AsRef<Path>,
    ) -> Result<Option<FilmSource>, FilmCacheError> {
        Self::open_chunk_directory(directory)?
            .map(|source| source.load())
            .transpose()
    }
    pub fn metadata(&self) -> &[FilmCacheMetadata] {
        &self.metadata
    }
    pub fn num_chunks(&self) -> usize {
        self.metadata.len()
    }
    pub fn chunk(&self, position: usize) -> Result<Vec<u8>, FilmCacheError> {
        let m = self.metadata.get(position).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "chunk position outside manifest",
            )
        })?;
        Ok(std::fs::read(
            self.directory.join(format!("chunk_{:02}.bin", m.index)),
        )?)
    }
    pub fn load(&self) -> Result<FilmSource, FilmCacheError> {
        let metadata: Vec<_> = self
            .metadata
            .iter()
            .map(|m| FilmSourceMetadata {
                index: m.index,
                chunk_type: m.chunk_type,
                start_ms: m.start_ms,
            })
            .collect();
        Ok(FilmSource::load_directory(&self.directory, &metadata)?)
    }
}

pub fn list_film_cache_ids(root: impl AsRef<Path>) -> Result<Vec<String>, FilmCacheError> {
    let entries = match std::fs::read_dir(film_cache_manifests_root(root)) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e.into()),
    };
    let mut out = Vec::new();
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir()
            && let Some(name) = entry
                .file_name()
                .to_str()
                .and_then(|s| s.strip_suffix(".json"))
        {
            out.push(name.to_owned());
        }
    }
    out.sort();
    Ok(out)
}

/// Raw bytes and metadata to archive. Bytes are persisted verbatim, not encoded
/// from decoded LegacyFilm structures.
#[derive(Debug, Clone, Serialize)]
pub struct FilmCacheWriteChunk {
    pub index: i64,
    pub chunk_type: i64,
    pub start_ms: i64,
    pub duration_ms: i64,
    #[serde(skip)]
    pub data: Vec<u8>,
}

fn cache_create_dirs(path: impl AsRef<Path>) -> std::io::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o755);
    }
    builder.create(path)
}
fn cache_write(path: impl AsRef<Path>, data: impl AsRef<[u8]>) -> std::io::Result<()> {
    use std::io::Write;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o644);
    }
    options.open(path)?.write_all(data.as_ref())
}

pub fn ensure_film_cache_dirs(root: impl AsRef<Path>) -> Result<(), FilmCacheError> {
    let root = root.as_ref();
    if root.as_os_str().to_string_lossy().trim().is_empty() {
        return Err(
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "empty film cache root").into(),
        );
    }
    cache_create_dirs(film_cache_chunks_root(root))?;
    cache_create_dirs(film_cache_manifests_root(root))?;
    Ok(())
}

/// Native cache writer: preserve existing paths, write missing raw chunks in
/// input order, then create the manifest if absent. This is not an atomic
/// transaction: an I/O failure can leave chunks without a manifest, and an
/// interrupted manifest write can leave partial JSON, as in the reference.
pub fn write_film_cache(
    root: impl AsRef<Path>,
    short_id: &str,
    chunks: &[FilmCacheWriteChunk],
) -> Result<(), FilmCacheError> {
    if chunks.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "no film chunks to write",
        )
        .into());
    }
    let directory = film_cache_chunk_dir(&root, short_id);
    cache_create_dirs(&directory)?;
    for chunk in chunks {
        let path = directory.join(format!("chunk_{:02}.bin", chunk.index));
        if std::fs::metadata(&path).is_ok() {
            continue;
        }
        cache_write(path, &chunk.data)?;
    }
    let path = film_cache_manifest_path(root, short_id);
    if std::fs::metadata(&path).is_ok() {
        return Ok(());
    }
    cache_create_dirs(path.parent().expect("manifest path has parent"))?;
    #[derive(Serialize)]
    struct Manifest<'a> {
        chunks: &'a [FilmCacheWriteChunk],
    }
    cache_write(path, serde_json::to_vec(&Manifest { chunks })?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[cfg(unix)]
    fn native_cache_paths() {
        let rows: Vec<serde_json::Value> =
            serde_json::from_slice(include_bytes!("fixtures/cache-paths-v41.json")).unwrap();
        for row in rows {
            let root = row["root"].as_str().unwrap();
            let id = row["id"].as_str().unwrap();
            for (key, path) in [
                ("chunks", film_cache_chunks_root(root)),
                ("manifests", film_cache_manifests_root(root)),
                ("directory", film_cache_chunk_dir(root, id)),
                ("manifest", film_cache_manifest_path(root, id)),
            ] {
                assert_eq!(
                    path.to_str().unwrap(),
                    row[key].as_str().unwrap(),
                    "{root:?}/{id:?}/{key}"
                );
            }
        }
    }
    #[test]
    #[cfg(unix)]
    fn native_cache_creation_modes() {
        use std::os::unix::fs::PermissionsExt;
        const CHILD: &str = "HALO_CACHE_MODE_TEST_MASK";
        if let Ok(mask) = std::env::var(CHILD) {
            let mask = u32::from_str_radix(&mask, 8).unwrap();
            let root = std::env::temp_dir().join(format!("halo-cache-mode-{}", std::process::id()));
            std::fs::create_dir(&root).unwrap();
            ensure_film_cache_dirs(&root).unwrap();
            write_film_cache(
                &root,
                "test",
                &[FilmCacheWriteChunk {
                    index: 1,
                    chunk_type: 2,
                    start_ms: 0,
                    duration_ms: 0,
                    data: vec![1],
                }],
            )
            .unwrap();
            for path in [
                film_cache_chunks_root(&root),
                film_cache_manifests_root(&root),
                film_cache_chunk_dir(&root, "test"),
            ] {
                assert_eq!(
                    std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
                    0o755 & !mask
                );
            }
            for path in [
                film_cache_manifest_path(&root, "test"),
                film_cache_chunk_dir(&root, "test").join("chunk_01.bin"),
            ] {
                assert_eq!(
                    std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
                    0o644 & !mask
                );
            }
            std::fs::remove_dir_all(root).unwrap();
            return;
        }
        // Keep umask changes in subprocesses; other tests run concurrently.
        for mask in ["000", "077"] {
            let status = std::process::Command::new("/bin/sh")
                .args(["-c", "umask \"$1\"; shift; exec \"$@\"", "sh", mask])
                .arg(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "theater::film_cache::tests::native_cache_creation_modes",
                    "--nocapture",
                ])
                .env(CHILD, mask)
                .status()
                .unwrap();
            assert!(status.success());
        }
    }
    #[test]
    fn native_cache_writer_snapshots() {
        let cases: Vec<serde_json::Value> =
            serde_json::from_slice(include_bytes!("fixtures/cache-writer-v41.json")).unwrap();
        for case in cases {
            let mode = case["mode"].as_u64().unwrap();
            let root = std::env::temp_dir()
                .join(format!("halo-cache-write-{}-{mode}", std::process::id()));
            std::fs::create_dir(&root).unwrap();
            ensure_film_cache_dirs(&root).unwrap();
            let dir = film_cache_chunk_dir(&root, "test");
            match mode {
                3 => std::fs::write(
                    film_cache_manifest_path(&root, "test"),
                    br#"{"blob_prefix":"keep"}"#,
                )
                .unwrap(),
                4 => {
                    std::fs::create_dir_all(&dir).unwrap();
                    std::fs::write(dir.join("chunk_04.bin"), b"old").unwrap();
                }
                5 => std::fs::write(&dir, b"block").unwrap(),
                6 => {
                    std::fs::remove_dir(film_cache_manifests_root(&root)).unwrap();
                    std::fs::write(film_cache_manifests_root(&root), b"block").unwrap();
                }
                7 => std::fs::create_dir_all(dir.join("chunk_04.bin")).unwrap(),
                _ => (),
            }
            let mut chunks = vec![
                FilmCacheWriteChunk {
                    index: 4,
                    chunk_type: 2,
                    start_ms: 123,
                    duration_ms: 456,
                    data: vec![0, 255, 1],
                },
                FilmCacheWriteChunk {
                    index: 1,
                    chunk_type: 0,
                    start_ms: 0,
                    duration_ms: 0,
                    data: b"second".to_vec(),
                },
            ];
            if mode == 0 {
                chunks.clear();
            }
            if mode == 2 {
                chunks.push(FilmCacheWriteChunk {
                    index: 4,
                    chunk_type: 3,
                    start_ms: -9,
                    duration_ms: 7,
                    data: b"replacement".to_vec(),
                });
            }
            assert_eq!(
                write_film_cache(&root, "test", &chunks).is_err(),
                case["error"].as_bool().unwrap(),
                "mode {mode}"
            );
            for expected in case["snapshot"].as_array().unwrap() {
                let path = root.join(expected["path"].as_str().unwrap());
                let metadata = std::fs::metadata(&path);
                let kind = match &metadata {
                    Ok(m) if m.is_dir() => "directory",
                    Ok(_) => "file",
                    Err(_) => "missing",
                };
                assert_eq!(
                    kind,
                    expected["kind"],
                    "mode {mode} path {}",
                    path.display()
                );
                if kind == "file" {
                    let hex = expected["data"].as_str().unwrap();
                    let bytes: Vec<u8> = (0..hex.len())
                        .step_by(2)
                        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                        .collect();
                    assert_eq!(
                        std::fs::read(&path).unwrap(),
                        bytes,
                        "mode {mode} path {}",
                        path.display()
                    );
                }
            }
            std::fs::remove_dir_all(root).unwrap();
        }
        assert!(ensure_film_cache_dirs("   ").is_err());
    }
    #[test]
    fn native_cache_manifest_contract() {
        let cases: Vec<serde_json::Value> =
            serde_json::from_slice(include_bytes!("fixtures/cache-reader-v41.json")).unwrap();
        for (i, case) in cases.iter().enumerate() {
            let got = serde_json::from_str::<Manifest>(case["json"].as_str().unwrap());
            assert_eq!(got.is_err(), case["error"].as_bool().unwrap(), "case {i}");
            if let Ok(got) = got {
                let metadata: Vec<_> = got.0.iter().map(|m| serde_json::json!({"index":m.index,"chunk_type":m.chunk_type,"start_ms":m.start_ms})).collect();
                assert_eq!(serde_json::json!(metadata), case["metadata"], "case {i}");
            }
        }
    }
    #[test]
    fn native_cache_partial_metadata() {
        let rows: Vec<serde_json::Value> =
            serde_json::from_slice(include_bytes!("fixtures/cache-partial-metadata-v41.json"))
                .unwrap();
        let root =
            std::env::temp_dir().join(format!("halo-cache-partial-meta-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir(film_cache_manifests_root(&root)).unwrap();
        std::fs::create_dir_all(film_cache_chunk_dir(&root, "test")).unwrap();
        for (name, bytes) in [
            ("chunk_04.bin", b"raw".as_slice()),
            ("chunk_unknown.bin", b"opaque".as_slice()),
        ] {
            std::fs::write(film_cache_chunk_dir(&root, "test").join(name), bytes).unwrap();
        }
        for (i, row) in rows.iter().enumerate() {
            std::fs::write(
                film_cache_manifest_path(&root, "test"),
                row["json"].as_str().unwrap(),
            )
            .unwrap();
            let cache = FilmCacheSource::open(&root, "test").unwrap().unwrap();
            assert_eq!(
                cache.manifest_bytes,
                row["json"].as_str().unwrap().as_bytes()
            );
            let source = cache.load().unwrap();
            let metadata_json = serde_json::to_vec(source.metadata()).unwrap();
            let restored: Vec<FilmSourceMetadata> = serde_json::from_slice(&metadata_json).unwrap();
            assert_eq!(restored, source.metadata());
            assert_eq!(
                serde_json::to_value(source.metadata()).unwrap(),
                row["metadata"],
                "case {i}"
            );
            for (position, expected) in row["chunks"].as_array().unwrap().iter().enumerate() {
                let bytes = source.chunk(position).unwrap();
                let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
                assert_eq!(hex, expected.as_str().unwrap());
            }
            let selected_type = source.metadata()[0].chunk_type;
            let converted = source.into_chunks();
            match i32::try_from(selected_type) {
                Ok(expected) => assert_eq!(converted.unwrap()[0].metadata.chunk_type, expected),
                Err(_) => assert!(
                    matches!(converted, Err(FilmSourceError::ChunkType(value)) if value == selected_type)
                ),
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cache_source_preserves_manifest_order_and_partial_file_loading() {
        let root = std::env::temp_dir().join(format!("halo-cache-reader-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        assert!(FilmCacheSource::open(&root, "test").unwrap().is_none());
        assert!(list_film_cache_ids(&root).unwrap().is_empty());
        std::fs::create_dir(film_cache_manifests_root(&root)).unwrap();
        std::fs::create_dir_all(film_cache_chunk_dir(&root, "test")).unwrap();
        let raw = br#"{"blob_prefix":"preserve","chunks":[{"index":4,"chunk_type":2,"start_ms":900},{"index":1}]}"#;
        std::fs::write(film_cache_manifest_path(&root, "test"), raw).unwrap();
        std::fs::write(
            film_cache_chunk_dir(&root, "test").join("chunk_04.bin"),
            b"raw",
        )
        .unwrap();
        std::fs::write(
            film_cache_chunk_dir(&root, "test").join("chunk_02.bin"),
            b"extra",
        )
        .unwrap();
        let cache = FilmCacheSource::open(&root, "test").unwrap().unwrap();
        assert_eq!(cache.manifest_bytes, raw);
        assert_eq!(cache.num_chunks(), 2);
        let directory = film_cache_chunk_dir(&root, "test").join("../test/.");
        let reopened = FilmCacheSource::open_chunk_directory(&directory)
            .unwrap()
            .unwrap();
        assert_eq!(reopened.metadata(), cache.metadata());
        assert_eq!(
            FilmCacheSource::load_chunk_directory(&directory)
                .unwrap()
                .unwrap()
                .metadata(),
            cache.load().unwrap().metadata()
        );
        assert_eq!(
            FilmCacheSource::load_cache(&root, "test")
                .unwrap()
                .unwrap()
                .metadata(),
            cache.load().unwrap().metadata()
        );
        assert!(
            FilmCacheSource::load_cache(&root, "absent")
                .unwrap()
                .is_none()
        );

        assert_eq!(cache.chunk(0).unwrap(), b"raw");
        assert!(cache.chunk(1).is_err());
        assert!(cache.chunk(2).is_err());
        let loaded = cache.load().unwrap();
        assert_eq!(
            loaded.metadata(),
            &[
                FilmSourceMetadata {
                    index: 2,
                    ..Default::default()
                },
                FilmSourceMetadata {
                    index: 4,
                    chunk_type: 2,
                    start_ms: 900
                }
            ]
        );
        assert_eq!(list_film_cache_ids(&root).unwrap(), vec!["test"]);
        std::fs::write(film_cache_manifest_path(&root, "bad"), b"{").unwrap();
        assert!(FilmCacheSource::open(&root, "bad").is_err());
        std::fs::remove_dir_all(&root).unwrap();
    }
}
