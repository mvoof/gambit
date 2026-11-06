//! Engines on demand: pinned releases, never in the installer.

use std::fs::{self, File};
use std::io::{self, Read};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use futures_util::StreamExt;
use gb_model::{CatalogEntry, EngineInstallProgress, EngineInstallStage};
use sha2::{Digest, Sha256};
use tar::Archive;
use tokio::io::AsyncWriteExt;
use tokio::task::spawn_blocking;
use zip::ZipArchive;

use crate::catalog::{
    catalog_engine, has_avx2, CatalogEngine, Package, PackageFormat, Platform, CATALOG,
};
use crate::error::EngineError;

const PARTIAL_EXTENSION: &str = "part";
const WINDOWS_ENGINE_EXTENSION: &str = "exe";
const HASH_BUFFER_BYTES: usize = 1 << 16;
#[cfg(unix)]
const EXECUTABLE_MODE: u32 = 0o755;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn verify_archive(path: &Path, size: u32, sha256: &str) -> Result<(), EngineError> {
    let actual_size = fs::metadata(path)?.len();

    if actual_size != u64::from(size) {
        return Err(EngineError::Corrupt(format!(
            "{actual_size} bytes, expected {size}"
        )));
    }

    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; HASH_BUFFER_BYTES];

    loop {
        let read = file.read(&mut buffer)?;

        if read == 0 {
            break;
        }

        hasher.update(&buffer[..read]);
    }

    let digest = hex(hasher.finalize().as_slice());

    if digest != sha256 {
        return Err(EngineError::Corrupt(format!("sha256 {digest}")));
    }

    Ok(())
}

/// Unpacks everything: a release may ship files next to the executable.
fn extract_zip(archive_path: &Path, target: &Path) -> Result<(), EngineError> {
    let archive_error = |error: zip::result::ZipError| EngineError::Archive(error.to_string());
    let mut archive = ZipArchive::new(File::open(archive_path)?).map_err(archive_error)?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(archive_error)?;
        let Some(relative) = entry.enclosed_name() else {
            return Err(EngineError::Archive(format!(
                "unsafe path in entry {index}"
            )));
        };
        let destination = target.join(relative);

        if entry.is_dir() {
            fs::create_dir_all(&destination)?;
        } else {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }

            io::copy(&mut entry, &mut File::create(&destination)?)?;
        }
    }

    Ok(())
}

fn extract_tar_gz(archive_path: &Path, target: &Path) -> Result<(), EngineError> {
    let mut archive = Archive::new(GzDecoder::new(File::open(archive_path)?));

    fs::create_dir_all(target)?;

    for entry in archive.entries()? {
        let mut entry = entry?;

        if !entry.unpack_in(target)? {
            return Err(EngineError::Archive(format!(
                "unsafe path {}",
                entry.path()?.display()
            )));
        }
    }

    Ok(())
}

fn binary_name(id: &str) -> String {
    if cfg!(windows) {
        format!("{id}.{WINDOWS_ENGINE_EXTENSION}")
    } else {
        id.to_owned()
    }
}

/// The release's own file name is not kept: in `PlentyChess-8.0.0-linux-avx2`
/// everything after the last dot would read as an extension.
fn place_binary(archive_path: &Path, target: &Path, id: &str) -> Result<(), EngineError> {
    fs::create_dir_all(target)?;
    fs::copy(archive_path, target.join(binary_name(id)))?;

    Ok(())
}

fn unpack(
    format: PackageFormat,
    archive_path: &Path,
    target: &Path,
    id: &str,
) -> Result<(), EngineError> {
    match format {
        PackageFormat::Binary => place_binary(archive_path, target, id),
        PackageFormat::Zip => extract_zip(archive_path, target),
        PackageFormat::TarGz => extract_tar_gz(archive_path, target),
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) -> io::Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(EXECUTABLE_MODE))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> io::Result<()> {
    Ok(())
}

/// A Linux executable has no extension; a release's notes and sources do.
fn looks_executable(path: &Path) -> bool {
    if cfg!(windows) {
        path.extension()
            .is_some_and(|extension| extension == WINDOWS_ENGINE_EXTENSION)
    } else {
        path.extension().is_none()
    }
}

fn collect_executables(folder: &Path, found: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(folder)? {
        let path = entry?.path();

        if path.is_dir() {
            collect_executables(&path, found)?;
        } else if looks_executable(&path) {
            found.push(path);
        }
    }

    Ok(())
}

/// The installed executable, if any: the largest one, should there be several.
pub fn installed_engine_path(data_dir: &Path, id: &str) -> Option<PathBuf> {
    let mut found = Vec::new();

    collect_executables(&data_dir.join(id), &mut found).ok()?;

    found
        .into_iter()
        .max_by_key(|path| fs::metadata(path).map_or(0, |metadata| metadata.len()))
}

/// The first engine of the catalogue found on disk.
pub fn first_installed(data_dir: &Path) -> Option<PathBuf> {
    CATALOG
        .iter()
        .find_map(|engine| installed_engine_path(data_dir, engine.id))
}

fn catalog_entry(engine: &CatalogEngine, package: &Package, data_dir: &Path) -> CatalogEntry {
    CatalogEntry {
        id: engine.id.to_owned(),
        name: engine.name.to_owned(),
        version: engine.version.to_owned(),
        licence: engine.licence.to_owned(),
        homepage: engine.homepage.to_owned(),
        size_bytes: package.bytes,
        path: installed_engine_path(data_dir, engine.id)
            .map(|path| path.to_string_lossy().into_owned()),
    }
}

/// What can be installed on this machine.
pub fn catalog_entries(data_dir: &Path) -> Vec<CatalogEntry> {
    let Some(platform) = Platform::current() else {
        return Vec::new();
    };
    let avx2 = has_avx2();

    CATALOG
        .iter()
        .filter_map(|engine| {
            engine
                .package(platform, avx2)
                .map(|package| catalog_entry(engine, package, data_dir))
        })
        .collect()
}

/// Deletes the engine's folder. Keeping the engine in use is the caller's job.
pub fn remove_engine(data_dir: &Path, id: &str) -> Result<(), EngineError> {
    let engine = catalog_engine(id).ok_or_else(|| EngineError::Unavailable(id.to_owned()))?;
    let folder = data_dir.join(engine.id);

    if folder.exists() {
        fs::remove_dir_all(folder)?;
    }

    Ok(())
}

async fn download(
    url: &str,
    total_bytes: u32,
    target: &Path,
    on_progress: &impl Fn(EngineInstallProgress),
) -> Result<(), EngineError> {
    let download_error = |error: reqwest::Error| EngineError::Download(error.to_string());
    let response = reqwest::get(url)
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(download_error)?;
    let mut stream = response.bytes_stream();
    let mut file = tokio::fs::File::create(target).await?;
    let mut downloaded: u32 = 0;

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(download_error)?;

        file.write_all(&bytes).await?;
        downloaded = downloaded.saturating_add(u32::try_from(bytes.len()).unwrap_or(u32::MAX));

        on_progress(EngineInstallProgress {
            stage: EngineInstallStage::Downloading,
            downloaded_bytes: downloaded,
            total_bytes,
        });
    }

    file.flush().await?;

    Ok(())
}

fn stage(stage: EngineInstallStage, total_bytes: u32) -> EngineInstallProgress {
    EngineInstallProgress {
        stage,
        downloaded_bytes: total_bytes,
        total_bytes,
    }
}

async fn run_blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, EngineError> + Send + 'static,
) -> Result<T, EngineError> {
    spawn_blocking(work)
        .await
        .map_err(|error| EngineError::Archive(error.to_string()))?
}

/// Downloads, verifies and unpacks an engine of the catalogue into `data_dir`.
pub async fn install_engine(
    data_dir: &Path,
    id: &str,
    on_progress: impl Fn(EngineInstallProgress),
) -> Result<PathBuf, EngineError> {
    let unavailable = || EngineError::Unavailable(id.to_owned());
    let engine = catalog_engine(id).ok_or_else(unavailable)?;
    let platform = Platform::current().ok_or_else(unavailable)?;
    let package = engine
        .package(platform, has_avx2())
        .ok_or_else(unavailable)?;
    let total_bytes = package.bytes;

    tokio::fs::create_dir_all(data_dir).await?;

    let target = data_dir.join(engine.id);
    let archive = target.with_extension(PARTIAL_EXTENSION);

    download(package.url, total_bytes, &archive, &on_progress).await?;
    on_progress(stage(EngineInstallStage::Verifying, total_bytes));

    let checked = archive.clone();
    let sha256 = package.sha256;

    run_blocking(move || verify_archive(&checked, total_bytes, sha256)).await?;
    on_progress(stage(EngineInstallStage::Extracting, total_bytes));

    let unpacked_from = archive.clone();
    let unpacked_to = target.clone();
    let format = package.format;
    let engine_id = engine.id;

    run_blocking(move || unpack(format, &unpacked_from, &unpacked_to, engine_id)).await?;
    tokio::fs::remove_file(&archive).await?;

    let installed = installed_engine_path(data_dir, engine.id)
        .ok_or_else(|| EngineError::Archive("no executable in the archive".to_owned()))?;

    make_executable(&installed)?;
    on_progress(stage(EngineInstallStage::Done, total_bytes));

    Ok(installed)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::{Builder, Header};
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    use super::*;

    const ENGINE_ID: &str = "stockfish";
    const LONG_NOTES: &[u8] = b"long enough to be the largest file of the release";
    const TAR_FILE_MODE: u32 = 0o644;
    const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    fn write_zip(path: &Path, entries: &[(&str, &[u8])]) {
        let mut writer = ZipWriter::new(File::create(path).unwrap());

        for (name, content) in entries {
            writer
                .start_file(*name, SimpleFileOptions::default())
                .unwrap();
            writer.write_all(content).unwrap();
        }

        writer.finish().unwrap();
    }

    fn write_tar_gz(path: &Path, entries: &[(&str, &[u8])]) {
        let encoder = GzEncoder::new(File::create(path).unwrap(), Compression::fast());
        let mut builder = Builder::new(encoder);

        for (name, content) in entries {
            let mut header = Header::new_gnu();

            header.set_size(content.len() as u64);
            header.set_mode(TAR_FILE_MODE);
            header.set_cksum();
            builder.append_data(&mut header, name, *content).unwrap();
        }

        builder.into_inner().unwrap().finish().unwrap();
    }

    fn installed_name(data_dir: &Path) -> String {
        installed_engine_path(data_dir, ENGINE_ID)
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn verification_checks_size_and_hash() {
        let folder = tempdir().unwrap();
        let file = folder.path().join("empty");

        File::create(&file).unwrap();

        assert!(verify_archive(&file, 0, EMPTY_SHA256).is_ok());
        assert!(matches!(
            verify_archive(&file, 1, EMPTY_SHA256),
            Err(EngineError::Corrupt(_))
        ));
        assert!(matches!(
            verify_archive(&file, 0, "00"),
            Err(EngineError::Corrupt(_))
        ));
    }

    #[test]
    fn the_largest_executable_is_the_engine() {
        let folder = tempdir().unwrap();
        let archive = folder.path().join("engine.zip");
        let helper = format!("stockfish/{}", binary_name("helper"));
        let engine = format!("stockfish/{}", binary_name("engine"));

        write_zip(
            &archive,
            &[
                ("stockfish/README.md", LONG_NOTES),
                (&helper, b"1"),
                (&engine, b"12345"),
            ],
        );
        extract_zip(&archive, &folder.path().join(ENGINE_ID)).unwrap();

        assert_eq!(installed_name(folder.path()), binary_name("engine"));
    }

    #[test]
    fn a_tar_gz_release_is_unpacked_and_its_engine_found() {
        let folder = tempdir().unwrap();
        let archive = folder.path().join("engine.tar.gz");
        let engine = format!("stockfish/{}", binary_name("engine"));

        write_tar_gz(
            &archive,
            &[("stockfish/wiki/Notes.md", LONG_NOTES), (&engine, b"12345")],
        );
        extract_tar_gz(&archive, &folder.path().join(ENGINE_ID)).unwrap();

        assert_eq!(installed_name(folder.path()), binary_name("engine"));
    }

    #[test]
    fn a_bare_binary_gets_the_name_of_its_engine() {
        let folder = tempdir().unwrap();
        let download = folder.path().join("PlentyChess-8.0.0-linux-avx2");
        let target = folder.path().join("plentychess");

        fs::write(&download, b"12345").unwrap();
        unpack(PackageFormat::Binary, &download, &target, "plentychess").unwrap();

        assert_eq!(
            installed_engine_path(folder.path(), "plentychess"),
            Some(target.join(binary_name("plentychess")))
        );
    }

    #[cfg(unix)]
    #[test]
    fn an_installed_engine_can_be_executed() {
        let folder = tempdir().unwrap();
        let engine = folder.path().join("engine");

        fs::write(&engine, b"12345").unwrap();
        make_executable(&engine).unwrap();

        let mode = fs::metadata(&engine).unwrap().permissions().mode();

        assert_eq!(mode & EXECUTABLE_MODE, EXECUTABLE_MODE);
    }

    #[test]
    fn a_path_leaving_the_target_is_refused() {
        let folder = tempdir().unwrap();
        let archive = folder.path().join("evil.zip");

        write_zip(&archive, &[("../evil.exe", b"1")]);

        assert!(matches!(
            extract_zip(&archive, &folder.path().join(ENGINE_ID)),
            Err(EngineError::Archive(_))
        ));
        assert!(!folder.path().join("evil.exe").exists());
    }

    #[test]
    fn nothing_installed_is_none() {
        let folder = tempdir().unwrap();

        assert_eq!(installed_engine_path(folder.path(), ENGINE_ID), None);
        assert_eq!(first_installed(folder.path()), None);
    }

    #[test]
    fn the_catalogue_says_what_is_installed() {
        let folder = tempdir().unwrap();
        let target = folder.path().join(ENGINE_ID);
        let binary = target.join(binary_name(ENGINE_ID));

        fs::create_dir_all(&target).unwrap();
        fs::write(&binary, b"1").unwrap();

        let entries = catalog_entries(folder.path());
        let stockfish = entries.iter().find(|entry| entry.id == ENGINE_ID).unwrap();
        let mut others = entries.iter().filter(|entry| entry.id != ENGINE_ID);

        assert_eq!(stockfish.path.as_deref(), binary.to_str());
        assert!(others.all(|entry| entry.path.is_none()));
        assert_eq!(first_installed(folder.path()), Some(binary));
    }

    #[test]
    fn a_removed_engine_is_gone_and_an_unknown_one_is_refused() {
        let folder = tempdir().unwrap();
        let target = folder.path().join(ENGINE_ID);

        fs::create_dir_all(&target).unwrap();
        fs::write(target.join(binary_name(ENGINE_ID)), b"1").unwrap();

        remove_engine(folder.path(), ENGINE_ID).unwrap();

        assert_eq!(installed_engine_path(folder.path(), ENGINE_ID), None);
        assert!(matches!(
            remove_engine(folder.path(), "../outside"),
            Err(EngineError::Unavailable(_))
        ));
    }
}
