# Stock Asset Metadata

Aplikasi desktop lokal untuk membaca dan menyunting **title**, **description**, dan
**keywords** aset microstock. Dibangun dengan Tauri, Rust, dan TypeScript untuk
**Fedora 44 x86_64**.

![Editor Stock Asset Metadata dalam bahasa Indonesia dengan tema light](docs/images/app-screenshot.png)

## Fitur

- Edit metadata per file atau batch, dengan urutan keyword yang dapat diatur.
- Impor file/folder dan drag-and-drop, pencarian, sorting, serta preview aset.
- Generator judul, deskripsi, dan keyword dengan AI opsional; hasilnya dalam English.
- Simpan salinan atau perbarui sumber dengan backup dan validasi metadata.
- Ekspor **CSV Shutterstock untuk video MP4/MOV**, berisi nama file, deskripsi,
  keyword, dan kategori untuk diimpor setelah video diunggah ke Shutterstock.
- Antarmuka Indonesia/English, tema light/dark, dan panel yang dapat diubah ukurannya.

Pemrosesan metadata berjalan lokal. Fitur AI mengirim permintaan ke provider
pilihan pengguna saat digunakan.

## Dukungan format

| Format | Metadata | Preview |
| --- | --- | --- |
| JPG/JPEG | XMP | Gambar dengan orientasi EXIF |
| PNG | XMP | Gambar dengan transparansi |
| SVG UTF-8 | XMP, title, dan description | Render SVG |
| EPS ASCII/biner | XMP | TIFF tertanam atau Ghostscript |
| MP4/MOV | XMP dan ekspor CSV Shutterstock | Frame statis melalui FFmpeg |

Preview video bukan pemutar video. Kompatibilitas pembacaan metadata mengikuti
portal tujuan; XMP tertanam tidak menjamin metadata terbaca di semua agency.

## Build

Prasyarat: **Node.js 22.12+**, npm, dan **Rust stable**. Dari direktori project
pada Fedora 44, jalankan:

```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel librsvg2-devel libxdo-devel gcc-c++ make ghostscript ffmpeg-free
npm ci
npm run bundle
```

Hasil RPM berada di `src-tauri/target/release/bundle/rpm/`.

## Instalasi RPM

Unduh file `.rpm` dan pasangan `.rpm.sha256` dari
[GitHub Releases](https://github.com/colozha/stock-asset-metadata/releases).
Di folder unduhan, verifikasi checksum lalu instal paket (sesuaikan nama file):

```bash
sha256sum --check 'Stock Asset Metadata-0.1.1-1.x86_64.rpm.sha256'
# Lanjutkan setelah hasil verifikasi OK.
sudo dnf install './Stock Asset Metadata-0.1.1-1.x86_64.rpm'
```

DNF memasang dependensi runtime, termasuk Ghostscript dan FFmpeg. RPM belum
ditandatangani. Setelah instalasi, buka **Stock Asset Metadata** dari menu aplikasi.
