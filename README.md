# Stock Asset Metadata

Aplikasi desktop lokal untuk membaca dan menanamkan **title**, **description**, dan **keywords** pada JPG/JPEG, PNG, SVG, EPS, MP4, dan MOV. Antarmuka memakai panel persegi dan warna lembut sesuai referensi. Versi awal ditujukan untuk Linux/Fedora.

## Menjalankan

Prasyarat: Node.js 22.12+, npm, Rust stable, compiler C++, serta dependensi Tauri Linux. Lingkungan pengembangan yang digunakan: Fedora 44, Rust 1.99, Node.js 22.23.

```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel librsvg2-devel libxdo-devel gcc-c++ make ghostscript ffmpeg-free
npm ci
npm run desktop
```

Seluruh command npm dijalankan dari direktori project ini. `npm run dev` hanya membuka pratinjau frontend di `http://127.0.0.1:1420`; akses file dan penyimpanan memerlukan aplikasi desktop Tauri.

## Penggunaan

1. Tambahkan file, pilih folder, atau seret file/folder ke jendela. Folder dibaca rekursif; symbolic link dilewati. Sidebar menampilkan tree folder/subfolder dengan kelompok format (JPEG digabung ke JPG). Lokasi folder disimpan lokal dan dibaca ulang saat aplikasi dibuka; file impor langsung hanya bertahan selama sesi.
2. Pilih aset lalu isi metadata. Keywords dipisahkan koma; frasa dengan spasi tetap satu keyword. Tombol panah mengubah urutan. Duplikat dihapus dengan mempertahankan kemunculan pertama.
3. Untuk perubahan batch, centang beberapa aset. Form menyunting aset aktif. Centang field yang ingin disalin dan pilih **Terapkan field ke pilihan**; field lain pada aset lain dipertahankan.
4. Atur **Mode penyimpanan** dan **Direktori salinan** lewat **Settings → Penyimpanan**, lalu pilih **Simpan pengaturan**. Defaultnya **Simpan salinan** ke direktori Documents sistem (atau `~/Documents` bila belum dikonfigurasi), dalam `stock-assets/[tipe file]`: `jpg`, `png`, `svg`, `eps`, `mp4`, dan `mov`. Direktori dibuat saat menyimpan. Tombol simpan memproses aset terpilih yang memiliki perubahan tanpa meminta tujuan lagi; footer memperlihatkan mode dan tujuan aktif.

Seret pembatas di kanan daftar aset atau kiri panel metadata untuk mengubah lebarnya. Pembatas menampilkan grip dan garis ungu saat disorot, difokuskan, atau diseret. Setelah fokus, gunakan panah kiri/kanan (Shift untuk langkah lebih besar), Home/End untuk batas, atau klik ganda untuk kembali ke lebar awal. Ukuran tersimpan lokal dan disesuaikan saat jendela diperkecil.

Di **Pengaturan → Tampilan**, pilih **ID — Indonesia** atau **EN — English**, serta tema **Terang / Light** atau **Gelap / Dark**. Satu tombol **Simpan pengaturan / Save settings** menyimpan tampilan, penyimpanan, dan perubahan konfigurasi AI. Bahasa dan tema berlaku setelah disimpan; kembali ke editor membuang perubahan pengaturan yang belum disimpan. Default konfigurasi lama adalah Indonesia dan light. Metadata pengguna tidak diterjemahkan dan hasil AI tetap English. AI tetap opsional: konfigurasi provider yang belum lengkap disimpan tanpa mengaktifkannya. Jika penyimpanan AI gagal setelah preferensi aplikasi tersimpan, pesan menjelaskan hasil parsial dan input AI tetap tersedia untuk dicoba kembali.

Bagian **Cache preview / Preview cache** di bagian bawah Settings menampilkan ukuran cache preview dan menyediakan tombol hapus terpisah dari penyimpanan pengaturan. Preview disimpan lokal setelah aset aktif dibuka, digunakan kembali saat dipilih atau dikirim ke AI, dan dibuat ulang setelah file sumber berubah.

Gunakan **Keluarkan folder dari daftar** (ikon silang pada root folder) untuk melepas lokasi tersimpan tanpa menghapus file di disk. Folder yang hilang tetap ditampilkan dengan pesan kesalahan. Draft yang akan dilepas memerlukan konfirmasi; aset yang masih tercakup folder lain atau diimpor langsung tetap tersedia. Pemindaian berjalan saat startup/impor, tanpa pemantauan filesystem otomatis.

Gunakan **Generate Title + Description** atau **Generate Keywords** untuk mengisi draft aset aktif. Isi **Konteks tambahan (opsional)** untuk memberi referensi subjek atau penggunaan bagi kedua aksi. Konteks terpisah per aset, bertahan saat berpindah aset selama sesi, dan tidak ditanamkan sebagai metadata atau disimpan setelah aplikasi ditutup. Pilih panjang description (Short, Medium, Long) dan jumlah keywords 10–50. Hasil AI tidak langsung ditanamkan; tinjau draft lalu simpan seperti perubahan metadata lain.

Pengaturan provider tersedia lewat tombol **Settings**. Daftar model dapat dimuat dari provider, dan ID model juga dapat diisi manual. Endpoint, model, provider aktif, serta preferensi generator disimpan lokal di perangkat. API key disimpan melalui credential store sistem operasi dan tidak ditulis ke file pengaturan aplikasi. Di Linux, sesi desktop harus menyediakan credential store yang kompatibel dengan Secret Service.

`Ctrl+O` menambah file. `Ctrl+S` menyimpan perubahan terpilih. Draft bertahan selama sesi; aplikasi meminta konfirmasi ketika ditutup dengan perubahan yang belum disimpan.

Simpan salinan tidak memperbarui file sumber. Mengganti mode ke pembaruan sumber memperlihatkan kembali perubahan yang belum diterapkan pada sumber tersebut.

Sorting sidebar tersedia melalui **Urutkan** berdasarkan nama, tanggal dibuat, atau tanggal diubah, dengan arah naik/turun. Urutan hanya berlaku di dalam setiap grup format. Tanggal mengikuti filesystem saat impor/muat ulang; tanggal yang tidak tersedia selalu diletakkan terakhir. Kolom dan arah urutan tersimpan lokal.

Pada panel AI, **Hasil judul & deskripsi** menyediakan **Terpisah**, **Gunakan judul**, atau **Gunakan deskripsi**. Dua pilihan terakhir memakai hasil terpilih untuk kedua field setelah generate berhasil. Pilihan tersimpan lokal; perubahan pilihan tidak menyalin draft saat ini dan edit manual tetap bebas.

Footer mode salinan menampilkan **Direktori output** beserta tombol ikon folder untuk membuka root output tersimpan di file manager Linux (`xdg-open`). Direktori yang belum ada dilaporkan sebagai error; folder tetap dibuat oleh alur simpan seperti biasa.

## Perlindungan file

Setiap penulisan dilakukan pada file sementara di filesystem tujuan. Metadata dibaca kembali dan dibandingkan sebelum hasil difinalisasi. Mode pembaruan membuat backup `file.ext.bak`, lalu `.bak.1`, dan seterusnya. File keluaran yang bertabrakan mendapat suffix `_1`, `_2`, dan seterusnya. Sumber yang berubah sejak diimpor ditolak; gunakan tombol muat ulang setelah memeriksa draft.

Data gambar dan stream video tidak melewati encoder ketika metadata disimpan. Preview PNG dibuat terpisah untuk aset aktif dan disimpan di cache aplikasi. Aplikasi tidak mengunggah aset secara otomatis dan tidak memakai server aplikasi atau database. Jika pengguna menekan tombol generate atau Test Connection, aplikasi mengirim permintaan ke provider terpilih; generate menyertakan nama aset, metadata, dan konteks tambahan jika diisi. Untuk model yang mendukung vision dan memenuhi batas ukuran yang ada, generate juga dapat mengirim preview PNG JPG, PNG, SVG, EPS, MP4, atau MOV. Permintaan dapat menggunakan kuota atau biaya provider, dan mengikuti kebijakan data provider tersebut. Keberhasilan satu file tidak menyembunyikan kegagalan file lain.

## Dukungan format

| Format | Metadata | Preview |
| --- | --- | --- |
| JPG/JPEG | Adobe XMP, termasuk rekonsiliasi metadata legacy oleh SDK | Thumbnail dengan orientasi EXIF |
| PNG | XMP standar | Thumbnail, termasuk transparansi |
| SVG UTF-8 | XMP, title/desc dokumen, serta pembacaan Dublin Core sederhana | PNG dirender lokal dari SVG |
| EPS | Handler PostScript Adobe; EPS ASCII dan DOS biner | TIFF tertanam; Ghostscript membuat preview jika TIFF tidak dapat dipakai |
| MP4/MOV | XMP di container melalui handler MPEG-4/QuickTime Adobe | Satu frame statis dari video melalui FFmpeg |

Jika preview TIFF tertanam EPS tidak dapat didekode, Ghostscript digunakan sebagai fallback. JPEG/PNG dan video dibatasi 128 MiB; fallback Ghostscript memproses EPS hingga 128 MiB; preview TIFF tertanam EPS dibatasi 32 MiB; SVG untuk edit metadata dibatasi 64 MiB. Alokasi decoder dibatasi 512 MiB dan preview dibatasi 1600 piksel pada sisi terpanjang. Render Ghostscript dan FFmpeg berhenti setelah 30 detik. Video tanpa frame yang dapat didekode atau codec yang tidak tersedia tetap bisa disunting metadatanya, tetapi tidak menghasilkan preview.

Validasi lokal belum membuktikan roundtrip buka/simpan di Illustrator. Ekspor CSV tersedia khusus video MP4/MOV untuk Shutterstock; pemutaran video belum termasuk versi ini.

Untuk MP4 yang metadatanya terbaca Adobe Stock tetapi kosong di Shutterstock,
tersedia [hasil eksperimen tag native MP4](docs/VALIDATION.md#batas-bukti). Pada uji
pengguna, kelima varian tetap menghasilkan description/keyword kosong di
Shutterstock. Belum ada profil tambahan yang terbukti; keberhasilan embedding
XMP bukan jaminan kompatibilitas portal.

### CSV video untuk Shutterstock

1. Pilih aset yang memiliki perubahan, lalu klik **Simpan metadata** atau tekan **Ctrl+S**.
   Jika mencakup video MP4/MOV, dialog **Simpan metadata video** muncul sebelum penyimpanan.
2. Centang **Ekspor CSV Shutterstock**, lalu pilih satu kategori untuk seluruh video dalam batch.
   Checkbox awalnya mati; tanpa ekspor, metadata tetap dapat disimpan seperti biasa.
3. Klik **Simpan metadata**. CSV dibuat otomatis di setiap folder output video yang berhasil
   disimpan, tanpa dialog lokasi. Mode salinan memakai folder format MP4/MOV; mode ganti
   sumber memakai folder sumber. Gambar tetap disimpan tetapi tidak dimasukkan ke CSV.
4. Nama default CSV adalah `shutterstock.csv`; jika sudah ada, gunakan `shutterstock_1.csv`,
   dan seterusnya tanpa menimpa file lama. Baris CSV memakai nama output final, termasuk suffix
   benturan nama video, serta description dan urutan keyword hasil simpan.
5. Unggah video terlebih dahulu, lalu gunakan **Upload CSV** pada halaman pengajuan Shutterstock.

CSV berisi `Filename,Description,Keywords,Categories` dalam UTF-8. Deskripsi minimal 5 kata/
maksimal 2.048 karakter dan 7–50 keyword unik diperlukan; validasi dilakukan sebelum aset
ditulis jika ekspor aktif. Jika tidak valid, perbaiki draft atau matikan ekspor untuk melanjutkan.
Batal/Escape membatalkan penyimpanan. Pilihan ekspor dan kategori direset setiap dialog dibuka.

Kegagalan CSV dilaporkan beserta foldernya, terpisah dari hasil simpan aset; video yang sudah
tersimpan tetap tersedia. CSV hanya mencakup video berhasil dari aksi tersebut dan tidak
menggabungkan isi CSV lama. Metadata tidak diterjemahkan; Shutterstock meminta description
dan keyword English. Ekspor agency lain belum disediakan. Lihat
[panduan Shutterstock](https://submit.shutterstock.com/help/en/articles/10617486-how-do-i-include-existing-metadata-with-my-content-submission).

## Build dan pemeriksaan

```bash
npm run check
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --no-default-features
npm run bundle
```

Pengujian backend dapat dijalankan tanpa WebKitGTK melalui `--no-default-features`. Tes prompt AI memerlukan fitur desktop: `cargo test --manifest-path src-tauri/Cargo.toml --lib` (dengan dependensi native terpasang). Paket RPM berada di `src-tauri/target/release/bundle/rpm/`; executable berada di `src-tauri/target/release/stock-asset-metadata`.

Fixture video adalah aset sintetis kecil. Preview EPS tanpa TIFF dan preview MP4/MOV memerlukan Ghostscript dan FFmpeg saat aplikasi berjalan; paket RPM mencantumkan `ghostscript` dan `ffmpeg-free` sebagai dependensi.

## Struktur

- `src/`: editor TypeScript dan CSS, dengan kontrak data di `types.ts`, kamus ID/EN di `messages.ts`, penerjemahan di `i18n.ts`, dan resize panel di `layout.ts`.
- `src-tauri/src/metadata.rs`: model dan integrasi Adobe XMP Toolkit 1.12.1.
- `src-tauri/src/svg.rs`: adapter XML yang mempertahankan byte artwork.
- `src-tauri/src/files.rs`: impor, deteksi perubahan sumber, file sementara, dan backup.
- `src-tauri/src/preview.rs`: cache PNG, thumbnail, rasterisasi SVG, preview EPS, dan frame poster video.
- `src-tauri/src/preferences.rs`: konfigurasi lokal mode simpan, direktori salinan, dan root folder.
- Ikon antarmuka menggunakan SVG lokal Lucide; atribusi dan lisensi ada di [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md). Renderer SVG memakai resvg 0.48.1.
- `src-tauri/tests/`: pengujian roundtrip, integritas, dan kegagalan penyimpanan.

Keputusan produk tersedia di `PRODUCT.md`, aturan visual di `DESIGN.md`, dan status implementasi di `IMPLEMENTATION_PLAN.md`. Rincian bukti pengujian serta batasnya tersedia di [docs/VALIDATION.md](docs/VALIDATION.md).
