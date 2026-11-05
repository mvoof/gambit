//! Custom piece sets: validation, sprite-sheet slicing and the stored library.
//!
//! The frontend receives data URLs, so a custom set looks like a bundled one.

use std::fmt;
use std::fs;
use std::io::{self, Cursor};
use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use gb_model::{PieceImages, PieceSetView, SheetColumnOrder};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageFormat};

const SHEET_COLUMNS: u32 = 6;
const SHEET_ROWS: u32 = 2;
const MIN_CELL_PIXELS: u32 = 16;
const NORMALISED_PIXELS: u32 = 256;
const PIECE_LETTERS_KQRBNP: [char; 6] = ['K', 'Q', 'R', 'B', 'N', 'P'];
const PIECE_LETTERS_KQBNRP: [char; 6] = ['K', 'Q', 'B', 'N', 'R', 'P'];
const COLOR_LETTERS: [char; 2] = ['w', 'b'];
const PNG_EXTENSION: &str = "png";
const SVG_EXTENSION: &str = "svg";
const NAME_FILE: &str = "name.txt";
const PIECE_COUNT: usize = 12;

#[derive(Debug)]
pub enum PieceError {
    Io(io::Error),
    Image(image::ImageError),
    BadSheet(String),
    MissingPiece(String),
    UnknownSet(String),
}

impl fmt::Display for PieceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "piece file: {error}"),
            Self::Image(error) => write!(formatter, "piece image: {error}"),
            Self::BadSheet(detail) => write!(formatter, "bad sprite sheet: {detail}"),
            Self::MissingPiece(code) => write!(formatter, "the folder has no {code}"),
            Self::UnknownSet(id) => write!(formatter, "no piece set {id}"),
        }
    }
}

impl std::error::Error for PieceError {}

impl From<io::Error> for PieceError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<image::ImageError> for PieceError {
    fn from(error: image::ImageError) -> Self {
        Self::Image(error)
    }
}

/// A piece as stored: its code (`wK`), file extension and bytes.
struct PieceFile {
    code: String,
    extension: &'static str,
    bytes: Vec<u8>,
}

fn data_url(extension: &str, bytes: &[u8]) -> String {
    let mime = if extension == SVG_EXTENSION {
        "image/svg+xml"
    } else {
        "image/png"
    };

    format!("data:{mime};base64,{}", STANDARD.encode(bytes))
}

fn images_of(files: &[PieceFile]) -> PieceImages {
    files
        .iter()
        .map(|file| (file.code.clone(), data_url(file.extension, &file.bytes)))
        .collect()
}

fn encode_png(cell: &DynamicImage) -> Result<Vec<u8>, PieceError> {
    let normal = cell.resize_exact(NORMALISED_PIXELS, NORMALISED_PIXELS, FilterType::Lanczos3);
    let mut bytes = Cursor::new(Vec::new());

    normal.to_rgba8().write_to(&mut bytes, ImageFormat::Png)?;

    Ok(bytes.into_inner())
}

fn letters_of(order: SheetColumnOrder) -> [char; 6] {
    match order {
        SheetColumnOrder::Kqrbnp => PIECE_LETTERS_KQRBNP,
        SheetColumnOrder::Kqbnrp => PIECE_LETTERS_KQBNRP,
    }
}

/// White in the top row, black below, in the given column order.
fn slice_sheet(
    sheet: &DynamicImage,
    order: SheetColumnOrder,
) -> Result<Vec<PieceFile>, PieceError> {
    let (width, height) = sheet.dimensions();
    let cell_width = width / SHEET_COLUMNS;
    let cell_height = height / SHEET_ROWS;

    if cell_width < MIN_CELL_PIXELS || cell_height < MIN_CELL_PIXELS {
        return Err(PieceError::BadSheet(format!(
            "{width}×{height} is too small for 6×2"
        )));
    }

    if !width.is_multiple_of(SHEET_COLUMNS) || !height.is_multiple_of(SHEET_ROWS) {
        return Err(PieceError::BadSheet(format!(
            "{width}×{height} does not divide into 6×2"
        )));
    }

    let mut files = Vec::with_capacity(PIECE_COUNT);

    for (row, color) in (0..SHEET_ROWS).zip(COLOR_LETTERS) {
        for (column, letter) in (0..SHEET_COLUMNS).zip(letters_of(order)) {
            let cell = sheet.crop_imm(
                column * cell_width,
                row * cell_height,
                cell_width,
                cell_height,
            );

            files.push(PieceFile {
                code: format!("{color}{letter}"),
                extension: PNG_EXTENSION,
                bytes: encode_png(&cell)?,
            });
        }
    }

    Ok(files)
}

fn slice_sheet_file(path: &Path, order: SheetColumnOrder) -> Result<Vec<PieceFile>, PieceError> {
    let bytes = fs::read(path)?;

    slice_sheet(&image::load_from_memory(&bytes)?, order)
}

fn read_piece_file(folder: &Path, code: String) -> Result<PieceFile, PieceError> {
    let svg = folder.join(format!("{code}.{SVG_EXTENSION}"));

    if svg.exists() {
        return Ok(PieceFile {
            bytes: fs::read(svg)?,
            code,
            extension: SVG_EXTENSION,
        });
    }

    let png = folder.join(format!("{code}.{PNG_EXTENSION}"));

    if !png.exists() {
        return Err(PieceError::MissingPiece(code));
    }

    let decoded = image::load_from_memory(&fs::read(png)?)?;

    Ok(PieceFile {
        bytes: encode_png(&decoded)?,
        code,
        extension: PNG_EXTENSION,
    })
}

fn read_piece_folder(folder: &Path) -> Result<Vec<PieceFile>, PieceError> {
    COLOR_LETTERS
        .into_iter()
        .flat_map(|color| PIECE_LETTERS_KQRBNP.map(|letter| format!("{color}{letter}")))
        .map(|code| read_piece_file(folder, code))
        .collect()
}

/// The set as the import would store it, for the preview before saving.
pub fn preview_sheet(path: &Path, order: SheetColumnOrder) -> Result<PieceImages, PieceError> {
    Ok(images_of(&slice_sheet_file(path, order)?))
}

/// Lower-case letters, digits and dashes: safe as a folder name.
fn slug(name: &str) -> String {
    let joined: String = name
        .chars()
        .map(|letter| {
            if letter.is_ascii_alphanumeric() {
                letter.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = joined.trim_matches('-');

    if trimmed.is_empty() {
        "custom".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn is_safe_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|letter| letter.is_ascii_alphanumeric() || letter == '-')
}

/// The stored custom sets: a folder per set.
pub struct PieceLibrary {
    root: PathBuf,
}

impl PieceLibrary {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn free_id(&self, name: &str) -> String {
        let base = slug(name);
        let mut id = base.clone();
        let mut suffix = 2;

        while self.root.join(&id).exists() {
            id = format!("{base}-{suffix}");
            suffix += 1;
        }

        id
    }

    fn store(&self, name: &str, files: &[PieceFile]) -> Result<PieceSetView, PieceError> {
        let id = self.free_id(name);
        let folder = self.root.join(&id);

        fs::create_dir_all(&folder)?;
        fs::write(folder.join(NAME_FILE), name)?;

        for file in files {
            fs::write(
                folder.join(format!("{}.{}", file.code, file.extension)),
                &file.bytes,
            )?;
        }

        Ok(PieceSetView {
            id,
            name: name.to_owned(),
            images: images_of(files),
        })
    }

    pub fn import_sheet(
        &self,
        name: &str,
        path: &Path,
        order: SheetColumnOrder,
    ) -> Result<PieceSetView, PieceError> {
        self.store(name, &slice_sheet_file(path, order)?)
    }

    pub fn import_folder(&self, name: &str, folder: &Path) -> Result<PieceSetView, PieceError> {
        self.store(name, &read_piece_folder(folder)?)
    }

    fn read_set(&self, folder: &Path, id: String) -> Result<PieceSetView, PieceError> {
        let name = fs::read_to_string(folder.join(NAME_FILE))?;
        let files = read_stored_files(folder)?;

        Ok(PieceSetView {
            id,
            name,
            images: images_of(&files),
        })
    }

    pub fn list(&self) -> Result<Vec<PieceSetView>, PieceError> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }

        let mut sets = Vec::new();

        for entry in fs::read_dir(&self.root)? {
            let folder = entry?.path();
            let id = folder
                .file_name()
                .map(|name| name.to_string_lossy().into_owned());

            if let (true, Some(id)) = (folder.is_dir(), id) {
                sets.push(self.read_set(&folder, id)?);
            }
        }

        sets.sort_by(|left, right| left.name.cmp(&right.name));

        Ok(sets)
    }

    pub fn delete(&self, id: &str) -> Result<(), PieceError> {
        let folder = self.root.join(id);

        if !is_safe_id(id) || !folder.is_dir() {
            return Err(PieceError::UnknownSet(id.to_owned()));
        }

        fs::remove_dir_all(folder)?;

        Ok(())
    }
}

fn read_stored_files(folder: &Path) -> Result<Vec<PieceFile>, PieceError> {
    let codes = COLOR_LETTERS
        .into_iter()
        .flat_map(|color| PIECE_LETTERS_KQRBNP.map(|letter| format!("{color}{letter}")));
    let mut files = Vec::with_capacity(PIECE_COUNT);

    for code in codes {
        let extension = if folder.join(format!("{code}.{SVG_EXTENSION}")).exists() {
            SVG_EXTENSION
        } else {
            PNG_EXTENSION
        };

        files.push(PieceFile {
            bytes: fs::read(folder.join(format!("{code}.{extension}")))?,
            code,
            extension,
        });
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use image::{Rgba, RgbaImage};
    use tempfile::tempdir;

    use super::*;

    const CELL: u32 = 20;
    const FIRST_COLUMN_WHITE: [u8; 4] = [10, 0, 0, 255];

    /// Every cell a different colour, to see which cell went where.
    fn sheet_image() -> DynamicImage {
        let image = RgbaImage::from_fn(CELL * SHEET_COLUMNS, CELL * SHEET_ROWS, |x, y| {
            let column = u8::try_from(x / CELL).unwrap();
            let row = u8::try_from(y / CELL).unwrap();

            Rgba([(column + 1) * 10, row * 100, 0, 255])
        });

        DynamicImage::ImageRgba8(image)
    }

    fn write_sheet(folder: &Path) -> PathBuf {
        let path = folder.join("sheet.png");

        sheet_image().save(&path).unwrap();

        path
    }

    fn file_of<'a>(files: &'a [PieceFile], code: &str) -> &'a PieceFile {
        files.iter().find(|file| file.code == code).unwrap()
    }

    fn center_pixel(file: &PieceFile) -> [u8; 4] {
        let decoded = image::load_from_memory(&file.bytes).unwrap();

        decoded
            .get_pixel(NORMALISED_PIXELS / 2, NORMALISED_PIXELS / 2)
            .0
    }

    #[test]
    fn a_sheet_slices_into_twelve_normal_pieces() {
        let files = slice_sheet(&sheet_image(), SheetColumnOrder::Kqrbnp).unwrap();

        assert_eq!(files.len(), PIECE_COUNT);

        let king = image::load_from_memory(&file_of(&files, "wK").bytes).unwrap();

        assert_eq!(king.dimensions(), (NORMALISED_PIXELS, NORMALISED_PIXELS));
    }

    #[test]
    fn the_column_order_decides_which_cell_is_which() {
        let first = slice_sheet(&sheet_image(), SheetColumnOrder::Kqrbnp).unwrap();
        let second = slice_sheet(&sheet_image(), SheetColumnOrder::Kqbnrp).unwrap();

        assert_eq!(center_pixel(file_of(&first, "wK")), FIRST_COLUMN_WHITE);
        assert_eq!(center_pixel(file_of(&first, "wR")), [30, 0, 0, 255]);
        assert_eq!(center_pixel(file_of(&second, "wR")), [50, 0, 0, 255]);
        assert_eq!(center_pixel(file_of(&second, "bK")), [10, 100, 0, 255]);
    }

    #[test]
    fn a_sheet_that_does_not_divide_is_refused() {
        let wrong = DynamicImage::ImageRgba8(RgbaImage::new(121, 40));

        assert!(matches!(
            slice_sheet(&wrong, SheetColumnOrder::Kqrbnp),
            Err(PieceError::BadSheet(_))
        ));
    }

    #[test]
    fn a_tiny_sheet_is_refused() {
        let tiny = DynamicImage::ImageRgba8(RgbaImage::new(12, 4));

        assert!(matches!(
            slice_sheet(&tiny, SheetColumnOrder::Kqrbnp),
            Err(PieceError::BadSheet(_))
        ));
    }

    #[test]
    fn an_imported_sheet_is_listed_and_deleted() {
        let folder = tempdir().unwrap();
        let sheet = write_sheet(folder.path());
        let library = PieceLibrary::new(folder.path().join("sets"));

        let imported = library
            .import_sheet("My Set!", &sheet, SheetColumnOrder::Kqrbnp)
            .unwrap();

        assert_eq!(imported.id, "my-set");
        assert!(imported.images["bP"].starts_with("data:image/png;base64,"));
        assert_eq!(library.list().unwrap(), vec![imported.clone()]);

        library.delete(&imported.id).unwrap();

        assert!(library.list().unwrap().is_empty());
    }

    #[test]
    fn two_sets_with_one_name_get_their_own_folders() {
        let folder = tempdir().unwrap();
        let sheet = write_sheet(folder.path());
        let library = PieceLibrary::new(folder.path().join("sets"));
        let order = SheetColumnOrder::Kqrbnp;

        library.import_sheet("Set", &sheet, order).unwrap();

        assert_eq!(
            library.import_sheet("Set", &sheet, order).unwrap().id,
            "set-2"
        );
    }

    #[test]
    fn a_folder_import_keeps_svg_and_needs_every_piece() {
        let folder = tempdir().unwrap();
        let source = folder.path().join("source");
        let library = PieceLibrary::new(folder.path().join("sets"));

        fs::create_dir_all(&source).unwrap();

        for color in COLOR_LETTERS {
            for letter in PIECE_LETTERS_KQRBNP {
                fs::write(source.join(format!("{color}{letter}.svg")), "<svg/>").unwrap();
            }
        }

        fs::remove_file(source.join("bP.svg")).unwrap();

        assert!(matches!(
            library.import_folder("Svg", &source),
            Err(PieceError::MissingPiece(_))
        ));

        fs::write(source.join("bP.svg"), "<svg/>").unwrap();

        let imported = library.import_folder("Svg", &source).unwrap();

        assert!(imported.images["wK"].starts_with("data:image/svg+xml;base64,"));
        assert_eq!(library.list().unwrap().len(), 1);
    }

    #[test]
    fn delete_refuses_a_path_outside_the_library() {
        let folder = tempdir().unwrap();
        let library = PieceLibrary::new(folder.path().join("sets"));

        fs::create_dir_all(folder.path().join("sets")).unwrap();

        assert!(matches!(
            library.delete(".."),
            Err(PieceError::UnknownSet(_))
        ));
        assert!(folder.path().exists());
    }
}
