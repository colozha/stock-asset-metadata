# Stock Asset Metadata

Aplikasi desktop lokal untuk membaca dan menanamkan **title**, **description**, dan **keywords** pada JPG/JPEG, PNG, SVG, EPS, MP4, dan MOV. Antarmuka memakai panel persegi dan warna lembut sesuai referensi. Versi awal ditujukan untuk Linux/Fedora.

## Menjalankan

Prasyarat: Node.js 22.12+, npm, Rust stable, compiler C++, serta dependensi Tauri Linux. Lingkungan pengembangan yang digunakan: Fedora 44, Rust 1.99, Node.js 22.23.

```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel librsvg2-devel libxdo-devel gcc-c++ make
npm ci
npm run desktop
```

Seluruh command npm dijalankan dari direktori project ini. `npm run dev` hanya membuka pratinjau frontend di `http://127.0.0.1:1420`; akses file dan penyimpanan memerlukan aplikasi desktop Tauri.

## Penggunaan

1. Tambahkan file, pilih folder, atau seret file ke jendela. Impor folder membaca satu tingkat; subfolder dan symbolic link dilewati.
2. Pilih aset lalu isi metadata. Keywords dipisahkan koma; frasa dengan spasi tetap satu keyword. Tombol panah mengubah urutan. Duplikat dihapus dengan mempertahankan kemunculan pertama.
3. Untuk perubahan batch, centang beberapa aset. Form menyunting aset aktif. Centang field yang ingin disalin dan pilih **Terapkan field ke pilihan**; field lain pada aset lain dipertahankan.
4. Pilih **Simpan salinan** dan folder keluaran, atau **Ganti sumber + backup**. Tombol simpan memproses aset terpilih yang memiliki perubahan.

`Ctrl+O` menambah file. `Ctrl+S` menyimpan perubahan terpilih. Draft bertahan selama sesi; aplikasi meminta konfirmasi ketika ditutup dengan perubahan yang belum disimpan.

Simpan salinan tidak memperbarui file sumber. Mengganti mode ke pembaruan sumber memperlihatkan kembali perubahan yang belum diterapkan pada sumber tersebut.

## Perlindungan file

Setiap penulisan dilakukan pada file sementara di filesystem tujuan. Metadata dibaca kembali dan dibandingkan sebelum hasil difinalisasi. Mode pembaruan membuat backup `file.ext.bak`, lalu `.bak.1`, dan seterusnya. File keluaran yang bertabrakan mendapat suffix `_1`, `_2`, dan seterusnya. Sumber yang berubah sejak diimpor ditolak; gunakan tombol muat ulang setelah memeriksa draft.

Data gambar dan stream video tidak melewati encoder ketika metadata disimpan. Thumbnail dibuat terpisah. Tidak ada upload, server aplikasi, atau database. Keberhasilan satu file tidak menyembunyikan kegagalan file lain.

## Dukungan format

| Format | Metadata | Preview |
| --- | --- | --- |
| JPG/JPEG | Adobe XMP, termasuk rekonsiliasi metadata legacy oleh SDK | Thumbnail dengan orientasi EXIF |
| PNG | XMP standar | Thumbnail, termasuk transparansi |
| SVG UTF-8 | XMP, title/desc dokumen, serta pembacaan Dublin Core sederhana dari vector-pro | Gambar SVG |
| EPS | Handler PostScript Adobe; EPS ASCII dan DOS biner | TIFF tertanam jika decoder mendukungnya |
| MP4/MOV | XMP di container melalui handler MPEG-4/QuickTime Adobe | Informasi nama, format, dan ukuran |

Decoder TIFF belum mendukung beberapa preview palet dengan alpha, termasuk variasi pada sampel Illustrator. Dalam kasus tersebut aplikasi menampilkan keterangan tanpa preview; metadata tetap dapat disunting. JPEG/PNG untuk preview dibatasi 128 MiB, SVG 64 MiB, preview TIFF EPS 32 MiB, dan alokasi decoder 512 MiB. Preview dibatasi 1600 piksel pada sisi terpanjang.

Validasi lokal belum membuktikan pembacaan portal microstock atau roundtrip buka/simpan di Illustrator. Ekspor CSV per agency dan pemutaran video belum termasuk versi ini.

## Build dan pemeriksaan

```bash
npm run check
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --no-default-features
npm run bundle
```

Pengujian backend dapat dijalankan tanpa WebKitGTK melalui `--no-default-features`. Paket RPM berada di `src-tauri/target/release/bundle/rpm/`; executable berada di `src-tauri/target/release/stock-asset-metadata`.

Untuk menyertakan dua sampel Illustrator lokal tanpa mengubah sumbernya:

```bash
VECTOR_PRO_EPS_DIR=/home/husain/Documents/Works/Personal/vector-pro cargo test --manifest-path src-tauri/Cargo.toml --no-default-features
```

Fixture video adalah aset sintetis kecil; pengujian tidak memerlukan FFmpeg sebagai dependensi runtime. Folder sampel eksternal bersifat opsional dan tidak disertakan dalam repository.

## Struktur

- `src/`: editor TypeScript dan CSS, dengan kontrak data di `types.ts`.
- `src-tauri/src/metadata.rs`: model dan integrasi Adobe XMP Toolkit 1.12.1.
- `src-tauri/src/svg.rs`: adapter XML yang mempertahankan byte artwork.
- `src-tauri/src/files.rs`: impor, deteksi perubahan sumber, file sementara, dan backup.
- `src-tauri/src/preview.rs`: thumbnail dan ekstraksi TIFF EPS.
- `src-tauri/tests/`: pengujian roundtrip, integritas, dan kegagalan penyimpanan.

Keputusan produk tersedia di `PRODUCT.md`, aturan visual di `DESIGN.md`, dan status implementasi di `IMPLEMENTATION_PLAN.md`. Rincian bukti pengujian serta batasnya tersedia di [docs/VALIDATION.md](docs/VALIDATION.md).
