//! Pixabay reference search. Local assets and metadata never enter provider requests.
use crate::messages::AppError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const TTL_MS: u64 = 24 * 60 * 60 * 1000;
const MAX_BYTES: u64 = 1024 * 1024;
pub const PER_PAGE: u32 = 20;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ImageType {
    All,
    Photo,
    Illustration,
    Vector,
}
impl ImageType {
    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "all" => Ok(Self::All),
            "photo" => Ok(Self::Photo),
            "illustration" => Ok(Self::Illustration),
            "vector" => Ok(Self::Vector),
            _ => Err("Filter gambar Pixabay tidak valid.".into()),
        }
    }
    fn value(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Photo => "photo",
            Self::Illustration => "illustration",
            Self::Vector => "vector",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageHit {
    pub id: u64,
    pub page_url: String,
    pub preview_url: String,
    pub tags: Vec<String>,
    pub contributor: String,
    pub views: u64,
    pub downloads: u64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    pub query: String,
    pub page: u32,
    pub per_page: u32,
    pub total_hits: u32,
    pub hits: Vec<ImageHit>,
    pub cached_at: u64,
}
#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(rename = "totalHits")]
    total_hits: u32,
    hits: Vec<ApiHit>,
}
#[derive(Debug, Deserialize)]
struct ApiHit {
    id: u64,
    #[serde(rename = "pageURL")]
    page_url: String,
    #[serde(rename = "previewURL")]
    preview_url: String,
    tags: String,
    #[serde(default, rename = "user")]
    contributor: String,
    views: u64,
    downloads: u64,
}

fn error(key: &str, detail: impl ToString) -> AppError {
    AppError {
        key: Some(key.into()),
        params: vec![detail.to_string()],
        detail: None,
    }
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
pub fn normalize_query(query: &str) -> Result<String, AppError> {
    let query = query.split_whitespace().collect::<Vec<_>>().join(" ");
    if query.is_empty() || query.chars().count() > 100 || query.chars().any(char::is_control) {
        return Err("Query Pixabay harus berisi 1–100 karakter.".into());
    }
    Ok(query)
}
fn allowed_url(url: &str, hosts: &[&str]) -> bool {
    !url.chars()
        .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
        && hosts
            .iter()
            .any(|host| url.starts_with(&format!("https://{host}/")))
}
pub fn validate_page_url(url: &str) -> Result<(), AppError> {
    if !allowed_url(url, &["pixabay.com", "www.pixabay.com"]) {
        return Err("URL sumber Pixabay tidak valid.".into());
    }
    Ok(())
}
fn validate_page(value: &SearchPage) -> Result<(), AppError> {
    if value.page == 0
        || value.page > 25
        || value.per_page != PER_PAGE
        || value.total_hits > 500
        || value.hits.len() > PER_PAGE as usize
        || value.page > value.total_hits.div_ceil(PER_PAGE).max(1)
    {
        return Err("Respons Pixabay tidak valid.".into());
    }
    let mut ids = HashSet::new();
    for hit in &value.hits {
        validate_page_url(&hit.page_url)?;
        if !allowed_url(&hit.preview_url, &["cdn.pixabay.com"]) {
            return Err("URL preview Pixabay tidak valid.".into());
        }
        if !ids.insert(hit.id)
            || hit
                .tags
                .iter()
                .any(|tag| tag.is_empty() || tag.contains(','))
        {
            return Err("Respons Pixabay tidak valid.".into());
        }
    }
    Ok(())
}
fn parse_response(bytes: &[u8], query: &str, page: u32) -> Result<SearchPage, AppError> {
    let raw: ApiResponse = serde_json::from_slice(bytes)
        .map_err(|_| AppError::from("Respons Pixabay tidak valid."))?;
    let value = SearchPage {
        query: query.into(),
        page,
        per_page: PER_PAGE,
        total_hits: raw.total_hits.min(500),
        cached_at: now(),
        hits: raw
            .hits
            .into_iter()
            .map(|hit| {
                let mut seen = HashSet::new();
                let tags = hit
                    .tags
                    .split(',')
                    .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
                    .filter(|tag| !tag.is_empty() && seen.insert(tag.to_lowercase()))
                    .collect();
                ImageHit {
                    id: hit.id,
                    page_url: hit.page_url,
                    preview_url: hit.preview_url,
                    tags,
                    contributor: hit.contributor,
                    views: hit.views,
                    downloads: hit.downloads,
                }
            })
            .collect(),
    };
    validate_page(&value)?;
    Ok(value)
}
fn cache_path(dir: &Path, query: &str, kind: ImageType, page: u32) -> PathBuf {
    let input = serde_json::json!([query, kind.value(), page, "en", "popular", PER_PAGE, true]);
    dir.join(format!("{:x}.json", Sha256::digest(input.to_string())))
}
fn read_cache(path: &Path, query: &str, page: u32, time: u64) -> Option<SearchPage> {
    let value = (|| {
        if fs::metadata(path).ok()?.len() > MAX_BYTES {
            return None;
        }
        let value: SearchPage = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
        if value.query != query
            || value.page != page
            || value.cached_at > time
            || time - value.cached_at >= TTL_MS
            || validate_page(&value).is_err()
        {
            return None;
        }
        Some(value)
    })();
    if value.is_none() {
        let _ = fs::remove_file(path);
    }
    value
}
fn prepare_cache(dir: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dir).map_err(|e| error("Cache Pixabay tidak dapat digunakan: {0}", e))?;
    // Refuse a network search if its required cache cannot be written.
    tempfile::NamedTempFile::new_in(dir)
        .map_err(|e| error("Cache Pixabay tidak dapat digunakan: {0}", e))?;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Ok(age) = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .and_then(|t| t.elapsed().map_err(std::io::Error::other))
                {
                    if age.as_millis() >= TTL_MS as u128 {
                        let _ = fs::remove_file(path);
                    }
                }
            }
        }
    }
    Ok(())
}
fn write_cache(path: &Path, value: &SearchPage) -> Result<(), AppError> {
    let write = || -> Result<(), Box<dyn std::error::Error>> {
        let mut file = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
        serde_json::to_writer(&mut file, value)?;
        file.flush()?;
        file.persist(path)?;
        Ok(())
    };
    write().map_err(|e| error("Cache Pixabay tidak dapat digunakan: {0}", e))
}

#[cfg(feature = "desktop")]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub has_api_key: bool,
    pub keyring_error: Option<AppError>,
}

#[cfg(feature = "desktop")]
fn credential() -> Result<keyring::Entry, AppError> {
    keyring::Entry::new("com.stockassetmetadata.desktop", "Pixabay")
        .map_err(|_| AppError::from("Credential store OS tidak dapat dibuka."))
}

#[cfg(feature = "desktop")]
fn api_key() -> Result<Option<String>, AppError> {
    match credential()?.get_password() {
        Ok(key) => Ok(Some(key).filter(|key| !key.trim().is_empty())),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("Credential store OS tidak tersedia.".into()),
    }
}

#[cfg(feature = "desktop")]
pub fn load_settings() -> SettingsView {
    match api_key() {
        Ok(key) => SettingsView {
            has_api_key: key.is_some(),
            keyring_error: None,
        },
        Err(error) => SettingsView {
            has_api_key: false,
            keyring_error: Some(error),
        },
    }
}

#[cfg(feature = "desktop")]
pub fn save_settings(
    api_key: Option<String>,
    clear_api_key: bool,
) -> Result<SettingsView, AppError> {
    let supplied = api_key
        .as_deref()
        .map(str::trim)
        .filter(|key| !key.is_empty());
    if clear_api_key && supplied.is_some() {
        return Err("Pilih penggantian atau penghapusan key Pixabay.".into());
    }
    if clear_api_key {
        match credential()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(_) => return Err("API key tidak dapat dihapus dari credential store OS.".into()),
        }
    } else if let Some(key) = supplied {
        credential()?
            .set_password(key)
            .map_err(|_| AppError::from("API key tidak dapat disimpan di credential store OS."))?;
    }
    Ok(SettingsView {
        has_api_key: self::api_key()?.is_some(),
        keyring_error: None,
    })
}

#[cfg(feature = "desktop")]
fn endpoint() -> Result<reqwest::Url, AppError> {
    #[cfg(debug_assertions)]
    if let Some(endpoint) = option_env!("PIXABAY_TEST_API_URL") {
        let url = reqwest::Url::parse(endpoint).map_err(|_| {
            AppError::from("Endpoint pengujian Pixabay harus memakai loopback HTTP.")
        })?;
        if url.scheme() != "http"
            || !matches!(url.host_str(), Some("127.0.0.1" | "[::1]"))
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err("Endpoint pengujian Pixabay harus memakai loopback HTTP.".into());
        }
        return Ok(url);
    }
    Ok(reqwest::Url::parse("https://pixabay.com/api/").unwrap())
}

#[cfg(feature = "desktop")]
pub async fn search(
    dir: &Path,
    query: &str,
    page: u32,
    kind: ImageType,
) -> Result<SearchPage, AppError> {
    let key = api_key()?.ok_or_else(|| {
        AppError::from("Atur API key Pixabay melalui Pengaturan untuk mencari referensi.")
    })?;
    search_with(
        dir,
        query,
        page,
        kind,
        &key,
        endpoint()?,
        std::time::Duration::from_secs(15),
    )
    .await
}

#[cfg(feature = "desktop")]
async fn search_with(
    dir: &Path,
    query: &str,
    page: u32,
    kind: ImageType,
    key: &str,
    endpoint: reqwest::Url,
    timeout: std::time::Duration,
) -> Result<SearchPage, AppError> {
    let query = normalize_query(query)?;
    if !(1..=25).contains(&page) {
        return Err("Halaman Pixabay tidak valid. Jalankan pencarian ulang.".into());
    }
    prepare_cache(dir)?;
    if page > 1 {
        let first =
            read_cache(&cache_path(dir, &query, kind, 1), &query, 1, now()).ok_or_else(|| {
                AppError::from("Halaman Pixabay tidak valid. Jalankan pencarian ulang.")
            })?;
        if page > first.total_hits.div_ceil(PER_PAGE).max(1) {
            return Err("Halaman Pixabay tidak valid. Jalankan pencarian ulang.".into());
        }
    }
    let path = cache_path(dir, &query, kind, page);
    if let Some(value) = read_cache(&path, &query, page, now()) {
        return Ok(value);
    }
    let client = reqwest::Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| error("Pencarian Pixabay gagal: {0}", e.without_url()))?;
    let network_error = |e: reqwest::Error| {
        error(
            "Pencarian Pixabay gagal: {0}",
            e.without_url().to_string().replace(key, "[redacted]"),
        )
    };
    let mut response = client
        .get(endpoint)
        .query(&[
            ("key", key),
            ("q", query.as_str()),
            ("image_type", kind.value()),
            ("page", &page.to_string()),
            ("per_page", "20"),
            ("lang", "en"),
            ("order", "popular"),
            ("safesearch", "true"),
        ])
        .send()
        .await
        .map_err(network_error)?;
    if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let seconds = response
            .headers()
            .get("X-RateLimit-Reset")
            .and_then(|s| s.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60);
        return Err(error(
            "Batas request Pixabay tercapai. Coba lagi setelah {0} detik.",
            seconds,
        ));
    }
    let status = response.status();
    if response
        .content_length()
        .is_some_and(|size| size > MAX_BYTES)
    {
        return Err("Respons Pixabay melebihi batas ukuran.".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(network_error)? {
        if bytes.len() as u64 + chunk.len() as u64 > MAX_BYTES {
            return Err("Respons Pixabay melebihi batas ukuran.".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    if !status.is_success() {
        return Err(error(
            "Pencarian Pixabay gagal: {0}",
            format!(
                "HTTP {}: {}",
                status.as_u16(),
                String::from_utf8_lossy(&bytes).replace(key, "[redacted]")
            ),
        ));
    }
    let value = parse_response(&bytes, &query, page)?;
    write_cache(&path, &value)?;
    Ok(value)
}

#[cfg(feature = "desktop")]
pub fn open_page(url: &str) -> Result<(), AppError> {
    validate_page_url(url)?;
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer.exe")
            .arg(url)
            .spawn()
            .map_err(|e| error("Halaman Pixabay tidak dapat dibuka: {0}", e))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let result = std::process::Command::new("xdg-open")
            .arg(url)
            .output()
            .map_err(|e| error("Halaman Pixabay tidak dapat dibuka: {0}", e))?;
        if !result.status.success() {
            return Err(error(
                "Halaman Pixabay tidak dapat dibuka: {0}",
                String::from_utf8_lossy(&result.stderr),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({"totalHits": 500, "hits": [{"id": 1, "pageURL": "https://pixabay.com/photos/apple-1/", "previewURL": "https://cdn.pixabay.com/photo/apple.jpg", "tags": " Apple, apple, , green   fruit, ÉTÉ, été", "user": "Artist", "views": 10, "downloads": 5}]})).unwrap()
    }
    #[test]
    fn parses_tags_and_validates_trust_boundary() {
        let value = parse_response(&fixture(), "apple", 1).unwrap();
        assert_eq!(value.hits[0].tags, ["Apple", "green fruit", "ÉTÉ"]);
        assert_eq!(normalize_query("  red\n apple  ").unwrap(), "red apple");
        assert!(normalize_query("").is_err());
        assert!(normalize_query(&"é".repeat(101)).is_err());
        assert!(normalize_query(&"é".repeat(100)).is_ok());
        assert!(ImageType::parse("video").is_err());
        for url in [
            "http://pixabay.com/a",
            "https://pixabay.com.evil/a",
            "https://pixabay.com@evil/a",
            "https://pixabay.com:444/a",
            "https://pixabay.com/\\evil",
            "https://evil/a",
        ] {
            assert!(validate_page_url(url).is_err(), "{url}");
        }
        let bad = String::from_utf8(fixture())
            .unwrap()
            .replace("cdn.pixabay.com", "evil.com");
        assert!(parse_response(bad.as_bytes(), "apple", 1).is_err());
        assert!(parse_response(b"bad json", "apple", 1).is_err());
    }
    #[test]
    fn cache_is_separate_atomic_and_expires() {
        let dir = tempfile::tempdir().unwrap();
        prepare_cache(dir.path()).unwrap();
        let path = cache_path(dir.path(), "apple", ImageType::All, 1);
        assert_ne!(path, cache_path(dir.path(), "apple", ImageType::Photo, 1));
        assert_ne!(path, cache_path(dir.path(), "pear", ImageType::All, 1));
        assert_ne!(path, cache_path(dir.path(), "apple", ImageType::All, 2));
        let value = parse_response(&fixture(), "apple", 1).unwrap();
        write_cache(&path, &value).unwrap();
        assert!(read_cache(&path, "apple", 1, value.cached_at + TTL_MS - 1).is_some());
        assert!(read_cache(&path, "apple", 1, value.cached_at + TTL_MS).is_none());
        assert!(!path.exists());
        fs::write(&path, b"broken").unwrap();
        assert!(read_cache(&path, "apple", 1, now()).is_none());
        assert!(!path.exists());
        let file = dir.path().join("not-a-directory");
        fs::write(&file, b"").unwrap();
        assert!(prepare_cache(&file).is_err());
    }
    #[cfg(feature = "desktop")]
    #[test]
    fn key_replacement_and_deletion_cannot_be_combined() {
        let error = save_settings(Some("dummy-private-key".into()), true)
            .err()
            .unwrap();
        assert_eq!(
            error.key.as_deref(),
            Some("Pilih penggantian atau penghapusan key Pixabay.")
        );
        assert!(!serde_json::to_string(&error)
            .unwrap()
            .contains("dummy-private-key"));
    }

    #[cfg(feature = "desktop")]
    fn serve(
        status: &str,
        body: Vec<u8>,
        delay: u64,
    ) -> (reqwest::Url, std::sync::mpsc::Receiver<String>) {
        use std::{io::Read, net::TcpListener};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = reqwest::Url::parse(&format!("http://{}/api/", listener.local_addr().unwrap()))
            .unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let status = status.to_owned();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut input = [0; 8192];
            let n = stream.read(&mut input).unwrap();
            tx.send(String::from_utf8_lossy(&input[..n]).into_owned())
                .ok();
            std::thread::sleep(std::time::Duration::from_millis(delay));
            let headers = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nX-RateLimit-Reset: 17\r\nConnection: close\r\n\r\n", body.len());
            let _ = stream.write_all(headers.as_bytes());
            let _ = stream.write_all(&body);
        });
        (url, rx)
    }
    #[cfg(feature = "desktop")]
    #[test]
    fn http_parameters_cache_pagination_and_failures() {
        use std::time::Duration;
        tauri::async_runtime::block_on(async {
            let dir = tempfile::tempdir().unwrap();
            let (url, rx) = serve("200 OK", fixture(), 0);
            let value = search_with(
                dir.path(),
                "red & été",
                1,
                ImageType::Vector,
                "secret-test",
                url.clone(),
                Duration::from_secs(1),
            )
            .await
            .unwrap();
            let request = rx.recv().unwrap();
            for expected in [
                "q=red+%26+%C3%A9t%C3%A9",
                "image_type=vector",
                "lang=en",
                "order=popular",
                "safesearch=true",
                "per_page=20",
                "page=1",
            ] {
                assert!(request.contains(expected), "{request}");
            }
            assert_eq!(value.hits.len(), 1);
            assert!(search_with(
                dir.path(),
                "red & été",
                1,
                ImageType::Vector,
                "other-key",
                url.clone(),
                Duration::from_secs(1)
            )
            .await
            .is_ok());
            let stored =
                fs::read_to_string(cache_path(dir.path(), "red & été", ImageType::Vector, 1))
                    .unwrap();
            assert!(!stored.contains("secret-test"));
            assert!(search_with(
                dir.path(),
                "apple",
                2,
                ImageType::All,
                "key",
                url.clone(),
                Duration::from_secs(1)
            )
            .await
            .is_err());
            for (status, body, delay, expected) in [
                ("429 Too Many Requests", vec![], 0, "Batas request"),
                (
                    "500 Internal Server Error",
                    b"secret-test provider failure".to_vec(),
                    0,
                    "Pencarian Pixabay gagal",
                ),
                (
                    "200 OK",
                    b"broken".to_vec(),
                    0,
                    "Respons Pixabay tidak valid",
                ),
                (
                    "200 OK",
                    vec![b'x'; MAX_BYTES as usize + 1],
                    0,
                    "Respons Pixabay melebihi",
                ),
                ("200 OK", fixture(), 150, "Pencarian Pixabay gagal"),
                ("302 Found", vec![], 0, "Pencarian Pixabay gagal"),
            ] {
                let (url, _rx) = serve(status, body, delay);
                let fresh = tempfile::tempdir().unwrap();
                let err = search_with(
                    fresh.path(),
                    "apple",
                    1,
                    ImageType::All,
                    "secret-test",
                    url,
                    Duration::from_millis(80),
                )
                .await
                .unwrap_err();
                assert!(err.key.as_deref().unwrap().starts_with(expected), "{err:?}");
                assert!(!serde_json::to_string(&err).unwrap().contains("secret-test"));
            }
        });
    }
}
