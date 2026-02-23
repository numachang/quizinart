use clap::Parser;
use quizinart::{db::Db, email::ResendEmailSender, services::auth::AuthService, AppState};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// PostgreSQL connection URL.
    #[clap(env)]
    database_url: String,

    /// The address to bind to.
    #[arg(short, long, env, default_value = "127.0.0.1:1414")]
    address: String,

    /// Set cookie Secure flag (enable for HTTPS deployments).
    #[arg(long, env, default_value = "false")]
    secure_cookies: bool,

    /// Resend API key for email verification. Leave empty to skip verification in dev.
    #[arg(long, env, default_value = "")]
    resend_api_key: String,

    /// Base URL for verification links (e.g., https://quizinart.onrender.com).
    #[arg(long, env, default_value = "http://127.0.0.1:1414")]
    base_url: String,

    /// Disable rate limiting on auth endpoints (for E2E testing).
    #[arg(long, env, default_value = "false")]
    disable_rate_limit: bool,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv().ok();

    let filter =
        std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,quizinart=debug".to_owned());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    tokio::runtime::Builder::new_multi_thread()
        .thread_stack_size(8 * 1024 * 1024) // 8 MB for worker threads
        .enable_all()
        .build()?
        .block_on(async {
            // Spawn on a worker thread (8 MB stack) instead of main thread (1 MB)
            tokio::spawn(run()).await?
        })
}

async fn run() -> color_eyre::Result<()> {
    let args = Args::parse();

    let db = Db::new(args.database_url).await?;
    let secure_cookies = args.secure_cookies || args.base_url.starts_with("https://");
    if secure_cookies && !args.secure_cookies {
        tracing::info!("secure_cookies auto-enabled because base_url uses HTTPS");
    }
    let email_sender = ResendEmailSender::new(args.resend_api_key);
    let auth = AuthService::new(db.clone(), email_sender, args.base_url);
    let state = AppState {
        db,
        auth,
        secure_cookies,
    };
    let app = quizinart::router(state, args.disable_rate_limit);

    let address = args.address.parse::<std::net::SocketAddr>()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!("listening on {}", address);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => tracing::info!("received Ctrl+C, shutting down"),
        () = terminate => tracing::info!("received SIGTERM, shutting down"),
    }
}
