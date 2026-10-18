use crate::metadata::{self, AppResult};
use image::{ImageDecoder, ImageFormat, ImageReader, Limits};
use std::{
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
    path::Path,
};

pub fn read(path: &Path) -> AppResult<Vec<u8>> {
    let kind = metadata::format(path)?;
    if kind == "svg" {
        crate::svg::read(path)?;
        return std::fs::read(path).map_err(|e| e.to_string());
    }
    if kind == "mp4" || kind == "mov" {
        return Err("Preview video belum tersedia. Metadata tetap dapat disunting.".into());
    }
    let mut reader = if kind == "eps" {
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
        ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|e| e.to_string())?
    } else {
        // Read only the selected asset; do not preload a batch into memory.
        let file = File::open(path).map_err(|e| e.to_string())?;
        if file.metadata().map_err(|e| e.to_string())?.len() > 128 * 1024 * 1024 {
            return Err("File terlalu besar untuk preview ringan.".into());
        }
        ImageReader::new(Cursor::new(std::fs::read(path).map_err(|e| e.to_string())?))
            .with_guessed_format()
            .map_err(|e| e.to_string())?
    };
    let mut limits = Limits::default();
    limits.max_alloc = Some(512 * 1024 * 1024);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().map_err(|e| e.to_string())?;
    let orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut image = image::DynamicImage::from_decoder(decoder).map_err(|e| e.to_string())?;
    image.apply_orientation(orientation);
    let image = image.thumbnail(1600, 1600);
    let mut bytes = Cursor::new(Vec::new());
    image
        .write_to(&mut bytes, ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(bytes.into_inner())
}
