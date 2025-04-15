#[derive(Debug, serde::Deserialize)]
pub struct Config {
    /// データベースの接続 URL
    pub database_url: String,
    /// HTTP サーバーのアドレス
    #[serde(default = "default_http_server_addr")]
    pub http_server_addr: String,
}

fn default_http_server_addr() -> String {
    "localhost:8000".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        envy::from_env::<Config>().expect("Failed to configure")
    }
}
