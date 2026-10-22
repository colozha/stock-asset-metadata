# Hasil validasi

## Workflow GitHub Release RPM — 22 Oktober 2026

- Workflow `release.yml` lulus `actionlint` 1.7.12; seluruh step shell lulus
  `bash -n`. Versi action checkout/setup-node dan inputnya diperiksa terhadap
  repository resmi GitHub Actions.
- Build bersih memakai image `registry.fedoraproject.org/fedora:44` melalui
  Podman, dependensi native dari workflow, Node.js 22.23.4, dan Rust stable
  1.99.1. `npm ci`, tes backend `--locked --no-default-features` (**22 tes**),
  serta `npm run bundle -- --ci -- --locked` lulus, termasuk TypeScript/Vite.
  RPM **0.1.1-1 x86_64** berhasil dibuat; dependensi `ghostscript`/`ffmpeg-free`
  tercantum dan `ldd` tidak menemukan library yang hilang. Checksum `.rpm.sha256`
  dibuat menggunakan nama relatif dan lulus verifikasi di container maupun host.
  Salinan paket, checksum, serta log QA ada di `/tmp/stock-release-qa/`.
- Uji terisolasi menjalankan script validasi versi dari workflow: tag `v0.1.1`
  diterima, versi berbeda/format tidak valid/prerelease/leading zero ditolak.
  Ketidaksesuaian setiap manifest dan lockfile, termasuk root package npm,
  menghentikan proses.
- Step publikasi diuji dengan pengganti lokal untuk HTTP/GitHub CLI: rilis baru
  dibuat sebagai draft sebelum publikasi; draft dilanjutkan dengan unggah ulang;
  rilis publik ditolak tanpa mutasi. Error API, pembuatan draft, atau unggahan
  menghentikan proses sebelum publikasi. Nama aset dengan spasi tetap utuh.
- Query GraphQL pencarian Release juga diperiksa secara read-only pada repository
  tujuan: tag `v0.1.1` belum memiliki Release. GraphQL dipakai karena endpoint
  REST pencarian berdasarkan tag tidak mencakup draft. GitHub Actions aktif dan
  kebijakan repository mengizinkan action yang digunakan.
- Pemeriksaan publikasi di atas tidak mengakses API tulis GitHub. Belum ada
  commit/tag yang dipush atau Release yang dibuat oleh perubahan ini. Eksekusi
  GitHub Actions penuh dan unduhan aset dari Release pertama belum diverifikasi;
  build tidak menggantikan pengujian IPC desktop native.

## Pencarian dari header sidebar — 22 Oktober 2026

- `npm run build` lulus. Perubahan hanya mencakup frontend dan dokumentasi;
  tes backend tidak diulang. Build dan instalasi berikutnya dicatat di Distribusi.
- Binary debug Tauri melalui WebKitWebDriver dengan frontend Vite lokal dan
  konfigurasi XDG/fixture sementara memeriksa input tersembunyi saat awal,
  buka/fokus lewat ikon pada kedua header, pembukaan section yang tertutup,
  filter nama, tutup/reset, clear tanpa menutup, serta state terpisah saat
  berpindah accordion. Enter pada tombol search berhasil membuka/menutup dan
  memindahkan fokus dengan benar. Tidak ada nested button.
- ID/EN × light/dark pada 1440×900 dan 1024×700 menghasilkan 16 capture di
  `/tmp/stock-search-qa/`. Tidak ada overflow horizontal atau runtime error;
  draft dan urutan keyword tetap utuh, termasuk setelah pergantian bahasa/tema.
  Reload mengembalikan pencarian kedua section ke keadaan tersembunyi.

## Verifikasi 22 Oktober 2026: settings, refresh workspace, dan Output

- `npm run build` dan build debug Rust lulus. Backend tanpa fitur desktop lulus
  **22 tes** (12 unit, 10 integrasi); tes lengkap dengan fitur desktop lulus
  **30 tes** (20 unit, 10 integrasi). Regresi baru mencakup pembaruan preferensi
  bersamaan, pemeliharaan field settings saat folder berubah, serta konfigurasi
  yang tetap utuh ketika validasi/penulisan gagal atau file konfigurasi rusak.
- Binary debug **Tauri asli** diperiksa melalui `tauri-driver`/WebKitWebDriver,
  memakai frontend Vite di `http://127.0.0.1:1420`, direktori XDG terisolasi,
  dan fixture SVG sementara. Pemindaian, settings, preview, dan simpan memakai
  IPC Rust asli. Impor 600 file berjalan bersamaan dengan simpan settings:
  settings selesai sebelum impor, dan hasil akhir mempertahankan settings
  terbaru sekaligus root folder baru. Pengeluaran root juga mempertahankan
  settings tersebut.
- Restart memulihkan EN/dark, mode ganti sumber, tujuan output khusus, dan root
  aset. Isi serta waktu modifikasi konfigurasi identik sebelum/sesudah startup;
  startup tidak menulis ulang settings. Dropdown tema dan mode nondefault tetap
  terpilih setelah penerjemahan label serta pembukaan dan penyimpanan Settings.
- Refresh menemukan file baru, melepas entri bersih yang dihapus dari folder,
  mempertahankan draft/konteks/urutan keyword, dan menjaga draft sumber yang
  hilang dengan pesan error. Perubahan eksternal tetap ditolak saat simpan,
  termasuk ketika path sumber juga tampil di Output. Isi perubahan eksternal
  tetap utuh.
- Output menampilkan hasil save tanpa refresh manual, memperbarui folder tujuan
  saat dipindai, dan melaporkan root hilang tanpa membuatnya. Root yang dipulihkan
  dapat dibaca lagi. Metadata read-only, tanpa checkbox/AI/batch/simpan; kembali
  ke Your assets mengembalikan draft. Pencarian nama/ekstensi tidak membedakan
  kapital, pilihan tersembunyi tetap dihitung, dan pencarian/sorting kedua section
  terpisah. Enter pada header native berhasil membuka accordion Output.
- Screenshot kedua section mencakup **ID/EN × light/dark** pada **1440×900 dan
  1024×700** (16 capture), tanpa overflow horizontal atau runtime error pada
  pemeriksaan. Screenshot berada di `/tmp/stock-sidebar-qa/`. Tidak ada perubahan
  dependensi atau permintaan provider AI. Distribusi berikutnya dicatat di bawah.

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

## Verifikasi opsi hasil AI, sorting, dan direktori output

- `npm run build` lulus. Tes backend tanpa fitur desktop lulus 16 tes, termasuk pemeriksaan timestamp filesystem dan field IPC `createdAt`/`modifiedAt`; tes library desktop lulus 14 tes. Build debug Tauri lulus dengan sysroot development Fedora yang dijelaskan di bawah.
- Browser plugin tidak tersedia; pemeriksaan renderer memakai Playwright yang sudah tersedia di cache dengan mock IPC resmi Tauri. Enam kombinasi sorting, urutan angka alami, tanggal kosong/sama, tiga mode hasil AI, kontrol terkunci selama generate, kegagalan AI yang mempertahankan draft, persistensi pilihan, dan fallback preferensi rusak lulus. Pemeriksaan juga mencakup tombol output, error, dan penyembunyiannya pada mode penggantian sumber.
- ID/EN × light/dark diperiksa pada 1440×900 dan 1024×700, termasuk lebar panel minimum 200/300 px. Draft dan urutan keyword tetap utuh; tidak ditemukan error runtime atau overflow horizontal. Label sorting memakai baris tersendiri agar nama kolom tanggal terbaca pada sidebar minimum; pesan daftar kosong mengikuti alur layout di bawah kontrol sorting.
- Binary debug Tauri asli melalui `tauri-driver`/WebKitWebDriver memakai XDG config/data/cache terisolasi dan frontend Vite lokal. Timestamp `modifiedAt` cocok dengan filesystem, `createdAt` berupa angka atau null, command dan tombol membuka direktori melalui `xdg-open`, serta direktori hilang mengembalikan error IPC terstruktur tanpa membuat folder. Screenshot native ID/light dan EN/dark pada kedua ukuran tersimpan di `/tmp/stock-request-native-{id,en}-{1440,1024}.png`; screenshot renderer mencakup keempat kombinasi bahasa/tema di `/tmp/stock-request-{id,en}-{light,dark}-{1440,1024}.png`.
- Hasil AI pada pemeriksaan ini memakai mock; tidak ada permintaan ke provider eksternal.

## Verifikasi alur simpan + CSV Shutterstock

18 Oktober 2026. Verifikasi alur simpan dengan ekspor CSV opsional.

- `npm run build` lulus. Backend tanpa fitur desktop lulus **20 tes** (10 unit,
  10 integrasi); tes library dengan fitur desktop lulus **18 tes**. Cakupan baru
  meliputi validasi patch sebelum penulisan, CSV atomik dengan suffix tanpa
  overwrite, pengelompokan output, pengabaian aset gagal/nonvideo, dan kelanjutan
  ekspor folder lain setelah satu folder gagal.
- Binary debug **Tauri asli** dijalankan melalui `tauri-driver`/WebKitWebDriver
  dengan fixture dan direktori XDG sementara. Tombol simpan dan Ctrl+S membuka
  popup video; checkbox awalnya mati, kategori wajib hanya ketika ekspor aktif,
  Batal/Escape membatalkan seluruh aksi, dan gambar saja langsung disimpan.
- Metadata CSV tidak valid ditolak sebelum folder/file output dibuat. Mematikan
  ekspor memungkinkan metadata tetap disimpan. Batch campuran menyimpan SVG dan
  video; CSV hanya memuat video berhasil dan dibaca independen dengan parser
  Python `csv`. Nama output bersuffix cocok dengan CSV, CSV sebelumnya tetap
  identik, serta hash semua sumber tetap identik dalam mode salinan.
- Mode ganti sumber membuat backup yang cocok dengan sumber awal; MP4/MOV pada
  folder sumber yang sama masuk satu CSV. Konflik perubahan sumber menggagalkan
  video terkait, sementara aset lain tetap tersimpan dan CSV hanya memuat video
  yang berhasil.
- Screenshot native mencakup ID/EN × light/dark pada 1440×900 dan 1024×700.
  Popup dan tombol berada dalam viewport tanpa overflow horizontal. Pergantian
  bahasa/tema mempertahankan draft. Satu konfirmasi memastikan ringkasan jumlah
  aset/video, kategori terpilih, dan footer tidak menampilkan proses sebelum
  pengguna mengonfirmasi simpan.
- Tidak ada upload atau validasi portal Shutterstock pada pemeriksaan ini.
  Paket RPM dan aplikasi terinstal tidak diperbarui oleh perubahan alur ini.

Hasil impor CSV pada portal Shutterstock belum diverifikasi. Ekspor tidak
mengunggah apa pun atau memanggil provider AI.

## Distribusi

Pembaruan lanjutan **22 Oktober 2026** membangun dan memasang ulang RPM
**0.1.1-1 x86_64**, termasuk input search tersembunyi dan ikon search di kedua
header sidebar. `npm run bundle` dan `pkexec rpm -Uvh --replacepkgs` berhasil;
`rpm -V` bersih dan semua library teresolusi. SHA-256 executable terpasang cocok
dengan payload RPM: `0455ade606f12c1ac5babebb11ab5e0317e62c128ac9d01ea9c7f927aa838065`.
Smoke check binary terpasang dari `tauri://localhost` memverifikasi kondisi
search awal tersembunyi, buka/fokus, filter, tutup/reset, ikon Output membuka
accordion dan inputnya, serta refresh dan metadata Output read-only.
Pemeriksaan menggunakan konfigurasi dan fixture sementara di
`/tmp/stock-search-install-qa/`, tanpa mengubah settings atau aset kerja pengguna.

Pada **22 Oktober 2026**, `npm run bundle` membangun ulang RPM
**0.1.1-1 x86_64** dari perubahan settings, refresh workspace, accordion Output,
dan pencarian nama. Paket dipasang melalui `pkexec rpm -Uvh --replacepkgs`.
`rpm -V stock-asset-metadata` bersih; SHA-256 executable terpasang cocok dengan
payload RPM: `6fa917aaddb61a7b7de5b50d9071c2ac3e0a1c10ac471b445d860afcf2a19380`.
Semua library executable teresolusi dan dependensi Ghostscript/FFmpeg tetap ada.

Smoke check `/usr/bin/stock-asset-metadata` melalui WebKitWebDriver memakai
konfigurasi XDG dan fixture sementara: aplikasi membuka frontend terbundel dari
`tauri://localhost`, memulihkan root aset/output, menemukan file baru lewat
Refresh, dan menampilkan Output dengan metadata read-only serta AI/simpan
nonaktif. Screenshot berada di `/tmp/stock-install-qa/installed-output.png`.
Tidak ada perubahan pada settings atau aset kerja pengguna dalam smoke check.

Paket RPM **0.1.1-1 x86_64** dibangun ulang pada 18 Oktober 2026 untuk fitur
ekspor CSV Shutterstock. `npm run bundle` lulus; smoke check binary release
melalui WebKitWebDriver membuka aset dan dialog ekspor dari `tauri://localhost`
(frontend terbundel). Dependensi RPM tetap Ghostscript, FFmpeg, GTK, dan WebKitGTK;
ExifTool tidak ditambahkan. Paket berada di
`src-tauri/target/release/bundle/rpm/Stock Asset Metadata-0.1.1-1.x86_64.rpm`.
Paket hasil perubahan CSV ini belum dipasang ke sistem.

RPM **0.1.1-1 x86_64** dibangun ulang dan dipasang pada 18 October 2026 16:21 WIB, termasuk opsi hasil AI, sorting per grup, dan tombol direktori output. Instalasi memakai `pkexec rpm -Uvh --replacepkgs`. `rpm -V stock-asset-metadata` bersih; SHA-256 `/usr/bin/stock-asset-metadata` cocok dengan manifest RPM (`b25692d727de7ee48f6774f58c5c25f7a8ca92a9a2827c478c6e55635a211480`). Semua library executable terinstal teresolusi.

`npm run bundle` menghasilkan executable dan paket RPM x86_64 pada `src-tauri/target/release/`. Pemeriksaan library executable tidak menemukan dependency yang hilang atau dependency ke sysroot sementara.

Paket RPM telah dibangun ulang dari kode terbaru pada 18 Oktober 2026 pukul 14:14 WIB, termasuk perubahan Settings/folder/konteks AI, lalu dipasang ulang pada pukul 14:15 WIB sebagai `stock-asset-metadata-0.1.0-1.x86_64`. Instalasi melalui `pkexec rpm -Uvh --replacepkgs` berhasil. `rpm -V` tidak melaporkan perbedaan; SHA-256 executable terinstal cocok dengan manifest RPM baru, dan seluruh library teresolusi.

Distribusi **0.1.1-1** dibangun dari perubahan panel/bahasa/tema terbaru dan dipasang melalui `pkexec dnf -y install` pada 18 Oktober 2026 sekitar 14:48 WIB, menggantikan 0.1.0-1. `rpm -q` mengonfirmasi `stock-asset-metadata-0.1.1-1.x86_64`; `rpm -V` bersih dan SHA-256 executable `/usr/bin/stock-asset-metadata` cocok dengan digest payload RPM (`ebdfa2359bc92a717b365fa32a3746d7a054b360f7aee022585c4649cf3cd61f`). Semua library executable teresolusi. Smoke check binary terinstal melalui WebKitWebDriver berhasil membuka editor/Settings, menemukan dua separator, satu tombol save, serta footer save yang terlihat. DNF melaporkan antrean pembaruan offline yang sebelumnya tertunda dibatalkan oleh transaksi instalasi ini.

Header development Fedora belum terpasang pada sistem pengujian. Build lokal menggunakan paket development Fedora resmi yang diekstrak ke `/tmp/stock-asset-metadata-sysroot`, tanpa mengubah paket sistem:

```bash
PKG_CONFIG_PATH=/tmp/stock-asset-metadata-sysroot/usr/lib64/pkgconfig:/tmp/stock-asset-metadata-sysroot/usr/share/pkgconfig CARGO_BUILD_JOBS=3 npm run bundle
```

Path sementara tersebut hanya diperlukan untuk build pada sesi ini. Untuk lingkungan development biasa, ikuti prasyarat Fedora di README.

## Batas bukti

Pada 18 Oktober 2026, pengguna menguji lima varian MP4 pada Shutterstock:
kontrol XMP, ItemList, UserData parsial, Keys, dan gabungan. Verifikasi lokal
dengan ExifTool 13.59 memastikan tag yang ditulis terbaca kembali, packet XMP
dan 40 keyword berurutan tetap utuh, serta payload `mdat` dan sumber identik.
Seluruh varian pada screenshot pengguna menampilkan description kosong
(`0/2048`) dan nol keyword (`0/50`). Hasil ini tidak membuktikan bahwa semua
variasi metadata MP4 ditolak oleh Shutterstock.

Karena tidak ada profil yang berhasil, integrasi writer tambahan dibatalkan
dan ExifTool tidak menjadi dependensi aplikasi/RPM. Pengguna memilih ekspor CSV
Shutterstock sebagai solusi terpisah; petunjuk tersedia di README. Hasil portal
merupakan pengamatan pengguna, bukan pengujian IPC/portal otomatis oleh agent.
Alat dan panduan eksperimen telah dihapus karena tidak digunakan dalam alur aplikasi.

Preview TIFF palet dengan alpha pada beberapa EPS Illustrator dialihkan ke Ghostscript; EPS tetap bisa tidak memiliki preview bila renderer gagal. Frame video memerlukan codec yang tersedia pada FFmpeg dan aplikasinya tidak menyediakan pemutar video. SVG dibatasi UTF-8.

Pengujian portal terbatas pada laporan pengguna: Adobe Stock membaca metadata
MP4 awal, sedangkan Shutterstock tidak membaca metadata kelima varian di atas.
Belum dilakukan buka/simpan ulang di Illustrator, pengujian distribusi selain
Fedora 44, atau sistem operasi lain. Pemeriksaan lokal XMP tidak menjamin tiap
portal membaca field yang sama.
