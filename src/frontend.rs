use crate::platform::WebViewSource;

#[derive(Debug, Clone)]
pub struct Config {
    pub dist: String,
    pub entry: String,
    pub origin: String,
    pub spa_fallback: bool,
    pub dev_url_env: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dist: "dist".into(),
            entry: "index.html".into(),
            origin: "zero://app".into(),
            spa_fallback: true,
            dev_url_env: "ZERO_NATIVE_FRONTEND_URL".into(),
        }
    }
}

pub fn source_from_env(env: &std::collections::HashMap<String, String>, config: &Config) -> WebViewSource {
    if let Some(url) = env.get(&config.dev_url_env) {
        if !url.is_empty() {
            return WebViewSource::url(url);
        }
    }
    production_source(config)
}

pub fn production_source(config: &Config) -> WebViewSource {
    use crate::platform::WebViewAssetSource;
    WebViewSource::assets(WebViewAssetSource {
        root_path: config.dist.clone(),
        entry: config.entry.clone(),
        origin: config.origin.clone(),
        spa_fallback: config.spa_fallback,
    })
}
