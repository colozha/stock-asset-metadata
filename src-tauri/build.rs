fn main() {
    println!("cargo:rerun-if-env-changed=PIXABAY_TEST_API_URL");
    #[cfg(feature = "desktop")]
    tauri_build::build();
}
