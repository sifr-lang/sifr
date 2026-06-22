pub struct BackendErrorBridge {
    pub message: String,
}

pub fn route_probe(path: &str) -> Result<String, BackendErrorBridge> {
    Ok(format!("{}:{}:{}", axum::route(path), tower_http::trace(), sqlx::offline_mode()))
}

pub fn map_panic(message: &str) -> BackendErrorBridge {
    BackendErrorBridge {
        message: message.to_owned(),
    }
}
