use clap::Parser;
use db::Db;
use futures::{future::OptionFuture, FutureExt};
use warp::Filter;

// Module declarations
mod db;
mod handlers;
mod models;
mod names;
mod rejections;
mod statics;
mod utils;
mod views;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// libSQL server address
    #[clap(env)]
    url: String,

    /// libSQL authentication token.
    #[clap(env)]
    auth_token: String,

    /// The address to bind to.
    #[arg(short, long, env, default_value = "127.0.0.1:1414")]
    address: String,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "tracing=info,warp=debug,quizzy=debug".to_owned());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    let args = Args::parse();

    let db = Db::new(args.url, args.auth_token).await?;
    let routes = routes(db);
    let static_files = warp::path("static").and(statics::routes());
    let routes = static_files
        .or(routes)
        .recover(rejections::handle_rejection);

    let address = args.address.parse::<std::net::SocketAddr>()?;
    warp::serve(routes).run(address).await;

    Ok(())
}

pub fn routes(
    conn: db::Db,
) -> impl warp::Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    handlers::homepage::route(conn.clone()).or(handlers::quiz::route(conn.clone()))
}

fn with_state<T: Clone + Send>(
    db: T,
) -> impl Filter<Extract = (T,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

trait FutureOptionExt<T> {
    fn to_future(self) -> OptionFuture<T>;
}

impl<T> FutureOptionExt<T> for Option<T> {
    fn to_future(self) -> OptionFuture<T> {
        OptionFuture::from(self)
    }
}

pub fn is_authorized(
    db: Db,
) -> impl Filter<Extract = ((),), Error = warp::reject::Rejection> + Clone {
    warp::any()
        .and(with_state(db.clone()))
        .and(warp::cookie::optional::<String>(names::ADMIN_SESSION_COOKIE_NAME))
        .and_then(authorized)
}

async fn authorized(db: Db, session: Option<String>) -> Result<(), warp::Rejection> {
    let session_exists = session
        .map(|s| db.admin_session_exists(s).map(|res| res.ok()))
        .to_future()
        .await
        .flatten()
        .unwrap_or_default();

    if session_exists {
        Ok(())
    } else {
        Err(warp::reject::custom(rejections::Unauthorized))
    }
}

pub fn is_htmx() -> impl Filter<Extract = (bool,), Error = warp::reject::Rejection> + Clone {
    warp::any()
        .and(warp::header::optional::<String>("HX-Request"))
        .map(|hx_req: Option<String>| hx_req.is_some_and(|x| x == "true"))
}
