# Implementasi Stock Asset Metadata

Root project adalah direktori ini; tidak ada direktori project pembungkus baru.

## Keputusan yang disetujui

- Linux dahulu, dengan Fedora 44 sebagai target verifikasi awal.
- Tauri 2 + Rust + Vite + vanilla TypeScript + CSS.
- Title, description, dan ordered keywords untuk JPG/JPEG, PNG, SVG, EPS, MP4, MOV.
- Salinan sebagai default, dengan pilihan pembaruan sumber dan backup berversi.
- Preview ringan; video menampilkan informasi file.
- Pembuatan UI langsung dari kode mengikuti referensi persegi dengan warna lembut.

## Struktur dan kontrak

Frontend terdiri dari `main.ts`, `types.ts`, dan `styles.css`. Backend dipisahkan menjadi metadata, adapter SVG, file, dan preview; `lib.rs` mendaftarkan command Tauri. `Cargo.lock` dan `package-lock.json` disimpan bersama source.

| Command | Kontrak |
| --- | --- |
| `inspect_assets(paths, refresh=false)` | Aset dan kesalahan per file; snapshot diperbarui hanya ketika muat ulang diminta. |
| `read_preview(path)` | Byte gambar untuk file yang sudah diimpor. |
| `save_metadata(request)` | Patch per file, mode simpan, folder keluaran opsional, hasil per file. |
| Event `save_progress` | Jumlah file selesai dan total file. |

Patch yang tidak menyertakan field mempertahankan field tersebut. String kosong atau array kosong menghapus nilai secara disengaja. UI mempertahankan baseline sumber secara terpisah dari salinan yang sudah disimpan sehingga penyimpanan salinan berulang tidak kehilangan perubahan sebelumnya.

## Tahapan

1. **Selesai — kerangka dan backend:** struktur root, dokumentasi, command, handler Adobe untuk format biner, dan adapter SVG dengan escaping XML.
2. **Selesai — editor:** impor file/folder/drop, draft per aset, batch field eksplisit, pengurutan keywords, preview ringan, status, dan kedua mode simpan.
3. **Selesai — validasi dan distribusi:** roundtrip seluruh format, integritas media, Unicode, backup, izin file, benturan nama, sumber berubah, kegagalan parsial, dan paket RPM. Tinjauan visual mencakup koreksi selector native dan kontras teks.

Bukti pemeriksaan serta batas pengujian dicatat di [docs/VALIDATION.md](docs/VALIDATION.md). Artefak rilis tersedia di `src-tauri/target/release/`.

## Penyimpanan

Salin ke file sementara → ubah metadata → tutup penulis dengan pelaporan error → baca kembali → cocokkan metadata → pastikan sumber tidak berubah → buat backup bila perlu → finalisasi → sinkronisasi file dan folder. Penulisan sumber tidak dijalankan bila salah satu pemeriksaan sebelumnya gagal.

## Batas verifikasi

Sampel Illustrator diuji hanya pada salinan sementara; preview TIFF mentah harus identik. Decoder preview tidak mendukung seluruh variasi TIFF, dan kegagalannya tidak memblokir metadata. Kompatibilitas setelah dibuka dan disimpan ulang di Illustrator serta pembacaan metadata oleh portal agency memerlukan pemeriksaan eksternal.

Ekspor CSV dan pemutar video berada di luar versi pertama. Tidak ada layanan cloud, database, atau framework UI tambahan.
