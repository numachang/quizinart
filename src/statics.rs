use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

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
use sha2::{Digest, Sha256};

static STATIC_DIR: Dir = include_dir!("static");
const CACHE_IMMUTABLE: &str = "max-age=31536000, immutable";
const CACHE_SHORT: &str = "max-age=3600, must-revalidate";

/// Maps original path (e.g. "index.css") to hashed filename (e.g. "index.abc12345.css")
static ASSET_MAP: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    build_asset_map(&STATIC_DIR, &mut map);
    map
});

/// Reverse map: hashed path → original path
static REVERSE_MAP: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    ASSET_MAP
        .iter()
        .map(|(original, hashed)| (hashed.clone(), original.clone()))
        .collect()
});

fn build_asset_map(dir: &Dir, map: &mut HashMap<String, String>) {
    for file in dir.files() {
        let path = file.path().to_string_lossy().to_string();
        let hash = {
            let mut hasher = Sha256::new();
            hasher.update(file.contents());
            let result = hasher.finalize();
            format!("{result:x}")[..8].to_string()
        };

        let file_path = Path::new(&path);
        let stem = file_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        let ext = file_path.extension().map(|e| e.to_string_lossy());
        let parent = file_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.to_string_lossy());

        let hashed_name = match ext {
            Some(ext) => format!("{stem}.{hash}.{ext}"),
            None => format!("{stem}.{hash}"),
        };

        let hashed_path = match parent {
            Some(parent) => format!("{parent}/{hashed_name}"),
            None => hashed_name,
        };

        map.insert(path, hashed_path);
    }

    for subdir in dir.dirs() {
        build_asset_map(subdir, map);
    }
}

/// Returns the hashed URL for a static asset, for use in templates.
/// e.g., `asset_path("index.css")` → `"/static/index.abc12345.css"`
pub fn asset_path(name: &str) -> String {
    match ASSET_MAP.get(name) {
        Some(hashed) => format!("/static/{hashed}"),
        None => format!("/static/{name}"),
    }
}

async fn send_file(AxumPath(path): AxumPath<String>) -> Result<impl IntoResponse, StatusCode> {
    let (file_path, cache_control) = if let Some(original) = REVERSE_MAP.get(&path) {
        (original.as_str(), CACHE_IMMUTABLE)
    } else {
        (path.as_str(), CACHE_SHORT)
    };

    let file = STATIC_DIR
        .get_file(Path::new(file_path))
        .ok_or(StatusCode::NOT_FOUND)?;

    let content_type = match file.path().extension() {
        Some(ext) if ext == "css" => "text/css",
        Some(ext) if ext == "svg" => "image/svg+xml",
        Some(ext) if ext == "js" => "text/javascript",
        _ => "application/octet-stream",
    };

    Ok((
        [
            (CONTENT_TYPE, content_type),
            (CACHE_CONTROL, cache_control),
        ],
        file.contents(),
    ))
}

pub fn routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new().route("/{*path}", get(send_file))
}
