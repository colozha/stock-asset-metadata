use crate::{files::timestamp, metadata, metadata::AppResult};
use image::{metadata::Orientation, ImageDecoder, ImageReader};
use serde::Serialize;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

#[derive(Debug, Serialize, PartialEq)]
pub struct Resolution {
    pub width: f64,
    pub height: f64,
    pub unit: &'static str,
}

impl Resolution {
    fn new(width: f64, height: f64, unit: &'static str) -> Option<Self> {
        (width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0).then_some(Self {
            width,
            height,
            unit,
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetDetails {
    pub resolution: Option<Resolution>,
    pub created_at: Option<u64>,
    pub modified_at: Option<u64>,
}

pub fn read(path: &Path) -> AppResult<AssetDetails> {
    let stat = fs::metadata(path).map_err(|e| e.to_string())?;
    if !stat.is_file() {
        return Err("Sumber bukan file biasa.".into());
    }
    let resolution = match metadata::format(path)?.as_str() {
        "jpg" | "jpeg" | "png" => raster_resolution(path),
        "svg" => svg_resolution(path),
        "eps" => eps_resolution(path),
        "mp4" | "mov" => video_resolution(path),
        _ => None,
    };
    Ok(AssetDetails {
        resolution,
        created_at: timestamp(stat.created()),
        modified_at: timestamp(stat.modified()),
    })
}

fn raster_resolution(path: &Path) -> Option<Resolution> {
    let mut decoder = ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .into_decoder()
        .ok()?;
    let (mut width, mut height) = decoder.dimensions();
    if matches!(
        decoder.orientation().ok(),
        Some(
            Orientation::Rotate90
                | Orientation::Rotate270
                | Orientation::Rotate90FlipH
                | Orientation::Rotate270FlipH
        )
    ) {
        std::mem::swap(&mut width, &mut height);
    }
    Resolution::new(width.into(), height.into(), "px")
}

fn svg_resolution(path: &Path) -> Option<Resolution> {
    // Match the preview's UTF-8 and source-size limits without rendering artwork.
    if fs::metadata(path).ok()?.len() > 64 * 1024 * 1024 {
        return None;
    }
    crate::svg::read(path).ok()?;
    let source = fs::read(path).ok()?;
    let tree = resvg::usvg::Tree::from_data(&source, &resvg::usvg::Options::default()).ok()?;
    Resolution::new(
        tree.size().width().into(),
        tree.size().height().into(),
        "px",
    )
}

fn eps_resolution(path: &Path) -> Option<Resolution> {
    let mut file = File::open(path).ok()?;
    let mut length = file.metadata().ok()?.len();
    let mut header = [0; 12];
    file.read_exact(&mut header).ok()?;
    if header[..4] == [0xc5, 0xd0, 0xd3, 0xc6] {
        let offset = u32::from_le_bytes(header[4..8].try_into().ok()?) as u64;
        let size = u32::from_le_bytes(header[8..12].try_into().ok()?) as u64;
        if offset < 30 || size == 0 || offset.checked_add(size)? > length {
            return None;
        }
        file.seek(SeekFrom::Start(offset)).ok()?;
        length = size;
    } else {
        file.seek(SeekFrom::Start(0)).ok()?;
    }
    // Bounding boxes may be in the trailer; bound scanning of untrusted EPS data.
    if length > 64 * 1024 * 1024 {
        return None;
    }
    let mut bounds = None;
    let mut hires = None;
    let mut nested = 0u32;
    for line in BufReader::new(file.take(length)).split(b'\n') {
        let line = line.ok()?;
        if line.starts_with(b"%%BeginDocument") {
            nested += 1;
        } else if line.starts_with(b"%%EndDocument") {
            nested = nested.saturating_sub(1);
        } else if nested == 0 {
            let (value, target) = if let Some(value) = line.strip_prefix(b"%%HiResBoundingBox:") {
                (value, &mut hires)
            } else if let Some(value) = line.strip_prefix(b"%%BoundingBox:") {
                (value, &mut bounds)
            } else {
                continue;
            };
            let numbers = std::str::from_utf8(value)
                .ok()?
                .split_whitespace()
                .map(str::parse::<f64>)
                .collect::<Result<Vec<_>, _>>();
            if let Ok(values) = numbers {
                if let [left, bottom, right, top] = values.as_slice() {
                    if let Some(size) = Resolution::new(right - left, top - bottom, "pt") {
                        *target = Some(size);
                    }
                }
            }
        }
    }
    hires.or(bounds)
}

fn video_resolution(path: &Path) -> Option<Resolution> {
    let mut command = Command::new("ffprobe");
    command
        .args([
            "-v",
            "error",
            "-protocol_whitelist",
            "file,crypto,data",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height:stream_tags=rotate:stream_side_data=rotation",
            "-of",
            "json",
        ])
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    probe_resolution(&mut command, Duration::from_secs(5))
}

fn probe_resolution(command: &mut Command, timeout: Duration) -> Option<Resolution> {
    let mut child = command.spawn().ok()?;
    let stdout = child.stdout.take()?;
    let reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.take(64 * 1024 + 1).read_to_end(&mut bytes).ok()?;
        (bytes.len() <= 64 * 1024).then_some(bytes)
    });
    let deadline = Instant::now() + timeout;
    let success = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.success(),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break false;
            }
        }
    };
    let bytes = reader.join().ok()??;
    if !success {
        return None;
    }
    parse_video_resolution(&bytes)
}

fn parse_video_resolution(bytes: &[u8]) -> Option<Resolution> {
    let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    let stream = value.get("streams")?.as_array()?.first()?;
    let mut width = stream.get("width")?.as_f64()?;
    let mut height = stream.get("height")?.as_f64()?;
    let rotation = stream
        .get("side_data_list")
        .and_then(|v| v.as_array())
        .and_then(|items| {
            items
                .iter()
                .find_map(|v| v.get("rotation").and_then(|v| v.as_f64()))
        })
        .or_else(|| {
            stream
                .pointer("/tags/rotate")
                .and_then(|v| v.as_str())?
                .parse::<f64>()
                .ok()
        })
        .unwrap_or(0.0);
    if rotation.is_finite() && (rotation.rem_euclid(180.0) - 90.0).abs() < 0.01 {
        std::mem::swap(&mut width, &mut height);
    }
    Resolution::new(width, height, "px")
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ExtendedColorType, ImageEncoder, RgbImage};
    use std::io::Write;

    fn fixture(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    #[test]
    fn source_dimensions_timestamps_and_unreadable_files() {
        let dir = tempfile::tempdir().unwrap();
        for extension in ["jpg", "jpeg", "png"] {
            let path = dir.path().join(format!("image.{extension}"));
            RgbImage::new(2048, 32).save(&path).unwrap();
            let before = fs::read(&path).unwrap();
            let details = read(&path).unwrap();
            assert_eq!(details.resolution, Resolution::new(2048.0, 32.0, "px"));
            let stat = fs::metadata(&path).unwrap();
            assert_eq!(details.created_at, timestamp(stat.created()));
            assert_eq!(details.modified_at, timestamp(stat.modified()));
            let json = serde_json::to_value(details).unwrap();
            assert!(json.get("createdAt").is_some());
            assert!(json.get("modifiedAt").is_some());
            assert_eq!(fs::read(&path).unwrap(), before);
        }
        let broken = dir.path().join("broken.png");
        fs::write(&broken, b"broken").unwrap();
        let details = read(&broken).unwrap();
        assert!(details.resolution.is_none());
        assert!(details.modified_at.is_some());
        assert!(read(&dir.path().join("missing.png")).is_err());
        assert!(read(dir.path()).is_err());
        assert!(timestamp(Ok(std::time::UNIX_EPOCH - Duration::from_secs(1))).is_none());
    }

    #[test]
    fn jpeg_exif_orientation_matches_displayed_dimensions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rotated.jpg");
        for orientation in [1u8, 3, 5, 6, 7, 8] {
            let mut encoder = image::codecs::jpeg::JpegEncoder::new(File::create(&path).unwrap());
            let mut exif =
                b"II\x2a\x00\x08\x00\x00\x00\x01\x00\x12\x01\x03\x00\x01\x00\x00\x00".to_vec();
            exif.extend([orientation, 0, 0, 0, 0, 0, 0, 0]);
            encoder.set_exif_metadata(exif).unwrap();
            encoder
                .write_image(&[0; 48 * 24 * 3], 48, 24, ExtendedColorType::Rgb8)
                .unwrap();
            let (width, height) = if orientation >= 5 {
                (24.0, 48.0)
            } else {
                (48.0, 24.0)
            };
            assert_eq!(
                read(&path).unwrap().resolution,
                Resolution::new(width, height, "px")
            );
        }
    }

    #[test]
    fn vectors_use_intrinsic_canvas_and_top_level_bounding_boxes() {
        assert_eq!(
            read(&fixture("artwork.svg")).unwrap().resolution,
            Resolution::new(100.0, 100.0, "px")
        );
        assert_eq!(
            read(&fixture("artwork.eps")).unwrap().resolution,
            Resolution::new(100.0, 100.0, "pt")
        );
        let dir = tempfile::tempdir().unwrap();
        let svg = dir.path().join("physical.svg");
        fs::write(
            &svg,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="2in" height="1in"/>"#,
        )
        .unwrap();
        assert_eq!(
            read(&svg).unwrap().resolution,
            Resolution::new(192.0, 96.0, "px")
        );
        let ps = b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: (atend)\n%%BeginDocument: nested.eps\n%%HiResBoundingBox: 0 0 999 999\n%%EndDocument\n%%Trailer\n%%BoundingBox: -10 -20 101 201\n%%HiResBoundingBox: -10.5 -20.25 100.25 200.5\n%%EOF\n";
        let path = dir.path().join("canvas.eps");
        for binary in [false, true] {
            let mut file = File::create(&path).unwrap();
            if binary {
                let mut header = [0u8; 30];
                header[..4].copy_from_slice(&[0xc5, 0xd0, 0xd3, 0xc6]);
                header[4..8].copy_from_slice(&30u32.to_le_bytes());
                header[8..12].copy_from_slice(&(ps.len() as u32).to_le_bytes());
                file.write_all(&header).unwrap();
            }
            file.write_all(ps).unwrap();
            if binary {
                file.write_all(b"%%BoundingBox: 0 0 888 888\n").unwrap();
            }
            assert_eq!(
                read(&path).unwrap().resolution,
                Resolution::new(110.75, 220.75, "pt")
            );
        }
        fs::write(
            &path,
            b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 NaN 1\n",
        )
        .unwrap();
        assert!(read(&path).unwrap().resolution.is_none());
    }

    #[test]
    fn videos_use_source_dimensions_and_rotation() {
        for name in ["clip.mp4", "clip.mov"] {
            let resolution = read(&fixture(name)).unwrap().resolution.unwrap();
            assert_eq!(resolution.unit, "px");
            assert!(resolution.width > 0.0 && resolution.height > 0.0);
        }
        for rotation in ["-90", "90", "270"] {
            let json = format!(
                r#"{{"streams":[{{"width":3840,"height":2160,"side_data_list":[{{"rotation":{rotation}}}]}}]}}"#
            );
            assert_eq!(
                parse_video_resolution(json.as_bytes()),
                Resolution::new(2160.0, 3840.0, "px")
            );
        }
        assert_eq!(
            parse_video_resolution(
                br#"{"streams":[{"width":1920,"height":1080,"tags":{"rotate":"180"}}]}"#
            ),
            Resolution::new(1920.0, 1080.0, "px")
        );
        for bytes in [
            b"broken".as_slice(),
            br#"{"streams":[]}"#,
            br#"{"streams":[{"width":0,"height":1080}]}"#,
        ] {
            assert!(parse_video_resolution(bytes).is_none());
        }
    }

    #[cfg(unix)]
    #[test]
    fn probe_missing_timeout_and_output_limit_are_nonfatal() {
        let prepare = |command: &mut Command| {
            command.stdout(Stdio::piped()).stderr(Stdio::null());
        };
        let mut missing = Command::new("/nonexistent/stock-asset-details-ffprobe");
        prepare(&mut missing);
        assert!(probe_resolution(&mut missing, Duration::from_millis(50)).is_none());
        let mut timeout = Command::new("sh");
        timeout.args(["-c", "exec sleep 2"]);
        prepare(&mut timeout);
        let start = Instant::now();
        assert!(probe_resolution(&mut timeout, Duration::from_millis(50)).is_none());
        assert!(start.elapsed() < Duration::from_secs(1));
        let mut oversized = Command::new("sh");
        oversized.args(["-c", "exec head -c 65537 /dev/zero"]);
        prepare(&mut oversized);
        assert!(probe_resolution(&mut oversized, Duration::from_secs(1)).is_none());
    }
}
