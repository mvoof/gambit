//! Stockfish on demand: the pinned release, never in the installer.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use gb_model::{EngineInstallProgress, EngineInstallStage};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tokio::task::spawn_blocking;
use zip::ZipArchive;

use crate::error::EngineError;

const STOCKFISH_URL: &str = "https://github.com/official-stockfish/Stockfish/releases/download/sf_19/stockfish-windows-x86-64-universal.zip";
const STOCKFISH_BYTES: u32 = 81_431_614;
const STOCKFISH_SHA256: &str = "3c8bf1f9ea66a09350a40df4f632288285ac206d99f33ab5842c408fc30b48a7";
const ARCHIVE_NAME: &str = "stockfish.zip.part";
const ENGINE_FOLDER: &str = "stockfish";
const ENGINE_EXTENSION: &str = "exe";
const HASH_BUFFER_BYTES: usize = 1 << 16;

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
fn extract_archive(archive_path: &Path, target: &Path) -> Result<(), EngineError> {
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

fn collect_executables(folder: &Path, found: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(folder)? {
        let path = entry?.path();

        if path.is_dir() {
            collect_executables(&path, found)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == ENGINE_EXTENSION)
        {
            found.push(path);
        }
    }

    Ok(())
}

/// The installed executable, if any: the largest one, should there be several.
pub fn stockfish_path(data_dir: &Path) -> Option<PathBuf> {
    let mut found = Vec::new();

    collect_executables(&data_dir.join(ENGINE_FOLDER), &mut found).ok()?;

    found
        .into_iter()
        .max_by_key(|path| fs::metadata(path).map_or(0, |metadata| metadata.len()))
}

async fn download(
    url: &str,
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
            total_bytes: STOCKFISH_BYTES,
        });
    }

    file.flush().await?;

    Ok(())
}

fn stage(stage: EngineInstallStage) -> EngineInstallProgress {
    EngineInstallProgress {
        stage,
        downloaded_bytes: STOCKFISH_BYTES,
        total_bytes: STOCKFISH_BYTES,
    }
}

async fn run_blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, EngineError> + Send + 'static,
) -> Result<T, EngineError> {
    spawn_blocking(work)
        .await
        .map_err(|error| EngineError::Archive(error.to_string()))?
}

/// Downloads, verifies and unpacks Stockfish into `data_dir`.
pub async fn install_stockfish(
    data_dir: &Path,
    on_progress: impl Fn(EngineInstallProgress),
) -> Result<PathBuf, EngineError> {
    tokio::fs::create_dir_all(data_dir).await?;

    let archive = data_dir.join(ARCHIVE_NAME);
    let target = data_dir.join(ENGINE_FOLDER);

    download(STOCKFISH_URL, &archive, &on_progress).await?;
    on_progress(stage(EngineInstallStage::Verifying));

    let checked = archive.clone();

    run_blocking(move || verify_archive(&checked, STOCKFISH_BYTES, STOCKFISH_SHA256)).await?;
    on_progress(stage(EngineInstallStage::Extracting));

    let unpacked_from = archive.clone();
    let unpacked_to = target.clone();

    run_blocking(move || extract_archive(&unpacked_from, &unpacked_to)).await?;
    tokio::fs::remove_file(&archive).await?;

    let installed = stockfish_path(data_dir)
        .ok_or_else(|| EngineError::Archive("no executable in the archive".to_owned()))?;

    on_progress(stage(EngineInstallStage::Done));

    Ok(installed)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    use super::*;

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
        let target = folder.path().join(ENGINE_FOLDER);

        write_zip(
            &archive,
            &[
                ("stockfish/README.md", b"text"),
                ("stockfish/helper.exe", b"1"),
                ("stockfish/engine.exe", b"12345"),
            ],
        );
        extract_archive(&archive, &target).unwrap();

        let found = stockfish_path(folder.path()).unwrap();

        assert_eq!(found.file_name().unwrap(), "engine.exe");
    }

    #[test]
    fn a_path_leaving_the_target_is_refused() {
        let folder = tempdir().unwrap();
        let archive = folder.path().join("evil.zip");

        write_zip(&archive, &[("../evil.exe", b"1")]);

        assert!(matches!(
            extract_archive(&archive, &folder.path().join(ENGINE_FOLDER)),
            Err(EngineError::Archive(_))
        ));
        assert!(!folder.path().join("evil.exe").exists());
    }

    #[test]
    fn nothing_installed_is_none() {
        let folder = tempdir().unwrap();

        assert_eq!(stockfish_path(folder.path()), None);
    }
}
