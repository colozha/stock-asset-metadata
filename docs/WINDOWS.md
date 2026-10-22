# Windows x64

## Instalasi

1. Unduh pasangan `Stock Asset Metadata_<versi>_x64-setup.exe` dan `.exe.sha256`
   dari [GitHub Releases](https://github.com/colozha/stock-asset-metadata/releases).
2. Buka PowerShell di folder unduhan dan periksa checksum (sesuaikan nama file):

   ```powershell
   $file = 'Stock Asset Metadata_0.1.1_x64-setup.exe'
   $expected = (Get-Content "$file.sha256" -Raw).Split(' ')[0]
   if ((Get-FileHash $file -Algorithm SHA256).Hash -ne $expected) {
       throw 'Checksum tidak cocok. Unduh ulang installer.'
   }
   ```

3. Setelah checksum cocok, jalankan installer dan ikuti petunjuknya. Aplikasi
   dipasang untuk pengguna saat ini. Installer mengunduh Microsoft WebView2 bila
   belum terpasang, sehingga instalasi pertama dapat memerlukan internet.
4. Buka **Stock Asset Metadata** dari Start Menu. Node.js, npm, Rust, dan Visual
   Studio tidak diperlukan untuk menjalankan aplikasi.

Target installer adalah Windows 10/11 x64; ARM dan Windows 32-bit belum termasuk.
Installer belum ditandatangani dengan sertifikat penerbit; Windows dapat
menampilkan peringatan penerbit tidak dikenal. Gunakan aset dari repository ini
dan cocokkan checksum sebelum menjalankannya.

### Preview EPS dan video

Edit metadata dan ekspor CSV tidak membutuhkan program tambahan berikut.
Untuk preview lengkap, pasang:

- **Ghostscript 64-bit**, dari [situs resmi Ghostscript](https://ghostscript.com/releases/gsdnld.html).
  Instalasi standar di `C:\Program Files\gs\<versi>\bin\gswin64c.exe`
  dideteksi otomatis; untuk lokasi khusus, tambahkan folder `bin` ke `PATH`.
- **FFmpeg**, melalui tautan Windows pada [halaman unduhan FFmpeg](https://ffmpeg.org/download.html).
  Tambahkan folder yang berisi `ffmpeg.exe` ke `PATH`.

Mulai ulang aplikasi setelah pemasangan. Tanpa Ghostscript, EPS dengan TIFF
tertanam masih dapat menampilkan preview; tanpa FFmpeg, preview MP4/MOV tidak
tersedia. Penyuntingan metadata tetap tersedia.

## Build lokal di Windows

Prasyarat: Git, Node.js 22.12+ dan npm, Rust stable **MSVC x64**, serta Visual
Studio Build Tools dengan workload **Desktop development with C++** dan Windows
SDK. Lihat [prasyarat Tauri Windows](https://v2.tauri.app/start/prerequisites/).

Dari root repository, jalankan di PowerShell:

```powershell
.\scripts\build-windows.ps1
```

Script menjalankan `npm ci`, build frontend dan NSIS, lalu memeriksa versi dan
arsitektur executable serta membuat checksum SHA-256. Tauri menyiapkan tooling
NSIS yang diperlukan; build awal memerlukan internet. Hasilnya berada di
`src-tauri/target/release/bundle/nsis/`.

Untuk menjalankan tes backend beserta roundtrip format, pasang Ghostscript dan
FFmpeg terlebih dahulu, lalu jalankan:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --locked
```

## Build otomatis di GitHub

Workflow **Release installers** membangun RPM di Fedora dan installer di runner
Windows. Job Windows menggunakan script PowerShell yang sama, menjalankan tes
backend, memasang installer secara senyap, dan memeriksa bahwa aplikasi membuka
jendela. Job publikasi menunggu kedua platform berhasil dan memverifikasi ulang
checksum sebelum menerbitkan satu Release dengan empat aset: RPM, installer
Windows, dan checksum masing-masing.

Samakan versi pada `package.json`, `src-tauri/Cargo.toml`, dan
`src-tauri/tauri.conf.json`, perbarui kedua lockfile, lalu commit dan push
perubahan. Buat tag stabil yang cocok, misalnya untuk versi saat ini:

```bash
git tag -a v0.1.1 -m "Release v0.1.1"
git push origin v0.1.1
```

Push branch biasa tidak menerbitkan Release. **Actions → Release installers →
Run workflow** pada branch menjalankan kedua build tanpa publikasi; hasil dapat
diunduh dari bagian **Artifacts**. Pengujian manual tidak memublikasikan Release;
publikasi hanya berjalan saat push tag. Jika salah satu build gagal, Release
tidak diterbitkan. Draft dapat dilanjutkan melalui **Re-run failed jobs**; Release
yang sudah publik tidak ditimpa. Tidak perlu personal access token tambahan.
