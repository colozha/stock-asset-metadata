//! Translate application-owned errors at the IPC boundary; external diagnostics stay intact.
use serde::{Serialize, Serializer};
#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub key: Option<String>,
    pub params: Vec<String>,
    pub detail: Option<String>,
}
impl From<&str> for AppError {
    fn from(message: &str) -> Self {
        message.to_owned().into()
    }
}
impl From<String> for AppError {
    fn from(message: String) -> Self {
        for &(key, parts) in MESSAGES {
            if parts.len() == 1 {
                if message == key {
                    return Self {
                        key: Some(key.into()),
                        params: vec![],
                        detail: None,
                    };
                }
                continue;
            }
            let Some(mut rest) = message.strip_prefix(parts[0]) else {
                continue;
            };
            let mut params = Vec::new();
            let mut matched = true;
            for (index, part) in parts.iter().enumerate().skip(1) {
                if index == parts.len() - 1 {
                    if let Some(value) = rest.strip_suffix(part) {
                        params.push(value.to_owned());
                    } else {
                        matched = false;
                    }
                } else if let Some((value, tail)) = rest.split_once(part) {
                    params.push(value.to_owned());
                    rest = tail;
                } else {
                    matched = false;
                    break;
                }
            }
            if matched {
                return Self {
                    key: Some(key.into()),
                    params,
                    detail: None,
                };
            }
        }
        Self {
            key: None,
            params: vec![],
            detail: Some(message),
        }
    }
}
pub fn serialize_error<S: Serializer>(
    error: &Option<String>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    error.clone().map(AppError::from).serialize(serializer)
}
const MESSAGES: &[(&str, &[&str])] = &[
    ("Direktori output tidak dapat dibuka: {0}", &["Direktori output tidak dapat dibuka: ", ""]),
    ("AI belum menghasilkan title dan description yang lengkap.", &["AI belum menghasilkan title dan description yang lengkap."]),
    ("AI mengembalikan keyword kosong, duplikat, atau berisi koma. Draft tidak diubah.", &["AI mengembalikan keyword kosong, duplikat, atau berisi koma. Draft tidak diubah."]),
    ("AI mengembalikan {0} dari {1} keyword yang diminta. Draft tidak diubah.", &["AI mengembalikan ", " dari ", " keyword yang diminta. Draft tidak diubah."]),
    ("API key tidak dapat dihapus dari credential store OS.", &["API key tidak dapat dihapus dari credential store OS."]),
    ("API key tidak dapat disimpan di credential store OS.", &["API key tidak dapat disimpan di credential store OS."]),
    ("Credential store OS tidak dapat dibuka.", &["Credential store OS tidak dapat dibuka."]),
    ("Credential store OS tidak tersedia.", &["Credential store OS tidak tersedia."]),
    ("Daftar model provider tidak dapat dibaca.", &["Daftar model provider tidak dapat dibaca."]),
    ("Daftar model tidak dapat dimuat. Periksa jaringan dan endpoint.", &["Daftar model tidak dapat dimuat. Periksa jaringan dan endpoint."]),
    ("Description hasil AI harus berisi 5–{0} kata. Draft tidak diubah.", &["Description hasil AI harus berisi 5–", " kata. Draft tidak diubah."]),
    ("Direktori Documents tidak tersedia: {0}", &["Direktori Documents tidak tersedia: ", ""]),
    ("Direktori keluaran harus berupa path absolut.", &["Direktori keluaran harus berupa path absolut."]),
    ("Dokumen XML tidak lengkap.", &["Dokumen XML tidak lengkap."]),
    ("EPS ini tidak memiliki preview DOS/TIFF. Metadata tetap dapat disunting.", &["EPS ini tidak memiliki preview DOS/TIFF. Metadata tetap dapat disunting."]),
    ("EPS tidak memiliki preview TIFF tertanam.", &["EPS tidak memiliki preview TIFF tertanam."]),
    ("Endpoint harus memakai URL HTTP atau HTTPS tanpa query, fragment, atau kredensial di URL.", &["Endpoint harus memakai URL HTTP atau HTTPS tanpa query, fragment, atau kredensial di URL."]),
    ("File berubah saat dibaca. Impor ulang file.", &["File berubah saat dibaca. Impor ulang file."]),
    ("File pengaturan AI rusak dan tidak dapat dibaca.", &["File pengaturan AI rusak dan tidak dapat dibaca."]),
    ("File sementara pengaturan tidak dapat dibuat.", &["File sementara pengaturan tidak dapat dibuat."]),
    ("File terlalu besar untuk preview ringan.", &["File terlalu besar untuk preview ringan."]),
    ("FFmpeg tidak tersedia. Pasang paket ffmpeg-free.", &["FFmpeg tidak tersedia. Pasang paket ffmpeg-free."]),
    ("Folder keluaran tidak tersedia.", &["Folder keluaran tidak tersedia."]),
    ("Folder keluaran {0} tidak dapat dibuat: {1}", &["Folder keluaran ", " tidak dapat dibuat: ", ""]),
    ("Folder pengaturan aplikasi tidak dapat dibuat.", &["Folder pengaturan aplikasi tidak dapat dibuat."]),
    ("Folder sumber tidak tersedia.", &["Folder sumber tidak tersedia."]),
    ("Format JSON dari provider AI tidak valid.", &["Format JSON dari provider AI tidak valid."]),
    ("Format tidak didukung. Gunakan JPG, PNG, SVG, EPS, MP4, atau MOV.", &["Format tidak didukung. Gunakan JPG, PNG, SVG, EPS, MP4, atau MOV."]),
    ("Handler tidak dapat menulis metadata pada file ini.", &["Handler tidak dapat menulis metadata pada file ini."]),
    ("Ghostscript tidak tersedia. Pasang paket ghostscript.", &["Ghostscript tidak tersedia. Pasang paket ghostscript."]),
    ("Hasil sudah ditulis, tetapi sinkronisasi folder gagal: {0}", &["Hasil sudah ditulis, tetapi sinkronisasi folder gagal: ", ""]),
    ("Impor aset terlebih dahulu.", &["Impor aset terlebih dahulu."]),
    ("Impor file terlebih dahulu.", &["Impor file terlebih dahulu."]),
    ("Impor ulang file sebelum menyimpan.", &["Impor ulang file sebelum menyimpan."]),
    ("Jumlah keyword harus antara 10 dan 50.", &["Jumlah keyword harus antara 10 dan 50."]),
    ("Klien jaringan AI tidak dapat disiapkan.", &["Klien jaringan AI tidak dapat disiapkan."]),
    ("Konteks tambahan maksimal 4000 karakter.", &["Konteks tambahan maksimal 4000 karakter."]),
    ("Lokasi pengaturan aplikasi tidak dapat dibuka.", &["Lokasi pengaturan aplikasi tidak dapat dibuka."]),
    ("Lokasi pengaturan aplikasi tidak valid.", &["Lokasi pengaturan aplikasi tidak valid."]),
    ("Lokasi pengaturan tidak valid.", &["Lokasi pengaturan tidak valid."]),
    ("Metadata mengandung karakter yang tidak valid untuk XML.", &["Metadata mengandung karakter yang tidak valid untuk XML."]),
    ("Nama file tidak valid.", &["Nama file tidak valid."]),
    ("Paket RDF tidak tersedia.", &["Paket RDF tidak tersedia."]),
    ("Panjang description tidak valid.", &["Panjang description tidak valid."]),
    ("Pembacaan ulang metadata tidak sesuai. Sumber tidak diganti.", &["Pembacaan ulang metadata tidak sesuai. Sumber tidak diganti."]),
    ("Pengaturan AI lokal tidak dapat dibaca.", &["Pengaturan AI lokal tidak dapat dibaca."]),
    ("Pengaturan AI tidak dapat difinalisasi.", &["Pengaturan AI tidak dapat difinalisasi."]),
    ("Pengaturan AI tidak dapat disiapkan untuk disimpan.", &["Pengaturan AI tidak dapat disiapkan untuk disimpan."]),
    ("Pengaturan AI tidak dapat ditulis.", &["Pengaturan AI tidak dapat ditulis."]),
    ("Pengaturan aplikasi tidak dapat dibaca: {0}", &["Pengaturan aplikasi tidak dapat dibaca: ", ""]),
    ("Pengaturan aplikasi tidak valid: {0}", &["Pengaturan aplikasi tidak valid: ", ""]),
    ("Penutup XML tanpa pembuka.", &["Penutup XML tanpa pembuka."]),
    ("Pilih folder keluaran terlebih dahulu.", &["Pilih folder keluaran terlebih dahulu."]),
    ("Pilih model AI terlebih dahulu.", &["Pilih model AI terlebih dahulu."]),
    ("Pilih provider AI di halaman Settings terlebih dahulu.", &["Pilih provider AI di halaman Settings terlebih dahulu."]),
    ("Pembuatan preview melebihi batas waktu.", &["Pembuatan preview melebihi batas waktu."]),
    ("Prefix namespace XML tidak dikenal.", &["Prefix namespace XML tidak dikenal."]),
    ("Preview TIFF EPS tidak tersedia atau offsetnya tidak valid.", &["Preview TIFF EPS tidak tersedia atau offsetnya tidak valid."]),
    ("Preview tidak dapat dibuat. {0}", &["Preview tidak dapat dibuat. ", ""]),
    ("Preview SVG terlalu besar untuk dirender.", &["Preview SVG terlalu besar untuk dirender."]),
    ("Preview video belum tersedia. Metadata tetap dapat disunting.", &["Preview video belum tersedia. Metadata tetap dapat disunting."]),
    ("Provider AI mengembalikan HTTP {0}. Periksa API key, model, dan endpoint.", &["Provider AI mengembalikan HTTP ", ". Periksa API key, model, dan endpoint."]),
    ("Provider AI mengembalikan respons kosong.", &["Provider AI mengembalikan respons kosong."]),
    ("Provider AI tidak dapat dihubungi. Periksa jaringan dan endpoint.", &["Provider AI tidak dapat dihubungi. Periksa jaringan dan endpoint."]),
    ("Provider merespons, tetapi format tes tidak sesuai.", &["Provider merespons, tetapi format tes tidak sesuai."]),
    ("Provider tidak mengembalikan model yang dapat digunakan. Masukkan model secara manual.", &["Provider tidak mengembalikan model yang dapat digunakan. Masukkan model secara manual."]),
    ("Provider tidak menyediakan daftar model (HTTP {0}). Masukkan model secara manual.", &["Provider tidak menyediakan daftar model (HTTP ", "). Masukkan model secara manual."]),
    ("Provider yang dipilih tidak tersedia.", &["Provider yang dipilih tidak tersedia."]),
    ("Respons AI tidak berisi JSON lengkap.", &["Respons AI tidak berisi JSON lengkap."]),
    ("Respons AI tidak berisi JSON.", &["Respons AI tidak berisi JSON."]),
    ("Respons provider AI tidak dapat dibaca.", &["Respons provider AI tidak dapat dibaca."]),
    ("Root dokumen bukan SVG.", &["Root dokumen bukan SVG."]),
    ("SVG harus berupa UTF-8: {0}", &["SVG harus berupa UTF-8: ", ""]),
    ("SVG harus menggunakan encoding UTF-8.", &["SVG harus menggunakan encoding UTF-8."]),
    ("SVG melebihi batas 64 MiB untuk penyuntingan metadata.", &["SVG melebihi batas 64 MiB untuk penyuntingan metadata."]),
    ("Sumber bersifat read-only. Gunakan Simpan salinan.", &["Sumber bersifat read-only. Gunakan Simpan salinan."]),
    ("Sumber berubah saat membuat backup. Sumber tidak diganti.", &["Sumber berubah saat membuat backup. Sumber tidak diganti."]),
    ("Sumber berubah sejak diimpor. Impor ulang sebelum menyimpan.", &["Sumber berubah sejak diimpor. Impor ulang sebelum menyimpan."]),
    ("Sumber berubah selama penyimpanan. Hasil dibatalkan.", &["Sumber berubah selama penyimpanan. Hasil dibatalkan."]),
    ("Sumber bukan file biasa.", &["Sumber bukan file biasa."]),
    ("Tujuan keluaran harus berupa direktori.", &["Tujuan keluaran harus berupa direktori."]),
    ("Ukuran hasil preview tidak didukung.", &["Ukuran hasil preview tidak didukung."]),
    ("URL endpoint tidak valid.", &["URL endpoint tidak valid."]),
    ("XML memiliki lebih dari satu elemen root.", &["XML memiliki lebih dari satu elemen root."]),
    ("XML tidak valid: {0}", &["XML tidak valid: ", ""]),
    ("Endpoint HTTP hanya diizinkan untuk layanan di komputer ini. Gunakan HTTPS untuk server lain.", &["Endpoint HTTP hanya diizinkan untuk layanan di komputer ini. Gunakan HTTPS untuk server lain."]),
    ("Masukkan API key provider di halaman Settings.", &["Masukkan API key provider di halaman Settings."]),
    ("Pemrosesan lain masih berlangsung. Tunggu hingga selesai.", &["Pemrosesan lain masih berlangsung. Tunggu hingga selesai."]),
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn error_parameters_and_external_diagnostics_are_preserved() {
        let error = AppError::from(
            "Folder keluaran /tmp/a tidak dapat dibuat: Permission denied".to_owned(),
        );
        assert_eq!(
            error.key.as_deref(),
            Some("Folder keluaran {0} tidak dapat dibuat: {1}")
        );
        assert_eq!(error.params, ["/tmp/a", "Permission denied"]);
        let error = AppError::from("Third-party diagnostic".to_owned());
        assert!(error.key.is_none());
        assert_eq!(error.detail.as_deref(), Some("Third-party diagnostic"));
    }
    #[test]
    fn every_backend_message_has_both_languages_and_matching_parameters() {
        let source = include_str!("../../src/messages.ts");
        // The dictionary is a TypeScript object with JSON-compatible entries.
        let object = source
            .split_once("= {")
            .unwrap()
            .1
            .split_once("} as const;")
            .unwrap()
            .0;
        let json = format!("{{{}}}", object.trim_end().trim_end_matches(','));
        let catalog: serde_json::Value = serde_json::from_str(&json).unwrap();
        for (key, parts) in MESSAGES {
            for language in ["id", "en"] {
                let text = catalog[key][language]
                    .as_str()
                    .unwrap_or_else(|| panic!("Missing {language}: {key}"));
                assert!(!text.is_empty());
                for i in 0..parts.len() - 1 {
                    assert!(
                        text.contains(&format!("{{{i}}}")),
                        "Missing parameter: {key}"
                    );
                }
            }
        }
        for (key, translations) in catalog.as_object().unwrap() {
            for language in ["id", "en"] {
                let text = translations[language].as_str().unwrap();
                let placeholders = |value: &str| {
                    value
                        .split('{')
                        .skip(1)
                        .filter_map(|s| s.split_once('}').map(|(n, _)| n.to_owned()))
                        .collect::<std::collections::BTreeSet<_>>()
                };
                assert_eq!(
                    placeholders(key),
                    placeholders(text),
                    "Parameters differ for {language}: {key}"
                );
            }
        }
    }
}
