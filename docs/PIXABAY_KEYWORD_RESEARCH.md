# Perencanaan: Pencarian Referensi dan Keyword Pixabay

Tanggal pemeriksaan sumber: **23 Oktober 2026**. Dokumen ini menyimpan rancangan enhancement, diperbarui agar key pribadi diisi melalui Settings. Implementasi telah ditambahkan; bukti dan batas verifikasi aktual dicatat di [VALIDATION.md](VALIDATION.md).

## Ringkasan

Tambahkan pencarian gambar Pixabay sebagai alat bantu riset keyword untuk aset lokal yang sedang diedit. Pengguna mencari gambar, memilih satu atau beberapa hasil yang relevan secara visual, meninjau tag yang paling konsisten di antara gambar terpilih, lalu menambahkan tag pilihannya ke draft keyword aset aktif. Pengguna tetap mengedit dan menyimpan metadata melalui alur yang sudah ada.

Enhancement mengikuti karakter project sebagai aplikasi desktop Tauri lokal: UI tetap memakai TypeScript vanilla dan CSS; permintaan API serta cache ditangani Rust; tidak diperlukan server aplikasi, database, atau upload aset lokal. Pencarian merupakan aksi eksplisit dan mengirim teks query ke Pixabay. Path, preview, dan isi file lokal tidak dikirim.

## Keputusan Produk dan Batasan

- Sumber v1 hanya **gambar Pixabay**: foto, ilustrasi, dan vektor. Video Pixabay, provider lain, unduh aset penuh, dan pencarian berbasis upload tidak termasuk.
- Hasil Pixabay dipakai sebagai referensi visual dan sumber tag. Hasil tersebut bukan rekomendasi yang disetujui agency dan tidak menjamin hak untuk mengirim aset Pixabay sebagai karya pengguna ke portal microstock.
- Urutan hasil pencarian memakai `order=popular`. Peringkat keyword dihitung dari jumlah gambar terpilih yang memiliki tag tersebut, bukan dari angka views/downloads Pixabay dan bukan dari volume pencarian global.
- Keyword yang dipilih ditambahkan ke draft aset lokal yang sedang aktif. Urutan keyword yang sudah ada dipertahankan; keyword duplikat dilewati tanpa membatalkan penambahan lain. Perubahan tetap belum tersimpan sampai pengguna menjalankan aksi simpan biasa.
- Gambar terpilih dapat berasal dari beberapa halaman dalam satu pencarian. Mengirim query baru atau mengganti filter gambar mengosongkan pilihan referensi dan saran lama setelah pencarian baru dijalankan. Status UI menyebutkan bahwa pilihan tersebut direset.
- Pencarian dan pilihan referensi hanya hidup selama sesi aplikasi. Respons API di-cache lokal selama 24 jam sesuai ketentuan API; cache terpisah dari cache preview aset lokal.
- API key pribadi diisi pada **Pengaturan → Pixabay**, disimpan melalui satu aksi Simpan pengaturan, dan diletakkan di credential store OS dengan account terpisah dari AI. Tidak ada key bawaan binary, fallback key build, atau kebutuhan rebuild. Input kosong mempertahankan key; penghapusan menunggu Simpan pengaturan; Kembali membatalkan perubahan. Keputusan ini menggantikan rancangan key bersama sebelumnya.

## Desain Antarmuka

Pertahankan relasi tiga kolom yang sudah ditetapkan di `docs/editor.md`: tree aset lokal, panel tengah, dan editor metadata. Tombol **Cari referensi Pixabay** berada di section Keywords untuk aset aktif. Tombol tersebut mengubah isi panel tengah dari preview lokal ke pencarian; tree dan panel metadata tetap terlihat.

```text
┌ Aset lokal ─────────┬ Referensi Pixabay ──────────────────┬ Metadata aset aktif ───┐
│ tree / format       │ [ query                         ] 🔎 │ file: apple.jpg        │
│                     │ [Semua] [Foto] [Ilustrasi] [Vektor] │ Title / Description    │
│                     │ [gambar] [gambar] [gambar]          │ Keywords draft         │
│                     │ [gambar] [gambar] [gambar]          │ Saran: apple  4/5      │
│                     │ pilihan referensi: 5  halaman 1/25  │        green  3/5      │
└─────────────────────┴─────────────────────────────────────┴────────────────────────┘
```

Komponen dan perilaku:

1. Header panel tengah berisi query, tombol cari, filter tipe gambar, status hasil, jumlah pilihan, dan tautan untuk kembali ke preview lokal. Enter menjalankan pencarian. Tidak ada request saat pengguna baru mengetik.
2. Grid memakai `previewURL` dari Pixabay, checkbox yang dapat dioperasikan dengan keyboard, nama kontributor bila tersedia, dan atribusi/link sumber Pixabay. Grid mengisi lebar panel yang tersedia; pada lebar sempit beralih ke satu kolom, tanpa memaksa workspace melewati minimum native 1024×700.
3. Pilihan gambar bertahan saat berpindah halaman hasil. Kartu yang dipilih menampilkan indikator selain warna. Query baru mengatur ulang pilihan dan saran; pilihan tidak digabung diam-diam di antara topik pencarian.
4. Panel Keywords menampilkan daftar keyword draft yang sekarang serta section **Saran dari gambar terpilih**. Tiap saran memiliki checkbox, jumlah `n/total`, label relevansi, dan warna. Tombol **Tambahkan pilihan ke draft** hanya aktif jika setidaknya satu gambar dan satu saran dipilih.
5. Sebelum menerapkan saran, tampilkan nama aset target yang sedang aktif. Bila pengguna mengganti aset melalui tree saat mode riset terbuka, penambahan diarahkan ke draft aset yang kini aktif; referensi gambar tetap tersedia sampai query/filter berubah atau mode ditutup.
6. Aksi **Kembali ke preview** memulihkan preview lokal tanpa mengubah draft, konteks, atau urutan keyword. Jika gambar Pixabay gagal dimuat, kartu tetap menunjukkan kegagalan preview dan tautan sumber bila tersedia.

Gunakan token warna yang sudah ada untuk panel, border, teks, focus, dan selection. Tambahkan pasangan token light/dark untuk tingkat relevansi, bukan warna literal di komponen. Gunakan tiga tingkat: **tinggi** jika tag ada di ≥75% referensi, **sedang** jika ada di ≥40% dan <75%, **rendah** jika ada di >0% dan <40%. Selalu tampilkan hitungan dan label; warna tidak menjadi satu-satunya pembawa makna. Untuk satu gambar, semua tag sumber akan memiliki cakupan 1/1; UI harus menjelaskan bahwa ini ukuran konsistensi antar referensi, bukan penilaian semantik otomatis.

Semua label, tooltip, status pencarian, aksesibilitas, dan error baru memiliki copy ID/EN pada `src/messages.ts`, dengan parameter interpolasi yang sama. Error buatan backend menggunakan `{ key, params, detail }` dan didaftarkan di `src-tauri/src/messages.rs`. Tetap gunakan fokus keyboard terlihat, kontras tema light/dark, dan status teks untuk loading, hasil kosong, reset pilihan, serta error.

## API, Data, dan Peringkat Keyword

### Permintaan Pixabay

Gunakan endpoint gambar `GET https://pixabay.com/api/` melalui `reqwest` di backend Rust. `reqwest`, `serde`, `serde_json`, dan `sha2` sudah ada dalam dependency project; untuk v1 tidak perlu dependency HTTP baru. Frontend tidak memanggil Pixabay langsung.

Parameter yang dikirim:

| Parameter | Nilai rancangan |
| --- | --- |
| `key` | Key pribadi dari credential store OS; tidak pernah dicatat ke log |
| `q` | Query wajib, trim whitespace, maksimal 100 karakter, di-encode oleh URL builder |
| `lang` | `en`, agar pencarian dan tag yang disarankan berorientasi keyword microstock bahasa Inggris |
| `image_type` | `all` secara default; pilihan `photo`, `illustration`, atau `vector` |
| `order` | `popular` |
| `page` | Halaman aktif, mulai dari 1 |
| `per_page` | 20 |
| `safesearch` | `true` |

Jangan meminta page di luar jumlah hasil yang dilaporkan API atau batas 500 hasil yang tersedia per query. Batasi aksi ke pencarian nyata oleh pengguna; jangan melakukan pencarian otomatis, prefetch halaman yang belum dibuka, atau pengambilan massal. API docs menyebut batas default 100 request per key dalam 60 detik, cache 24 jam, dan larangan systematic mass downloads. Respons API menyertakan header status rate limit; HTTP 429 ditampilkan sebagai error yang bisa dipahami tanpa retry loop.

### Kontrak IPC yang Direncanakan

Daftarkan `load_pixabay_settings() → { hasApiKey, keyringError }` dan `save_pixabay_settings(apiKey, clearApiKey)` untuk key pribadi, tanpa mengirim nilai tersimpan ke UI. Daftarkan command Tauri `search_pixabay_images` dan tipe hasil terpisah dari metadata aset lokal. Command menerima `query`, `page`, dan `imageType`; `order`, `lang`, `perPage`, dan SafeSearch ditetapkan di backend agar kontrak UI tidak dapat melonggarkan batas tersebut.

```ts
interface PixabayImageHit {
  id: number;
  pageUrl: string;
  previewUrl: string;
  tags: string[];
  contributor: string;
  views: number;
  downloads: number;
}

interface PixabaySearchPage {
  query: string;
  page: number;
  perPage: number;
  totalHits: number;
  hits: PixabayImageHit[];
  cachedAt: number;
}
```

Backend memetakan string tag Pixabay yang dipisahkan koma ke `tags[]`, mengabaikan elemen kosong, dan mempertahankan teks tampilan yang pertama ditemukan. Validasi query, filter enum, URL HTTPS, ukuran respons, timeout request 15 detik, serta struktur JSON dilakukan sebelum hasil dikirim ke UI. URL thumbnail dibatasi ke host `cdn.pixabay.com`; URL sumber harus memakai HTTPS dan host Pixabay yang diharapkan.

### Cache dan URL Gambar

- Simpan respons hasil yang sudah dinormalisasi di subfolder khusus, misalnya `app_cache_dir/pixabay-search/`, bukan bersama cache preview aset. Cache key adalah SHA-256 dari query yang dinormalisasi, `image_type`, nomor halaman, `lang`, urutan, dan ukuran halaman; jangan menggunakan query mentah sebagai nama file.
- Cache hasil untuk 24 jam dan pakai ulang respons yang belum kedaluwarsa. Hapus entry kedaluwarsa secara opportunistic. Jangan cache file gambar atau URL penuh resolusi tinggi.
- UI hanya memakai `previewURL` untuk sementara saat menampilkan hasil pencarian. Pixabay melarang permanent hotlinking; tidak ada download gambar asli, penyimpanan ke folder aset, ekspor gambar, atau penanaman URL thumbnail ke metadata.
- Karena WebView saat ini membatasi `img-src`, implementasi perlu mengizinkan hanya `https://cdn.pixabay.com` untuk thumbnail. Request API tetap dilakukan Rust, sehingga tidak perlu membuka `connect-src` WebView ke domain Pixabay.

### Agregasi dan Urutan

1. Satu-satunya sumber saran adalah tag dari hasil yang dicentang pengguna; hasil yang belum dipilih tidak ikut dihitung.
2. Normalisasi untuk pencocokan: trim spasi tepi, gabungkan whitespace berulang, dan bandingkan tanpa membedakan kapitalisasi. Di dalam satu gambar, tag yang sama dihitung sekali. Pertahankan kapitalisasi tampilan dari kemunculan pertama.
3. Setiap tag dihitung paling banyak satu kali per gambar referensi. Skor frekuensi `count` adalah jumlah gambar terpilih yang memiliki tag; denominator adalah jumlah gambar unik terpilih.
4. Urutkan `count` menurun. Untuk seri, pakai urutan kemunculan pertama di hasil API yang berurutan popular agar stabil dan tidak merusak prioritas provider.
5. Warna dihitung dari `count / selectedImageCount` sesuai ambang UI. Nilai views/downloads adalah sinyal popularitas **aset Pixabay**; jangan gunakan sebagai bobot ranking keyword karena desain ini mengukur konsistensi tag di referensi yang dipilih.
6. Bila pengguna memilih saran lalu menambahkannya, tambahkan sesuai urutan ranking ke akhir `draft.keywords`. Deduplikasi terhadap draft menggunakan pencocokan tanpa kapitalisasi; urutan lama tetap sama. Status menyebut jumlah yang ditambahkan dan jumlah duplikat yang dilewati.

## Key, Privasi, dan Ketentuan Pixabay

Project tidak memiliki server aplikasi. Key pribadi dimasukkan di Settings dan dikirim frontend ke backend hanya untuk penyimpanan eksplisit. Backend menyimpannya di credential store OS (`com.stockassetmetadata.desktop`, account `Pixabay`), membaca key saat pencarian dijalankan, dan hanya mengirim status/error kepada UI. Tidak ada key dalam source, binary, log, JSON pengaturan, atau cache hasil. Menyimpan key tidak memanggil API Pixabay; validitasnya diperiksa pada pencarian pengguna.

Build dan pipeline RPM/Windows tidak lagi membaca atau mensyaratkan `PIXABAY_API_KEY`. Build lama yang memiliki key bawaan tidak dimigrasikan; pengguna mengisi key sendiri. Credential store yang tidak tersedia dilaporkan sebagai error terstruktur. Penyimpanan tampilan/penyimpanan, AI, dan Pixabay memiliki hasil parsial yang eksplisit; input bagian yang gagal tetap tersedia untuk percobaan ulang.

Pencarian mengirim query dan request network dari perangkat ke Pixabay. Tidak mengirim file, nama/path lokal, preview lokal, maupun metadata pengguna. Sebutkan hal ini di helper text sebelum pencarian pertama atau di informasi fitur. Ketika hasil ditampilkan, selalu cantumkan sumber Pixabay beserta kontributor/tautan halaman bila tersedia. Tautan eksternal harus dibuka dengan alur OS yang sudah digunakan project dan URL-nya divalidasi; jangan mengizinkan navigasi WebView bebas.

API usage terms dan lisensi konten adalah dua hal yang berbeda. API gratis tetap memiliki key, atribusi hasil, batas request, cache, dan batas penggunaan. Konten Pixabay tunduk pada Content License dan prohibited uses; hasil pencarian tidak memberikan hak eksklusif atas gambar. Pengguna bertanggung jawab memeriksa izin pihak ketiga untuk penggunaan komersial, termasuk orang, merek, properti, atau karya yang ditampilkan. Tool ini mengusulkan tag dan referensi, bukan memvalidasi lisensi atau aturan submission agency.

## Perubahan Teknis Saat Implementasi

Dokumen ini merencanakan, bukan melakukan, perubahan berikut:

1. **Backend provider:** modul Rust terpisah (misalnya `src-tauri/src/pixabay.rs`), tipe request/response, validasi, `reqwest` timeout, cache 24 jam, pemetaan error, dan command Tauri. Daftarkan command di `src-tauri/src/lib.rs`; jangan memakai command inspeksi aset lokal untuk hasil remote.
2. **UI dan state:** tambah tipe IPC Pixabay di `src/types.ts`; state pencarian/pilihan referensi sementara di `src/main.ts`; toggle panel tengah preview/pencarian; grid, chips ranking, serta layout responsif dan token relevansi di `src/styles.css`.
3. **Copy dan kebijakan akses:** terjemahan ID/EN di `src/messages.ts` dan `src-tauri/src/messages.rs`; update `img-src` terbatas pada CDN Pixabay di `src-tauri/tauri.conf.json`; input key pribadi di Settings dan penyimpanan credential OS tanpa key di repository atau binary.
4. **Dokumentasi setelah implementasi:** perbarui README untuk koneksi internet, fungsi pencarian, atribusi, dan key pribadi melalui Settings. Bukti pengujian serta batas yang belum diverifikasi dicatat di `docs/VALIDATION.md`.

Tidak ada perubahan metadata yang disimpan otomatis. Pemilihan lokal untuk batch save tetap terpisah dari referensi Pixabay; perintah tambahkan hanya mengubah draft aset aktif. Aksi Generate Keywords AI yang ada tetap berbeda: bila dijalankan setelah penambahan tag Pixabay, perilaku generator yang mengganti daftar keyword mengikuti semantik saat ini dan perlu dinyatakan di UI/help agar tidak mengejutkan.

## Skenario Verifikasi Implementasi Mendatang

**Backend dan cache**

- Unit test parsing tag kosong, spasi, kapitalisasi, tag duplikat dalam satu gambar, dan Unicode; ranking frekuensi, pemecah seri, serta batas warna.
- Mock HTTP lokal menguji query encode, filter yang diizinkan, mapping `hits`, pagination maksimum, timeout, HTTP 429, error HTTP lain, JSON rusak, dan URL/host gambar yang ditolak.
- Tes cache memastikan key berbeda untuk query/filter/page berbeda; cache fresh dipakai ulang hingga 24 jam; expired dibuang/diganti; file cache tidak memuat API key dan tidak menyimpan bytes gambar.
- Tanpa key tersimpan, pencarian dinonaktifkan dengan petunjuk Settings dan error terstruktur. Simpan/ganti/hapus/batal serta persistensi restart diverifikasi melalui credential store OS terisolasi.

**Renderer dan Tauri**

- Pilih beberapa gambar pada satu dan beberapa halaman; saran hanya berasal dari gambar terpilih; perubahan query/filter mereset selection secara eksplisit.
- Tambahkan tag ke draft aktif: urutan lama tetap, tag baru mengikuti ranking, duplikat beda kapitalisasi dilewati, dan simpan metadata tetap aksi terpisah.
- Keluar dari mode riset mengembalikan preview lokal; draft, additional context, dan keyword order tidak berubah. Ganti aset aktif saat mode riset terbuka; tombol tambah menunjukkan target yang benar.
- Uji hasil kosong, preview eksternal gagal, loading, offline, 429, key tidak tersedia, dan error membuka sumber. Uji tidak mengirim path/isi file lokal dan tidak menampilkan key pada log/error.
- Uji keyboard, pembaca layar/nama kontrol, fokus, teks status, grid pada 1440×900 dan minimum native 1024×700, serta keempat kombinasi ID/EN × light/dark tanpa overflow.
- UI memakai IPC tiruan untuk variasi/error; paling sedikit satu verifikasi Tauri native menggunakan HTTP mock lokal untuk membuktikan command Rust, permission, cache, dan CSP thumbnail. Pencarian live ke Pixabay hanya smoke check manual dengan key pribadi yang disediakan, tidak menjadi tes otomatis.

Perintah pemeriksaan yang tetap relevan sesudah implementasi mengikuti `AGENTS.md`: `npm run check`, `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml --no-default-features`, dan verifikasi IPC Tauri native. Tidak ada perintah tersebut yang dijalankan saat membuat dokumen ini.

## Risiko dan Kriteria Selesai

Key pribadi memerlukan credential store OS yang berfungsi dan tetap mengikuti kuota API Pixabay. Cache/query menyimpan istilah pencarian lokal selama masa cache. Gambar/tag/URL provider dapat berubah, dihapus, tidak tersedia, atau lisensinya memiliki batas dan hak pihak ketiga yang tidak diverifikasi aplikasi.

Enhancement dianggap siap untuk rilis setelah pencarian manual berhasil dengan batas API, key tidak bocor ke UI/log, attribution tampil, tag ranking dapat dijelaskan lewat hitungan referensi, penambahan hanya mengubah draft aktif, UI konsisten di dua tema/bahasa dan ukuran minimum, kegagalan jaringan terlapor dengan benar, serta pemeriksaan native dan build release tercatat. Klaim ranking harus tetap disebut sebagai **frekuensi tag dalam gambar Pixabay yang dipilih**, bukan popularitas keyword seluruh pasar microstock.

## Sumber Resmi dan Konteks Project

- [Pixabay API Documentation](https://pixabay.com/api/docs/) — parameter gambar, key, tag, preview, views/downloads, rate limit, cache, atribusi hasil, dan hotlinking. Diperiksa 23 Oktober 2026.
- [Pixabay Terms of Service](https://pixabay.com/service/terms/) — ketentuan layanan dan lisensi konten; terakhir diperbarui 20 Oktober 2026 saat diperiksa.
- [Pixabay Content License Summary](https://pixabay.com/service/license-summary/) — ringkasan penggunaan gratis dan prohibited uses; dokumen menyatakan lisensi lengkap yang mengikat secara hukum.
- [PRODUCT.md](../PRODUCT.md), [DESIGN.md](../DESIGN.md), [docs/editor.md](editor.md), dan [AGENTS.md](../AGENTS.md) — tujuan produk, desain, alur editor, stack, i18n, tema, dan verifikasi yang sudah berlaku.
