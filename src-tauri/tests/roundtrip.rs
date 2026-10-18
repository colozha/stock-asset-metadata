use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};
use stock_asset_metadata_lib::{
    files::{self, SaveItem, SaveMode},
    metadata::{self, Metadata, MetadataPatch},
    preview,
};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}
fn value() -> Metadata {
    Metadata {
        title: "A&B <forest> — 日本 / Hutan".into(),
        description: "Description with \"quotes\", café & green trees.\nSecond line.".into(),
        keywords: vec!["tropical forest".into(), "nature".into(), "日本".into()],
    }
}
fn patch(value: &Metadata) -> MetadataPatch {
    MetadataPatch {
        title: Some(value.title.clone()),
        description: Some(value.description.clone()),
        keywords: Some(value.keywords.clone()),
    }
}
fn raster(path: &Path) {
    image::RgbImage::from_pixel(24, 24, image::Rgb([123u8, 104, 164]))
        .save(path)
        .unwrap();
}
fn dos_eps(path: &Path) {
    let ps = fs::read(fixture("artwork.eps")).unwrap();
    let mut tiff = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
        16,
        16,
        image::Rgb([123, 104, 164]),
    ))
    .write_to(&mut tiff, image::ImageFormat::Tiff)
    .unwrap();
    let mut bytes = vec![0xc5, 0xd0, 0xd3, 0xc6];
    for number in [
        30u32,
        ps.len() as u32,
        0,
        0,
        30 + ps.len() as u32,
        tiff.get_ref().len() as u32,
    ] {
        bytes.extend(number.to_le_bytes());
    }
    bytes.extend([0xff, 0xff]);
    bytes.extend(ps);
    bytes.extend(tiff.into_inner());
    fs::write(path, bytes).unwrap();
}
fn mp4_media(bytes: &[u8]) -> Vec<u8> {
    let mut cursor = 0;
    while cursor + 8 <= bytes.len() {
        let size = u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().unwrap()) as usize;
        assert!(size >= 8 && cursor + size <= bytes.len());
        if &bytes[cursor + 4..cursor + 8] == b"mdat" {
            return bytes[cursor + 8..cursor + size].to_vec();
        }
        cursor += size;
    }
    panic!("mdat missing");
}

#[test]
fn all_formats_roundtrip_unicode_and_keep_media() {
    let dir = tempfile::tempdir().unwrap();
    for name in [
        "photo.jpg",
        "image.png",
        "artwork.svg",
        "artwork.eps",
        "clip.mp4",
        "clip.mov",
    ] {
        let path = dir.path().join(name);
        if name.ends_with("jpg") || name.ends_with("png") {
            raster(&path);
        } else {
            fs::copy(fixture(name), &path).unwrap();
        }
        let before = fs::read(&path).unwrap();
        metadata::write(&path, &value()).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(metadata::read(&path).unwrap(), value(), "{name}");
        let after = fs::read(&path).unwrap();
        if name.ends_with("jpg") {
            let sos = |data: &[u8]| data.windows(2).position(|w| w == [0xff, 0xda]).unwrap();
            assert_eq!(&before[sos(&before)..], &after[sos(&after)..]);
        } else if name.ends_with("png") {
            assert_eq!(
                image::load_from_memory(&before).unwrap().to_rgba8(),
                image::load_from_memory(&after).unwrap().to_rgba8()
            );
        } else if name.ends_with("mp4") || name.ends_with("mov") {
            assert_eq!(mp4_media(&before), mp4_media(&after));
        } else if name.ends_with("svg") {
            let xml = String::from_utf8(after).unwrap();
            assert!(xml.contains("<g id=\"artwork\" transform=\"translate(10 10)\"><title>Keep shape title</title><path fill=\"url(#paint)\" d=\"M0 0H80V80H0Z\"/></g>"));
            assert!(xml.contains("Fixture attribution"));
        }
        for _ in 0..3 {
            metadata::write(&path, &value()).unwrap();
        }
        assert_eq!(metadata::read(&path).unwrap(), value());
        let cleared = Metadata::default();
        metadata::write(&path, &cleared).unwrap();
        assert_eq!(metadata::read(&path).unwrap(), cleared, "clear {name}");
    }
}

#[test]
fn dos_eps_offsets_preview_and_artwork_survive() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("binary.eps");
    dos_eps(&path);
    let before_preview = preview::read(&path).unwrap();
    metadata::write(&path, &value()).unwrap();
    assert_eq!(metadata::read(&path).unwrap(), value());
    assert_eq!(preview::read(&path).unwrap(), before_preview);
    let bytes = fs::read(&path).unwrap();
    let number =
        |offset: usize| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
    assert!(number(4) + number(8) <= bytes.len());
    assert!(number(20) + number(24) <= bytes.len());
    assert!(bytes
        .windows(b"80 0 rlineto".len())
        .any(|w| w == b"80 0 rlineto"));
}

#[test]
fn svg_legacy_namespace_empty_root_and_unrelated_xmp() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.svg");
    fs::copy(fixture("artwork.svg"), &path).unwrap();
    assert_eq!(
        metadata::read(&path).unwrap().keywords,
        vec!["tropical forest", "nature"]
    );
    for svg in ["<svg xmlns=\"http://www.w3.org/2000/svg\"/>", "<s:svg xmlns:s=\"http://www.w3.org/2000/svg\"><s:metadata/><s:g/></s:svg>", "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\"><metadata><rdf:RDF><rdf:Description rdf:about=\"\"><dc:rights><rdf:Alt><rdf:li xml:lang=\"x-default\">Copyright fixture</rdf:li></rdf:Alt></dc:rights></rdf:Description></rdf:RDF></metadata></svg>"] {
        fs::write(&path, svg).unwrap();
        metadata::write(&path, &value()).unwrap();
        assert_eq!(metadata::read(&path).unwrap(), value());
        let first_length = fs::metadata(&path).unwrap().len();
        metadata::write(&path, &value()).unwrap();
        assert_eq!(fs::metadata(&path).unwrap().len(), first_length);
        if svg.contains("Copyright fixture") { assert!(fs::read_to_string(&path).unwrap().contains("Copyright fixture")); }
    }
}

#[test]
fn copies_and_versioned_backups_never_clobber_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("photo.jpg");
    raster(&path);
    let original = fs::read(&path).unwrap();
    let before = files::stamp(&path).unwrap();
    let item = SaveItem {
        path: path.to_str().unwrap().into(),
        patch: patch(&value()),
    };
    let first = files::save_one(&item, &SaveMode::Copy, Some(dir.path()), &before).unwrap();
    let second = files::save_one(&item, &SaveMode::Copy, Some(dir.path()), &before).unwrap();
    assert_ne!(first.output_path, second.output_path);
    assert_eq!(fs::read(&path).unwrap(), original);
    let replaced = files::save_one(&item, &SaveMode::ReplaceWithBackup, None, &before).unwrap();
    assert_eq!(fs::read(replaced.backup_path.unwrap()).unwrap(), original);
    let before = files::stamp(&path).unwrap();
    let previous = fs::read(&path).unwrap();
    let again = files::save_one(&item, &SaveMode::ReplaceWithBackup, None, &before).unwrap();
    assert!(again.backup_path.as_ref().unwrap().ends_with(".bak.1"));
    assert_eq!(fs::read(again.backup_path.unwrap()).unwrap(), previous);
}

#[test]
fn changed_source_and_failed_write_preserve_original() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("photo.jpg");
    raster(&path);
    let before = files::stamp(&path).unwrap();
    let item = SaveItem {
        path: path.to_str().unwrap().into(),
        patch: patch(&value()),
    };
    let original = fs::read(&path).unwrap();
    assert!(files::save_one(
        &item,
        &SaveMode::Copy,
        Some(&dir.path().join("missing")),
        &before
    )
    .is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::write(&path, b"changed source").unwrap();
    assert!(
        files::save_one(&item, &SaveMode::ReplaceWithBackup, None, &before)
            .unwrap_err()
            .contains("berubah")
    );
    assert_eq!(fs::read(&path).unwrap(), b"changed source");
}

#[cfg(unix)]
#[test]
fn readonly_sources_allow_copies_and_backups_preserve_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.jpg");
    raster(&source);
    fs::set_permissions(&source, fs::Permissions::from_mode(0o400)).unwrap();
    let before = files::stamp(&source).unwrap();
    let item = SaveItem {
        path: source.to_str().unwrap().into(),
        patch: patch(&value()),
    };
    let result = files::save_one(&item, &SaveMode::Copy, Some(dir.path()), &before).unwrap();
    assert_eq!(
        metadata::read(Path::new(result.output_path.as_ref().unwrap())).unwrap(),
        value()
    );
    assert!(files::save_one(&item, &SaveMode::ReplaceWithBackup, None, &before).is_err());
    fs::set_permissions(&source, fs::Permissions::from_mode(0o600)).unwrap();
    let before = files::stamp(&source).unwrap();
    let result = files::save_one(&item, &SaveMode::ReplaceWithBackup, None, &before).unwrap();
    assert_eq!(
        fs::metadata(result.backup_path.unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert_eq!(
        fs::metadata(&source).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn partial_patch_validation_and_import_errors() {
    let original = value();
    let patch = MetadataPatch {
        title: Some(" New ".into()),
        ..Default::default()
    };
    let changed = patch.apply(&original).unwrap();
    assert_eq!(changed.title, "New");
    assert_eq!(changed.description, original.description);
    assert_eq!(changed.keywords, original.keywords);
    assert!(MetadataPatch {
        title: Some("bad\0text".into()),
        ..Default::default()
    }
    .apply(&original)
    .is_err());
    let normalized = MetadataPatch {
        keywords: Some(vec![
            " Nature ".into(),
            "nature".into(),
            "tropical forest".into(),
            " ".into(),
        ]),
        ..Default::default()
    }
    .apply(&original)
    .unwrap();
    assert_eq!(normalized.keywords, vec!["Nature", "tropical forest"]);
    let dir = tempfile::tempdir().unwrap();
    raster(&dir.path().join("good.jpg"));
    fs::write(dir.path().join("bad.jpg"), b"broken").unwrap();
    let (assets, stamps) = files::inspect(vec![dir.path().to_str().unwrap().into()]);
    assert_eq!(assets.len(), 2);
    assert_eq!(stamps.len(), 1);
    assert_eq!(assets.iter().filter(|a| a.error.is_some()).count(), 1);
}

#[test]
fn non_utf8_svg_declarations_are_rejected_without_changes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("encoded.svg");
    fs::write(&path, "<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?><svg xmlns=\"http://www.w3.org/2000/svg\"/>").unwrap();
    let source = fs::read(&path).unwrap();
    assert!(metadata::write(&path, &value()).is_err());
    assert_eq!(fs::read(&path).unwrap(), source);
}
