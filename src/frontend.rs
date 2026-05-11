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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::WebViewSourceKind;

    #[test]
    fn frontend_source_prefers_dev_server_url() {
        let mut env = std::collections::HashMap::new();
        env.insert("ZERO_NATIVE_FRONTEND_URL".into(), "http://127.0.0.1:5173/".into());
        let config = Config::default();
        let source = source_from_env(&env, &config);
        assert_eq!(WebViewSourceKind::Url, source.kind);
        assert_eq!("http://127.0.0.1:5173/", source.bytes);
    }

    #[test]
    fn frontend_source_falls_back_to_production_assets() {
        let env = std::collections::HashMap::new();
        let config = Config { dist: "frontend/dist".into(), entry: "app.html".into(), ..Default::default() };
        let source = source_from_env(&env, &config);
        assert_eq!(WebViewSourceKind::Assets, source.kind);
        let opts = source.asset_options.as_ref().unwrap();
        assert_eq!("frontend/dist", opts.root_path);
        assert_eq!("app.html", opts.entry);
    }
}
