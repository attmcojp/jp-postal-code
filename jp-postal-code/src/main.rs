use axum::{http::StatusCode, routing::get, Json, Router};
use jp_postal_code::{config, infra, repo::UtfKenAllRepository as _, usecase, MIGRATOR};
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

    let mut repo = infra::postgres::UtfKenAllRepositoryPostgres::new(pool);

    // 郵便番号データベースが空ならば初回ダウンロードを行う
    if repo.count().await? == 0 {
        tracing::info!("Postal address database is empty. Initializing...");
        usecase::update_postal_code_database(&mut repo, None::<String>).await?;
    }

    let state = AppState { repo };
    let app = Router::new()
        .route("/api/search", get(search))
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
