use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use jp_postal_code::{config, infra, usecase, MIGRATOR};
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

    let state = AppState {
        repo: infra::postgres::UtfKenAllRepositoryPostgres::new(pool),
    };
    let app = Router::new()
        .route("/api/search", get(search))
        .route("/api/update", post(update))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(conf.http_server_addr.as_str()).await?;
    tracing::info!("Listening on http://{}", conf.http_server_addr.as_str());
    axum::serve(listener, app).await?;

    Ok(())
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
    city: String,
    town: String,
}

#[derive(serde::Deserialize)]
struct SearchQuery {
    postal_code: Option<String>,
}

async fn search(
    axum::extract::Query(query): axum::extract::Query<SearchQuery>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let postal_code = query.postal_code.unwrap_or("".to_string());
    let records = usecase::search_postal_code(&state.repo, postal_code).await?;
    let addresses = records
        .into_iter()
        .map(|r| PostalAddress {
            postal_code: r.postal_code,
            prefecture: r.prefecture,
            city: r.city,
            town: r.town,
        })
        .collect::<Vec<_>>();
    Ok((StatusCode::OK, Json(addresses)))
}

async fn update(
    axum::extract::State(mut state): axum::extract::State<AppState>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    usecase::update_postal_code_database(&mut state.repo, None::<String>).await?;
    Ok(StatusCode::NO_CONTENT)
}
