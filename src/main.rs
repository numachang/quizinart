use clap::Parser;
use quizinart::db::Db;
use warp::Filter;

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
        .unwrap_or_else(|_| "tracing=info,warp=debug,quizinart=debug".to_owned());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    let args = Args::parse();

    let db = Db::new(args.url, args.auth_token).await?;
    let routes = quizinart::routes(db);
    let static_files = warp::path("static").and(quizinart::statics::routes());
    let routes = static_files
        .or(routes)
        .recover(quizinart::rejections::handle_rejection);

    let address = args.address.parse::<std::net::SocketAddr>()?;
    warp::serve(routes).run(address).await;

    Ok(())
}
