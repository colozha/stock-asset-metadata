use crate::metadata::{self, AppResult};
use image::{ImageDecoder, ImageFormat, ImageReader, Limits};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{BufRead, Cursor, Read, Seek, SeekFrom, Write},
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant, UNIX_EPOCH},
};

const MAX_SOURCE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_PREVIEW_BYTES: u64 = 128 * 1024 * 1024;
const MAX_EDGE: u32 = 1600;
// Bump when preview output changes so old cache files are ignored.
const RENDERER_VERSION: &[u8] = b"preview-renderer-v1";

pub fn read(path: &Path) -> AppResult<Vec<u8>> {
    match metadata::format(path)?.as_str() {
        "svg" => render_svg(path),
        "eps" => render_eps(path),
        "mp4" | "mov" => render_video(path),
        _ => render_raster(path),
    }
}

pub fn read_cached(path: &Path, cache_dir: &Path) -> AppResult<Vec<u8>> {
    let key = cache_key(path)?;
    let cached = cache_dir.join(format!("{key}.png"));
    if let Ok(bytes) = std::fs::read(&cached) {
        return Ok(bytes);
    }

    std::fs::create_dir_all(cache_dir).map_err(|e| e.to_string())?;
    let bytes = read(path)?;
    if cache_key(path)? != key {
        return Err("File berubah saat dibaca. Impor ulang file.".into());
    }

    let mut temporary = tempfile::NamedTempFile::new_in(cache_dir).map_err(|e| e.to_string())?;
    temporary.write_all(&bytes).map_err(|e| e.to_string())?;
    match temporary.persist(&cached) {
        Ok(_) => Ok(bytes),
        Err(_error) if cached.is_file() => std::fs::read(cached).map_err(|e| e.to_string()),
        Err(error) => Err(error.error.to_string()),
    }
}

pub fn cache_size(cache_dir: &Path) -> AppResult<u64> {
    let mut entries = match std::fs::read_dir(cache_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error.to_string()),
    };
    entries.try_fold(0u64, |total, entry| {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_file() {
            Ok(total.saturating_add(entry.metadata().map_err(|e| e.to_string())?.len()))
        } else {
            Ok(total)
        }
    })
}

pub fn clear_cache(cache_dir: &Path) -> AppResult<u64> {
    let size = cache_size(cache_dir)?;
    match std::fs::symlink_metadata(cache_dir) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            std::fs::remove_file(cache_dir).map_err(|e| e.to_string())?;
        }
        Ok(_) => std::fs::remove_dir_all(cache_dir).map_err(|e| e.to_string())?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
        Err(error) => return Err(error.to_string()),
    }
    Ok(size)
}

fn cache_key(path: &Path) -> AppResult<String> {
    let canonical = path.canonicalize().map_err(|e| e.to_string())?;
    let metadata = std::fs::metadata(&canonical).map_err(|e| e.to_string())?;
    if !metadata.is_file() {
        return Err("Sumber bukan file biasa.".into());
    }
    let modified = metadata
        .modified()
        .map_err(|e| e.to_string())?
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let mut hash = Sha256::new();
    hash.update(RENDERER_VERSION);
    hash.update(canonical.as_os_str().to_string_lossy().as_bytes());
    hash.update(metadata.len().to_le_bytes());
    hash.update(modified.as_secs().to_le_bytes());
    hash.update(modified.subsec_nanos().to_le_bytes());
    Ok(format!("{:x}", hash.finalize()))
}

fn render_raster(path: &Path) -> AppResult<Vec<u8>> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_SOURCE_BYTES {
        return Err("File terlalu besar untuk preview ringan.".into());
    }
    decode(ImageReader::new(Cursor::new(
        std::fs::read(path).map_err(|e| e.to_string())?,
    )))
}

fn render_eps(path: &Path) -> AppResult<Vec<u8>> {
    if let Ok(preview) = embedded_eps(path) {
        return Ok(preview);
    }
    check_source_size(path)?;
    let directory = tempfile::tempdir().map_err(|e| e.to_string())?;
    let output = directory.path().join("preview.png");
    run_tool(
        Command::new(ghostscript())
            .arg("-q")
            .arg("-dSAFER")
            .arg("-dBATCH")
            .arg("-dNOPAUSE")
            .arg("-dEPSCrop")
            .arg("-dFirstPage=1")
            .arg("-dLastPage=1")
            .arg("-dMaxBitmap=67108864")
            .arg("-sDEVICE=pngalpha")
            .arg("-r96")
            .arg("-dGraphicsAlphaBits=4")
            .arg("-dTextAlphaBits=4")
            .arg(format!("-sOutputFile={}", output.display()))
            .arg(path),
        "Ghostscript tidak tersedia. Pasang Ghostscript dan mulai ulang aplikasi.",
    )?;
    read_rendered_png(&output)
}

fn ghostscript() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Official Windows installers do not always add Ghostscript to PATH.
        if let Some(program_files) = std::env::var_os("ProgramFiles") {
            if let Ok(entries) = std::fs::read_dir(Path::new(&program_files).join("gs")) {
                let mut executables: Vec<_> = entries
                    .flatten()
                    .map(|entry| entry.path().join("bin/gswin64c.exe"))
                    .filter(|path| path.is_file())
                    .collect();
                executables.sort();
                if let Some(path) = executables.pop() {
                    return path;
                }
            }
        }
        "gswin64c.exe".into()
    }
    #[cfg(not(target_os = "windows"))]
    "gs".into()
}

fn embedded_eps(path: &Path) -> AppResult<Vec<u8>> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let length = file.metadata().map_err(|e| e.to_string())?.len();
    let mut header = [0u8; 30];
    file.read_exact(&mut header)
        .map_err(|_| "EPS tidak memiliki preview TIFF tertanam.")?;
    if header[..4] != [0xc5, 0xd0, 0xd3, 0xc6] {
        return Err(
            "EPS ini tidak memiliki preview DOS/TIFF. Metadata tetap dapat disunting.".into(),
        );
    }
    let offset = u32::from_le_bytes(header[20..24].try_into().unwrap()) as u64;
    let size = u32::from_le_bytes(header[24..28].try_into().unwrap()) as u64;
    if offset < 30
        || size == 0
        || size > 32 * 1024 * 1024
        || offset.checked_add(size).is_none_or(|end| end > length)
    {
        return Err("Preview TIFF EPS tidak tersedia atau offsetnya tidak valid.".into());
    }
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| e.to_string())?;
    let mut bytes = vec![0; size as usize];
    file.read_exact(&mut bytes).map_err(|e| e.to_string())?;
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    decode(reader)
}

fn render_svg(path: &Path) -> AppResult<Vec<u8>> {
    crate::svg::read(path)?;
    let source = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut options = resvg::usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    let tree = resvg::usvg::Tree::from_data(&source, &options).map_err(|e| e.to_string())?;
    let size = tree.size();
    let scale = (MAX_EDGE as f32 / size.width().max(size.height())).min(1.0);
    let width = (size.width() * scale).ceil().max(1.0) as u32;
    let height = (size.height() * scale).ceil().max(1.0) as u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or("Preview SVG terlalu besar untuk dirender.")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(
            width as f32 / size.width(),
            height as f32 / size.height(),
        ),
        &mut pixmap.as_mut(),
    );
    pixmap.encode_png().map_err(|e| e.to_string())
}

fn render_video(path: &Path) -> AppResult<Vec<u8>> {
    check_source_size(path)?;
    let directory = tempfile::tempdir().map_err(|e| e.to_string())?;
    let output = directory.path().join("preview.png");
    run_tool(
        Command::new("ffmpeg")
            .args(["-hide_banner", "-v", "error", "-nostdin"])
            .args(["-protocol_whitelist", "file,crypto,data"])
            .arg("-i")
            .arg(path)
            .args([
                "-map",
                "0:v:0",
                "-an",
                "-vf",
                "scale=w='min(1600,iw)':h='min(1600,ih)':force_original_aspect_ratio=decrease,thumbnail=20",
                "-frames:v",
                "1",
                "-f",
                "image2",
                "-update",
                "1",
                "-vcodec",
                "png",
                "-y",
            ])
            .arg(&output),
        "FFmpeg tidak tersedia. Pasang FFmpeg di PATH dan mulai ulang aplikasi.",
    )?;
    read_rendered_png(&output)
}

fn check_source_size(path: &Path) -> AppResult<()> {
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > MAX_SOURCE_BYTES {
        return Err("File terlalu besar untuk preview ringan.".into());
    }
    Ok(())
}

fn read_rendered_png(path: &Path) -> AppResult<Vec<u8>> {
    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if metadata.len() == 0 || metadata.len() > MAX_PREVIEW_BYTES {
        return Err("Ukuran hasil preview tidak didukung.".into());
    }
    std::fs::read(path).map_err(|e| e.to_string())
}

fn run_tool(command: &mut Command, missing: &'static str) -> AppResult<()> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let mut diagnostics = tempfile::tempfile().map_err(|e| e.to_string())?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(
            diagnostics.try_clone().map_err(|e| e.to_string())?,
        ));
    let mut child = command.spawn().map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            missing.to_owned()
        } else {
            error.to_string()
        }
    })?;
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Pembuatan preview melebihi batas waktu.".into());
        }
        thread::sleep(Duration::from_millis(20));
    };
    if status.success() {
        return Ok(());
    }
    diagnostics
        .seek(SeekFrom::Start(0))
        .map_err(|e| e.to_string())?;
    let mut detail = String::new();
    diagnostics
        .take(4096)
        .read_to_string(&mut detail)
        .map_err(|e| e.to_string())?;
    let detail = detail.trim();
    Err(format!(
        "Preview tidak dapat dibuat. {}",
        if detail.is_empty() {
            "Renderer exited with a nonzero status."
        } else {
            detail
        }
    ))
}

fn decode<R: BufRead + Read + Seek>(reader: ImageReader<R>) -> AppResult<Vec<u8>> {
    let mut reader = reader.with_guessed_format().map_err(|e| e.to_string())?;
    let mut limits = Limits::default();
    limits.max_alloc = Some(512 * 1024 * 1024);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().map_err(|e| e.to_string())?;
    let orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut image = image::DynamicImage::from_decoder(decoder).map_err(|e| e.to_string())?;
    image.apply_orientation(orientation);
    let image = image.thumbnail(MAX_EDGE, MAX_EDGE);
    let mut bytes = Cursor::new(Vec::new());
    image
        .write_to(&mut bytes, ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(bytes.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut output = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            width,
            height,
            Rgba([15, 90, 180, 255]),
        ))
        .write_to(&mut output, ImageFormat::Png)
        .unwrap();
        output.into_inner()
    }

    use image::DynamicImage;

    #[test]
    fn cached_preview_is_reused_invalidated_and_cleared() {
        let source = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        let path = source.path().join("image.png");
        std::fs::write(&path, png(3, 2)).unwrap();

        let first = read_cached(&path, cache.path()).unwrap();
        assert_eq!(first, read_cached(&path, cache.path()).unwrap());
        assert_eq!(cache_size(cache.path()).unwrap(), first.len() as u64);

        std::fs::write(&path, png(8, 2)).unwrap();
        let second = read_cached(&path, cache.path()).unwrap();
        assert_ne!(first, second);
        assert_eq!(
            cache_size(cache.path()).unwrap(),
            (first.len() + second.len()) as u64
        );

        assert_eq!(
            clear_cache(cache.path()).unwrap(),
            (first.len() + second.len()) as u64
        );
        assert_eq!(cache_size(cache.path()).unwrap(), 0);
    }

    #[test]
    fn svg_eps_and_video_produce_png_previews() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let svg = root.join("artwork.svg");
        assert!(image::load_from_memory(&read(&svg).unwrap()).is_ok());
        let eps = root.join("artwork.eps");
        assert!(image::load_from_memory(&read(&eps).unwrap()).is_ok());
        for name in ["clip.mp4", "clip.mov"] {
            let video = root.join(name);
            assert!(image::load_from_memory(&read(&video).unwrap()).is_ok());
        }
    }
}
