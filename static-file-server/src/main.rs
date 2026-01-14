use axum::extract::State;
use axum::http::header::{InvalidHeaderName, InvalidHeaderValue};
use axum::http::{HeaderName, HeaderValue};
use axum::response::Response;
use axum::{Router, middleware};
use clap::{ArgAction, Parser};
use std::net::{AddrParseError, IpAddr};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::{select, signal};
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{Level, error, info};

/// Simple and configurable static file server.
#[derive(Parser)]
#[clap(disable_help_flag = true)]
#[command(version)]
struct Args {
  /// Print help.
  #[clap(long, action = ArgAction::HelpLong)]
  help: Option<bool>,

  /// Accept HTTP requests on the specified IP address.
  #[arg(short, long, default_value = "0.0.0.0")]
  address: String,

  /// Accept HTTP requests on the specified port.
  #[arg(short, long)]
  port: u16,

  /// Serve static files from the specified directory.
  #[arg(short, long)]
  directory: PathBuf,

  /// Add the specified header names to all HTTP responses.
  ///
  /// Provide the headers as a series of alternating name-value pairs.
  ///
  /// For example, the following invocation:
  ///
  /// ```
  /// static-file-server <...> -h X-Server -v http-server -h X-Protocol -v h2
  /// ```
  ///
  /// will result in the following HTTP headers being returned from requests:
  ///
  /// ```
  /// X-Server: http-server
  /// X-Protocol: h2
  /// ```
  #[arg(short, long)]
  header: Vec<String>,

  /// Add the specified header values to all HTTP responses.
  ///
  /// See the [`header`] argument for more information.
  #[arg(short, long)]
  value: Vec<String>,
}

#[derive(Debug, Error)]
enum AppError {
  #[error("could not parse header name: {0}")]
  ParseHeaderName(InvalidHeaderName),
  #[error("could not parse header value: {0}")]
  ParseHeaderValue(InvalidHeaderValue),
  #[error("could not parse IP address: {0}")]
  ParseIpAddress(AddrParseError),
  #[error("could not bind TCP socket: {0}")]
  BindTcpListener(std::io::Error),
  #[error("could not get TCP listener address: {0}")]
  GetListenerAddress(std::io::Error),
  #[error("could not start Axum server: {0}")]
  ServeApp(std::io::Error),
}

struct AppState {
  headers: Vec<(HeaderName, HeaderValue)>,
}

async fn apply_headers_middleware<B>(
  State(state): State<Arc<AppState>>,
  mut response: Response<B>,
) -> Response<B> {
  let headers = response.headers_mut();
  for (name, value) in &state.headers {
    headers.insert(name.clone(), value.clone());
  }

  response
}

#[tokio::main]
async fn start(args: &Args) -> Result<(), AppError> {
  let headers = args
    .header
    .iter()
    .zip(args.value.iter())
    .map(|(name, value)| {
      Ok((
        HeaderName::from_bytes(name.as_bytes()).map_err(AppError::ParseHeaderName)?,
        HeaderValue::from_str(value).map_err(AppError::ParseHeaderValue)?,
      ))
    })
    .collect::<Result<_, _>>()?;

  let state = Arc::new(AppState { headers });
  let app = Router::new()
    .fallback_service(ServeDir::new(args.directory.clone()))
    .layer(middleware::map_response_with_state(
      Arc::clone(&state),
      apply_headers_middleware,
    ))
    .layer(
      TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO)),
    )
    .with_state(state);

  let ip_address: IpAddr = args.address.parse().map_err(AppError::ParseIpAddress)?;
  let listener = TcpListener::bind((ip_address, args.port))
    .await
    .map_err(AppError::BindTcpListener)?;

  info!(
    "serving static files on http://{} (path: {})",
    listener
      .local_addr()
      .map_err(AppError::GetListenerAddress)?,
    args.directory.display(),
  );

  axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await
    .map_err(AppError::ServeApp)
}

fn main() {
  let args = Args::parse();

  tracing_subscriber::fmt()
    .with_target(false)
    .compact()
    .with_max_level(Level::DEBUG)
    .init();

  info!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

  match start(&args) {
    Ok(_) => info!("app exited successfully"),
    Err(err) => error!("app exited due to error: {}", err),
  };
}

async fn shutdown_signal() {
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
  select! {
    _ = ctrl_c => {},
    _ = terminate => {},
  }
}
