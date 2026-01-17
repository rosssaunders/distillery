#[derive(Clone)]
pub struct AppConfig {
    pub api_key: String,
    pub model: String,
    pub use_cache: bool,
    pub cache_file: String,
}
