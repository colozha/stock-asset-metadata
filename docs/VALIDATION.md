# Hasil validasi

Tanggal: 18 Oktober 2026. Lingkungan: Fedora 44 x86_64, Rust 1.99, Node.js 22.23, WebKitGTK 2.54.0. Target awal adalah desktop Linux.

## Backend

Command berikut lulus: **8 pengujian**. Pengujian backend menggunakan fixture sintetis yang disimpan di repository atau dibuat pada direktori sementara.

```bash
cargo test --manifest-path src-tauri/Cargo.toml --no-default-features
```

- Enam format: penulisan, pembacaan ulang, Unicode, karakter XML, penulisan berulang, dan penghapusan nilai.
- Integritas: entropy JPEG, piksel PNG, payload `mdat` MP4/MOV, byte artwork SVG, serta preview TIFF dan offset DOS EPS.
- Penyimpanan: benturan nama salinan, backup berversi dengan isi identik, sumber berubah, tujuan tidak tersedia, dan patch yang mempertahankan field lain.
- Izin: sumber read-only dapat disalin; penggantian sumber read-only ditolak; izin sumber dan backup dipertahankan setelah penggantian inode oleh SDK.
- SVG: namespace, root kosong, metadata legacy, RDF lain tetap tersimpan; deklarasi encoding selain UTF-8 ditolak tanpa perubahan sumber.

## Antarmuka dan aplikasi native

`npm run build` menyelesaikan pemeriksaan TypeScript dan build Vite.

Pemeriksaan renderer Chromium menggunakan mock IPC resmi Tauri untuk menguji impor, draft per aset, urutan keyword, batch field eksplisit, penyimpanan salinan berulang, perpindahan mode simpan, dan kegagalan parsial. Tidak ditemukan error runtime atau overflow horizontal pada 1440×900 dan 1024×700. Mock hanya menggantikan akses desktop dalam pemeriksaan renderer ini.

Pengujian terpisah menjalankan **binary release Tauri asli** melalui `tauri-driver` dan WebKitWebDriver. Impor enam format, edit form, penerapan batch, simpan dengan backup, pembacaan ulang Unicode melalui Rust, dan simpan salinan berhasil. Semua backup cocok dengan sumber; izin sumber dan backup `0600` tetap utuh pada keenam format. Impor duplikat setelah perubahan eksternal tidak mereset deteksi konflik; penulisan ditolak dan perubahan eksternal tetap utuh.

Browser plugin tidak menyediakan browser aktif pada lingkungan ini; pemeriksaan renderer memakai Playwright. Pengujian native menggunakan IPC Rust sungguhan. Script otomasi sementara tidak menambah dependensi runtime project.

## Distribusi

`npm run bundle` menghasilkan executable dan paket RPM x86_64 pada `src-tauri/target/release/`. Pemeriksaan library executable tidak menemukan dependency yang hilang atau dependency ke sysroot sementara.

Header development Fedora belum terpasang pada sistem pengujian. Build lokal menggunakan paket development Fedora resmi yang diekstrak ke `/tmp/stock-asset-metadata-sysroot`, tanpa mengubah paket sistem:

```bash
PKG_CONFIG_PATH=/tmp/stock-asset-metadata-sysroot/usr/lib64/pkgconfig:/tmp/stock-asset-metadata-sysroot/usr/share/pkgconfig CARGO_BUILD_JOBS=3 npm run bundle
```

Path sementara tersebut hanya diperlukan untuk build pada sesi ini. Untuk lingkungan development biasa, ikuti prasyarat Fedora di README.

## Batas bukti

Preview TIFF palet dengan alpha pada beberapa EPS Illustrator belum dapat didekode, tetapi byte preview tetap dipertahankan saat metadata ditulis. Video menampilkan informasi file; aplikasi tidak menyediakan pemutar video. SVG dibatasi UTF-8.

Belum dilakukan pengujian unggah ke portal microstock, buka/simpan ulang di Illustrator, distribusi selain Fedora 44, atau sistem operasi lain. Pemeriksaan lokal XMP tidak menjamin tiap portal membaca field yang sama.
