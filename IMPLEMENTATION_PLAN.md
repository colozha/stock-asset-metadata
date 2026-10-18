# Implementasi Stock Asset Metadata

Root project adalah direktori ini; tidak ada direktori project pembungkus baru.

## Keputusan yang disetujui

- Linux dahulu, dengan Fedora 44 sebagai target verifikasi awal.
- Tauri 2 + Rust + Vite + vanilla TypeScript + CSS.
- Title, description, dan ordered keywords untuk JPG/JPEG, PNG, SVG, EPS, MP4, MOV.
- Salinan sebagai default, dengan pilihan pembaruan sumber dan backup berversi.
- Preview PNG lokal untuk aset aktif, dipakai kembali oleh editor dan AI; EPS memakai TIFF tertanam atau Ghostscript dan video memakai satu poster frame FFmpeg.
- Pembuatan UI langsung dari kode mengikuti referensi persegi dengan warna lembut.

## Struktur dan kontrak

Frontend terdiri dari `main.ts`, `types.ts`, dan `styles.css`. Backend dipisahkan menjadi metadata, adapter SVG, file, dan preview; `lib.rs` mendaftarkan command Tauri. `Cargo.lock` dan `package-lock.json` disimpan bersama source.

| Command | Kontrak |
| --- | --- |
| `inspect_assets(paths, refresh=false)` | `{ assets, folders, files }`: aset unik, root folder/status pemindaian, dan path file impor langsung; pemindaian rekursif tanpa symbolic link. Snapshot diperbarui hanya ketika muat ulang diminta. |
| `read_preview(path)` | Byte gambar untuk file yang sudah diimpor. |
| `get_preview_cache_size()` | Total byte file cache preview. |
| `clear_preview_cache()` | Hapus cache preview dan kembalikan jumlah byte yang dihapus. |
| `save_metadata({ items })` | Patch per file; mode dan root keluaran dibaca dari preferensi sekali per batch; hasil per file. |
| `load_app_preferences()` | Mode simpan, root keluaran, daftar folder, bahasa, dan tema tersimpan. |
| `save_app_preferences(saveMode, outputDir, language, theme)` | Simpan mode/tujuan/bahasa/tema tanpa mengubah daftar folder. |
| `remove_asset_folder(path)` | Hapus referensi root tanpa menghapus file disk. |
| Event `save_progress` | Jumlah file selesai dan total file. |

Error command IPC dan error per aset/folder memakai `{ key, params, detail }` untuk terjemahan frontend; diagnostik eksternal berada pada `detail`.

Patch yang tidak menyertakan field mempertahankan field tersebut. String kosong atau array kosong menghapus nilai secara disengaja. UI mempertahankan baseline sumber secara terpisah dari salinan yang sudah disimpan sehingga penyimpanan salinan berulang tidak kehilangan perubahan sebelumnya.

## Tahapan

1. **Selesai — kerangka dan backend:** struktur root, dokumentasi, command, handler Adobe untuk format biner, dan adapter SVG dengan escaping XML.
2. **Selesai — editor:** impor file/folder/drop, draft per aset, batch field eksplisit, pengurutan keywords, preview ringan, status, dan kedua mode simpan.
3. **Selesai — validasi dan distribusi:** roundtrip seluruh format, integritas media, Unicode, backup, izin file, benturan nama, sumber berubah, kegagalan parsial, dan paket RPM. Tinjauan visual mencakup koreksi selector native dan kontras teks.
4. **Selesai — AI metadata opsional:** generate title/description dan keywords ke draft aset aktif, pengaturan provider dan model, uji koneksi, serta preferensi tersimpan lokal. API key disimpan di credential store OS; permintaan AI hanya berjalan setelah tindakan pengguna.

5. **Selesai — ruang kerja persisten:** dropdown WebKit diperbaiki, status tombol Settings dipulihkan, ikon SVG Lucide lokal, konteks AI per aset, impor folder rekursif/tree berdasarkan format, daftar folder persisten, serta mode dan tujuan simpan melalui Settings.

6. **Selesai — panel dan tampilan:** resize panel dengan pointer/keyboard dan persistensi, satu tombol simpan pengaturan, bahasa ID/EN untuk copy aplikasi dan error IPC terstruktur, tema light/dark, serta migrasi preferensi lama.

7. **Selesai — preview dan cache:** cache PNG yang memakai versi file dan renderer, rasterisasi SVG, fallback Ghostscript untuk EPS, poster frame FFmpeg untuk MP4/MOV, preview vision AI dari cache yang sama, serta ukuran dan tombol hapus cache terpisah di bagian bawah Settings.

Bukti pemeriksaan serta batas pengujian dicatat di [docs/VALIDATION.md](docs/VALIDATION.md). Artefak rilis tersedia di `src-tauri/target/release/`.

## Penyimpanan

Salin ke file sementara → ubah metadata → tutup penulis dengan pelaporan error → baca kembali → cocokkan metadata → pastikan sumber tidak berubah → buat backup bila perlu → finalisasi → sinkronisasi file dan folder. Penulisan sumber tidak dijalankan bila salah satu pemeriksaan sebelumnya gagal.

## Batas verifikasi

Sampel Illustrator diuji hanya pada salinan sementara; preview TIFF mentah harus identik. Jika preview EPS tertanam tidak dapat didekode, Ghostscript menjadi fallback; format EPS atau codec video yang tidak didukung renderer tetap dapat disunting metadatanya tanpa preview. Kompatibilitas setelah dibuka dan disimpan ulang di Illustrator serta pembacaan metadata oleh portal agency memerlukan pemeriksaan eksternal.

Ekspor CSV dan pemutaran video berada di luar versi pertama; preview video hanya satu frame statis. Tidak ada layanan cloud, database, atau framework UI tambahan.
