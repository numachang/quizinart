use std::path::Path;

use axum::{
    extract::Path as AxumPath,
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        StatusCode,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use include_dir::{include_dir, Dir};

static STATIC_DIR: Dir = include_dir!("static");
const STATIC_CACHE_CONTROL: &str = "max-age=3600, must-revalidate";

async fn send_file(AxumPath(path): AxumPath<String>) -> Result<impl IntoResponse, StatusCode> {
    let path = Path::new(&path);
    let file = STATIC_DIR.get_file(path).ok_or(StatusCode::NOT_FOUND)?;

    let content_type = match file.path().extension() {
        Some(ext) if ext == "css" => "text/css",
        Some(ext) if ext == "svg" => "image/svg+xml",
        Some(ext) if ext == "js" => "text/javascript",
        _ => "application/octet-stream",
    };

    Ok((
        [
            (CONTENT_TYPE, content_type),
            (CACHE_CONTROL, STATIC_CACHE_CONTROL),
        ],
        file.contents(),
    ))
}

pub fn routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new().route("/{*path}", get(send_file))
}
