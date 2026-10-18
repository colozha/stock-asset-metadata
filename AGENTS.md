# Panduan Agent

## Stack

- Aplikasi desktop lokal: Tauri 2 dengan backend Rust dan webview.
- Frontend: Vite, TypeScript vanilla, dan CSS; tanpa framework UI.
- Metadata format biner memakai Adobe XMP Toolkit melalui binding Rust. SVG memakai adapter XML yang menjaga byte artwork tetap utuh.
- Tidak ada server aplikasi, database, atau upload. Fedora 44 adalah target Linux yang diverifikasi.

## Struktur project

- `src/`: UI editor, tipe frontend, dan stylesheet.
- `src-tauri/src/`: command Tauri dan logika backend.
- `src-tauri/tests/`: pengujian roundtrip dan fixture format.
- `docs/`: detail editor dan bukti/batas validasi.
- `README.md`: penggunaan, format, prasyarat, dan perintah build.
- `PRODUCT.md`, `DESIGN.md`, `IMPLEMENTATION_PLAN.md`: keputusan produk, aturan visual, dan status implementasi.
- `package.json`, `vite.config.ts`, `tsconfig.json`: konfigurasi frontend; `src-tauri/Cargo.toml` dan `src-tauri/tauri.conf.json`: backend dan aplikasi Tauri.

## Fitur dan lokasi

- Editor metadata, daftar aset, draft per file, impor file/folder/drop, shortcut, batch field, dan urutan keyword: `src/main.ts`.
- Kontrak metadata, aset, mode simpan, dan hasil: `src/types.ts` serta struktur backend di `src-tauri/src/metadata.rs` dan `src-tauri/src/files.rs`.
- Layout dan gaya antarmuka: `src/styles.css`.
- Pendaftaran command IPC Tauri (`inspect_assets`, `read_preview`, `save_metadata`) dan event progres simpan: `src-tauri/src/lib.rs`.
- Pembacaan/penulisan XMP dan dukungan format: `src-tauri/src/metadata.rs`; penanganan SVG: `src-tauri/src/svg.rs`.
- Pemeriksaan aset, deteksi perubahan sumber, penyimpanan salinan/pembaruan sumber, file sementara, dan backup: `src-tauri/src/files.rs`.
- Thumbnail gambar dan ekstraksi preview EPS: `src-tauri/src/preview.rs`. Video menampilkan informasi file, tanpa pemutar.
- Roundtrip dan integritas format: `src-tauri/tests/roundtrip.rs` dan `src-tauri/tests/fixtures/`.

## Panduan perubahan

- Format yang ditangani adalah JPG/JPEG, PNG, SVG UTF-8, EPS, MP4, dan MOV. CSV agency dan pemutaran video belum termasuk.
- Pertahankan urutan keyword dan field yang tidak disertakan dalam patch. Ikuti alur simpan yang memvalidasi hasil sebelum finalisasi; pembaruan sumber memakai backup.
- Periksa `README.md` dan `docs/VALIDATION.md` untuk batas dukungan dan bukti validasi. Build di browser tidak menggantikan verifikasi IPC Tauri native.

## Bahasa dan tema

- Semua fitur baru wajib menyediakan copy **ID (Indonesia)** dan **EN (English)** pada `src/messages.ts`, termasuk label, tombol, placeholder, tooltip, teks aksesibilitas, status, dialog, progres, dan error. Gunakan `t()` untuk teks dinamis dan atribut `data-i18n` untuk teks statis; jangan menanam copy UI langsung di logika.
- Kedua terjemahan wajib memiliki parameter interpolasi yang sama. Bahasa UI tidak menerjemahkan isi metadata pengguna, nama/path file, atau nama model/provider; output AI tetap English.
- Error buatan backend harus terdaftar pada `src-tauri/src/messages.rs` dan kamus ID/EN. IPC mengirim `{ key, params, detail }`; frontend menampilkan melalui `errorText()`. Diagnostik OS/library/provider tetap utuh.
- Setiap warna antarmuka menggunakan token CSS untuk light dan dark, termasuk hover, fokus, disabled, selection, dan error. Light mempertahankan desain default.
- Bahasa/tema disimpan melalui satu aksi Simpan pengaturan. Perubahan label/tema harus mempertahankan draft metadata dan urutan keyword. Uji kedua bahasa/tema pada desktop dan ukuran minimum native.
- Resize panel menggunakan `src/layout.ts`; pertahankan akses keyboard, indikator hover/fokus/drag, batas lebar, dan persistensi lokal.

## Perintah pemeriksaan

- `npm run check` — pemeriksaan tipe TypeScript.
- `npm run build` — pemeriksaan tipe dan build Vite.
- `cargo test --manifest-path src-tauri/Cargo.toml --no-default-features` — tes backend tanpa WebKitGTK.
- `npm run bundle` — build aplikasi desktop/RPM; memerlukan dependensi native Linux sesuai README.
