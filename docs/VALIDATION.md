# Hasil validasi

Tanggal: 18 Oktober 2026. Lingkungan: Fedora 44 x86_64, Rust 1.99, Node.js 22.23, WebKitGTK 2.54.0. Target awal adalah desktop Linux.

## Backend

Command berikut lulus: **14 pengujian** (4 unit preferensi/terjemahan dan 10 integrasi format/penyimpanan). Pengujian backend menggunakan fixture sintetis yang disimpan di repository atau dibuat pada direktori sementara.

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

## Verifikasi tambahan: Settings, folder, dan konteks AI

- `npm run check`, `npm run build`, dan kompilasi desktop Rust lulus. Tes lengkap dengan fitur desktop `cargo test --manifest-path src-tauri/Cargo.toml` lulus **18 pengujian** (8 unit dan 10 integrasi), termasuk konteks opsional pada kedua prompt dan preferensi aplikasi.
- Tes backend tambahan memeriksa konfigurasi default/roundtrip/error, pemindaian subfolder rekursif, deduplikasi root bertumpuk dan impor file, symbolic link siklik yang dilewati, root tidak tersedia, normalisasi JPEG, pembuatan subdirektori tujuan, serta tujuan terhalang file biasa.
- Renderer Chromium dengan mock IPC resmi Tauri diperiksa pada 1440×900 dan 1024×700: tree folder/format, konteks terpisah per aset dan terkirim pada kedua aksi, mode/tujuan pulih setelah reload, pengeluaran folder mempertahankan file impor langsung, serta simpan tanpa dialog tujuan. Tidak ditemukan error runtime atau overflow horizontal. Tombol Settings kembali aktif setelah kegagalan koneksi, simpan preferensi, dan muat pengaturan. Semua dropdown memakai padding vertikal nol.
- Aplikasi **debug Tauri asli** diperiksa melalui `tauri-driver`/WebKitWebDriver dengan konfigurasi XDG terisolasi: impor folder berisi enam format, restart memulihkan lokasi dan aset, salinan masuk direktori format dan terbaca kembali, benturan nama memperoleh suffix, backup cocok dengan sumber dan izin `0600` terjaga, konflik perubahan eksternal ditolak, serta tujuan terhalang menghasilkan kegagalan tanpa mengubah sumber. Root yang dipindahkan tetap ditampilkan dengan kesalahan; mengeluarkannya dari daftar tidak menghapus file/backup disk.
- Kedua aksi generator diperiksa melalui IPC Rust dan server HTTP tiruan **lokal** sebagai Ollama. Prompt menyertakan konteks tambahan; tidak ada permintaan ke provider eksternal atau penggunaan API key nyata dalam pengujian ini.
- Sistem tanpa konfigurasi XDG Documents memakai fallback `~/Documents`. Daftar folder dan preferensi simpan tersimpan di `app-preferences.json`; credential AI tetap terpisah.
- Tinjauan screenshot desktop/compact dan native mengikuti gaya panel yang ada. Detektor desain hanya memberi advisory atas warna/ukuran huruf pada aturan lama; drift sistem desain tersebut tidak diperbaiki dalam pekerjaan ini.

## Verifikasi 0.1.1: resize, bahasa, tema, dan satu tombol simpan

- `npm run build` lulus tanpa peringatan parser HTML. Tes backend tanpa fitur desktop lulus 14 pengujian; tes library desktop lulus 11 pengujian, termasuk AI, migrasi konfigurasi lama, roundtrip bahasa/tema, dan kesetaraan key/parameter ID/EN untuk seluruh error backend.
- Chromium dengan mock IPC resmi Tauri memeriksa drag pointer dan keyboard separator, batas lebar, reset, persistensi setelah reload, satu tombol simpan, penerapan bahasa/tema hanya setelah save, pembatalan melalui Back/Escape, serta draft yang tetap utuh. Kegagalan simpan AI setelah preferensi aplikasi berhasil menampilkan hasil parsial, mempertahankan input, dan dapat dicoba kembali. Provider yang tidak berubah tidak memicu command simpan AI. Tidak ditemukan error runtime atau overflow horizontal pada 1440×900 dan 1024×700.
- Binary **release Tauri 0.1.1** diperiksa melalui `tauri-driver` dan WebKitWebDriver, dengan direktori XDG config/data/cache terisolasi dan fixture sementara. UI simpan tunggal berhasil menyimpan ID/EN, light/dark, dan konfigurasi AI yang belum lengkap tanpa mengaktifkannya. Pergantian bahasa mempertahankan draft, konteks, dan urutan keyword; restart memulihkan bahasa/tema dan lebar panel.
- Screenshot native mencakup editor dan Settings pada 1440×900 serta 1024×700 untuk ID/light dan EN/dark. Satu batch perbaikan menempatkan tombol simpan pada footer Settings yang tetap terlihat, merapikan jarak label, dan menyusun pilihan bahasa/tema berdampingan; konfirmasi visual selesai.
- IPC error mengirim key/parameter terstruktur. Impor, simpan salinan, dan readback JPG, PNG, SVG, EPS, MP4, dan MOV melalui backend release berhasil; byte semua sumber tetap identik. Uji ini tidak memakai provider eksternal atau menulis credential nyata.

## Verifikasi preview/cache

- `cargo test --manifest-path src-tauri/Cargo.toml --no-default-features` lulus 16 tes (6 unit, 10 integrasi). Tes preview merender fixture SVG, EPS tanpa TIFF tertanam, MP4, dan MOV menjadi PNG; tes cache memeriksa reuse, invalidasi setelah sumber berubah, ukuran total, dan penghapusan.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib` lulus 14 tes. Tes AI memeriksa gambar dikirim hanya pada model vision dan tetap mematuhi batas ukuran preview.
- Binary release Tauri diuji melalui `tauri-driver` + WebKitWebDriver dengan XDG config/data/cache terisolasi. IPC mengimpor fixture SVG/EPS/MP4/MOV dan menghasilkan empat preview (total cache 1,116 byte). Tombol Clear Preview Cache menghapusnya hingga ukuran 0 B dan tidak menyimpan perubahan bahasa yang belum disimpan.
- Settings diperiksa pada 1440×900 dan 1024×700 dalam kombinasi ID/light, ID/dark, EN/light, dan EN/dark. Tombol cache tetap terlihat, tidak ada overflow horizontal pada ukuran compact, dan footer Save settings tidak menutupi section cache. Screenshot pemeriksaan disimpan di `/tmp/preview-settings-*.png`.
- `npm run build` lulus. `npm run bundle` menghasilkan RPM x86_64; `rpm -qp --requires` mengonfirmasi `ghostscript` dan `ffmpeg-free` tercantum sebagai dependensi.
- Tidak ada permintaan ke provider AI eksternal dalam verifikasi ini. Browser plugin dan Playwright tidak tersedia di lingkungan; pemeriksaan UI memakai WebKitWebDriver pada binary Tauri asli.

## Distribusi

`npm run bundle` menghasilkan executable dan paket RPM x86_64 pada `src-tauri/target/release/`. Pemeriksaan library executable tidak menemukan dependency yang hilang atau dependency ke sysroot sementara.

Paket RPM telah dibangun ulang dari kode terbaru pada 18 Oktober 2026 pukul 14:14 WIB, termasuk perubahan Settings/folder/konteks AI, lalu dipasang ulang pada pukul 14:15 WIB sebagai `stock-asset-metadata-0.1.0-1.x86_64`. Instalasi melalui `pkexec rpm -Uvh --replacepkgs` berhasil. `rpm -V` tidak melaporkan perbedaan; SHA-256 executable terinstal cocok dengan manifest RPM baru, dan seluruh library teresolusi.

Distribusi **0.1.1-1** dibangun dari perubahan panel/bahasa/tema terbaru dan dipasang melalui `pkexec dnf -y install` pada 18 Oktober 2026 sekitar 14:48 WIB, menggantikan 0.1.0-1. `rpm -q` mengonfirmasi `stock-asset-metadata-0.1.1-1.x86_64`; `rpm -V` bersih dan SHA-256 executable `/usr/bin/stock-asset-metadata` cocok dengan digest payload RPM (`ebdfa2359bc92a717b365fa32a3746d7a054b360f7aee022585c4649cf3cd61f`). Semua library executable teresolusi. Smoke check binary terinstal melalui WebKitWebDriver berhasil membuka editor/Settings, menemukan dua separator, satu tombol save, serta footer save yang terlihat. DNF melaporkan antrean pembaruan offline yang sebelumnya tertunda dibatalkan oleh transaksi instalasi ini.

Header development Fedora belum terpasang pada sistem pengujian. Build lokal menggunakan paket development Fedora resmi yang diekstrak ke `/tmp/stock-asset-metadata-sysroot`, tanpa mengubah paket sistem:

```bash
PKG_CONFIG_PATH=/tmp/stock-asset-metadata-sysroot/usr/lib64/pkgconfig:/tmp/stock-asset-metadata-sysroot/usr/share/pkgconfig CARGO_BUILD_JOBS=3 npm run bundle
```

Path sementara tersebut hanya diperlukan untuk build pada sesi ini. Untuk lingkungan development biasa, ikuti prasyarat Fedora di README.

## Batas bukti

Preview TIFF palet dengan alpha pada beberapa EPS Illustrator dialihkan ke Ghostscript; EPS tetap bisa tidak memiliki preview bila renderer gagal. Frame video memerlukan codec yang tersedia pada FFmpeg dan aplikasinya tidak menyediakan pemutar video. SVG dibatasi UTF-8.

Belum dilakukan pengujian unggah ke portal microstock, buka/simpan ulang di Illustrator, distribusi selain Fedora 44, atau sistem operasi lain. Pemeriksaan lokal XMP tidak menjamin tiap portal membaca field yang sama.
