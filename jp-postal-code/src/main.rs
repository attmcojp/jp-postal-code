use anyhow::Context as _;
use axum::{http::StatusCode, routing::get, Json, Router};
use jp_postal_address::postal_address_service_server::PostalAddressServiceServer;
use jp_postal_code::{
    config, grpc_service, infra, repo::UtfKenAllRepository as _, usecase, MIGRATOR,
};
use std::net::ToSocketAddrs;
use tonic::transport::Server;
use tower_http::trace::{DefaultOnFailure, DefaultOnResponse, TraceLayer};
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    // `.env` ファイルが存在するなら環境変数として読む
    let _ = dotenvy::dotenv();
    // ロギングの設定
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "jp_postal_code=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    if let Err(err) = main_internal().await {
        tracing::error!(?err, "Unexpected error occurred.");
        std::process::exit(1);
    }
}

async fn main_internal() -> Result<(), anyhow::Error> {
    let conf = config::Config::new();
    let pool = sqlx::PgPool::connect(conf.database_url.as_ref()).await?;
    MIGRATOR.run(&pool).await?;

    let mut repo = infra::postgres::UtfKenAllRepositoryPostgres::new(pool.clone());

    // 郵便番号データベースが空ならば初回ダウンロードを行う
    if repo.count().await? == 0 {
        tracing::info!("Postal address database is empty. Initializing...");
        usecase::update_postal_code_database(&mut repo, None::<String>).await?;
    }

    // HTTP サーバーの設定
    let http_state = AppState {
        repo: infra::postgres::UtfKenAllRepositoryPostgres::new(pool.clone()),
    };
    let http_app = Router::new()
        .route("/api/search", get(search))
        .layer(
            TraceLayer::new_for_http()
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO))
                .on_failure(DefaultOnFailure::new().level(tracing::Level::ERROR)),
        )
        .with_state(http_state);

    // gRPC サーバーの設定
    let grpc_service = grpc_service::PostalAddressServiceImpl::new(
        infra::postgres::UtfKenAllRepositoryPostgres::new(pool),
    );

    let http_addr = conf.http_server_addr.clone();
    let grpc_addr = conf.grpc_server_addr.clone();

    // HTTP と gRPC サーバーを並行実行
    tokio::try_join!(
        start_http_server(http_addr, http_app),
        start_grpc_server(grpc_addr, grpc_service)
    )?;
    Ok(())
}

async fn start_http_server(addr: String, app: Router) -> Result<(), anyhow::Error> {
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind HTTP server address: {}", addr))?;
    tracing::info!("HTTP server listening on http://{}", addr);
    axum::serve(listener, app)
        .await
        .map_err(anyhow::Error::from)
}

async fn start_grpc_server(
    addr: String,
    service: grpc_service::PostalAddressServiceImpl,
) -> Result<(), anyhow::Error> {
    let addr: std::net::SocketAddr = addr
        .to_socket_addrs()
        .with_context(|| format!("Failed to parse gRPC address: {}", addr))?
        .next()
        .with_context(|| format!("No valid address is parsed from gRPC address: {}", addr))?;
    tracing::info!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(PostalAddressServiceServer::new(service))
        .serve(addr)
        .await
        .map_err(anyhow::Error::from)
}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self),
        )
            .into_response()
    }
}

#[derive(Debug, Clone)]
struct AppState {
    repo: infra::postgres::UtfKenAllRepositoryPostgres,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PostalAddress {
    postal_code: String,
    prefecture: String,
    prefecture_kana: String,
    city: String,
    city_kana: String,
    town: String,
    town_kana: String,
}

#[derive(serde::Deserialize)]
struct SearchQuery {
    postal_code: Option<String>,
    page_size: Option<usize>,
    page_token: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchResponse {
    addresses: Vec<PostalAddress>,
    next_page_token: Option<String>,
}

async fn search(
    axum::extract::Query(query): axum::extract::Query<SearchQuery>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let postal_code = query.postal_code.unwrap_or("".to_string());
    let response = usecase::search_postal_code(
        &state.repo,
        usecase::SearchPostalCodeRequest {
            postal_code,
            page_size: query.page_size,
            page_token: query.page_token,
        },
    )
    .await?;
    let addresses = response
        .records
        .into_iter()
        .map(|r| PostalAddress {
            postal_code: r.postal_code,
            prefecture: r.prefecture,
            prefecture_kana: r.prefecture_kana,
            city: r.city,
            city_kana: r.city_kana,
            town: r.town,
            town_kana: r.town_kana,
        })
        .collect::<Vec<_>>();
    Ok((
        StatusCode::OK,
        Json(SearchResponse {
            addresses,
            next_page_token: response.next_page_token,
        }),
    ))
}
