use std::path::Path;

use include_dir::{include_dir, Dir};
use warp::{
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE},
        Response,
    },
    Filter,
};

static STATIC_DIR: Dir = include_dir!("static");
const STATIC_CACHE_CONTROL: &str = "max-age=3600, must-revalidate";

async fn send_file(path: warp::path::Tail) -> Result<impl warp::Reply, warp::Rejection> {
    let path = Path::new(path.as_str());
    let file = STATIC_DIR
        .get_file(path)
        .ok_or_else(warp::reject::not_found)?;

    let content_type = match file.path().extension() {
        Some(ext) if ext == "css" => "text/css",
        Some(ext) if ext == "svg" => "image/svg+xml",
        Some(ext) if ext == "js" => "text/javascript",
        _ => "application/octet-stream",
    };

    let resp = Response::builder()
        .header(CONTENT_TYPE, content_type)
        .header(CACHE_CONTROL, STATIC_CACHE_CONTROL)
        .body(file.contents())
        .unwrap();

    Ok(resp)
}

pub fn routes() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path::tail().and_then(send_file)
}
