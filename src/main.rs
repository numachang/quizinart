use clap::Parser;
use quizinart::{db::Db, AppState};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// libSQL server address
    #[clap(env)]
    url: String,

    /// libSQL authentication token.
    #[clap(env, default_value = "")]
    auth_token: String,

    /// The address to bind to.
    #[arg(short, long, env, default_value = "127.0.0.1:1414")]
    address: String,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv().ok();

    let filter =
        std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,quizinart=debug".to_owned());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    let args = Args::parse();

    let db = Db::new(args.url, args.auth_token).await?;
    let state = AppState { db };
    let app = quizinart::router(state);

    let address = args.address.parse::<std::net::SocketAddr>()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!("listening on {}", address);
    axum::serve(listener, app).await?;

    Ok(())
}
