use crate::{db::get_pool_from_config, AppConfig};
use envconfig::Envconfig;
use std::time::{Duration, Instant};
use tokio::sync::OnceCell;
use warp::{Filter, Future, Reply};

static CONFIG: OnceCell<AppConfig> = OnceCell::const_new();
static DB_POOL: OnceCell<sqlx::Pool<sqlx::Postgres>> = OnceCell::const_new();
static INIT_DB: OnceCell<()> = OnceCell::const_new();

async fn init_config() -> &'static AppConfig {
    CONFIG
        .get_or_init(|| async {
            dotenv::dotenv().expect("Could not load .env for testing.");
            let env_hashmap = std::env::vars().into_iter().map(|(k, v)| (k, v)).collect();
            AppConfig::init_from_hashmap(&env_hashmap).unwrap()
        })
        .await
}

async fn get_db_pool(config: &AppConfig) -> &'static sqlx::Pool<sqlx::Postgres> {
    DB_POOL
        .get_or_init(|| async { get_pool_from_config(&config).await })
        .await
}

async fn init_db(db_pool: sqlx::Pool<sqlx::Postgres>, config: &AppConfig) {
    INIT_DB
        .get_or_init(|| async {
            crate::db::guarantee_db_components(
                db_pool,
                config.db_schema.clone(),
                config.db_table.clone(),
            )
            .await;
        })
        .await;
}

#[tokio::test]
async fn time_entries() {
    let filters = get_filters().await;

    let (_, duration) = time_async(async {
        let tasks: Vec<_> = (0..10_000)
            .into_iter()
            .map(|_| tokio::spawn(post_event(filters.clone())))
            .collect();

        for task in tasks {
            task.await.unwrap();
        }
    })
    .await;

    println!("Duration: {duration:?}");
}

#[tokio::test]
async fn db_ready() {
    let filters = get_filters().await;
    let req = warp::test::request().method("GET").path("/health/live");
    let res = req.reply(&filters).await;
    let status = res.status();
    assert_eq!(status, warp::http::StatusCode::ACCEPTED);
    let time = String::from_utf8(res.body().to_vec()).expect("Bad ready response.");
    println!("Time: {time}");
}

async fn get_filters() -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let config = init_config().await;
    let db_pool = get_db_pool(&config).await;
    init_db(db_pool.clone(), config).await;
    super::filters(
        db_pool.clone(),
        config.db_schema.clone(),
        config.db_table.clone(),
    )
}

async fn time_async<F, O>(f: F) -> (O, Duration)
where
    F: Future<Output = O>,
{
    let start = Instant::now();
    let out = f.await;
    let duration = start.elapsed();
    (out, duration)
}

async fn post_event(
    filters: impl warp::Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone + 'static,
) -> warp::http::Response<warp::hyper::body::Bytes> {
    let req = warp::test::request()
        .method("POST")
        .path("/")
        .header("ce-type", "com.acme.events.something")
        .header("ce-source", "com.acme.apps.ingress")
        .header("ce-id", uuid::Uuid::new_v4().to_string())
        .header("ce-specversion", "1.0")
        .header("ce-time", chrono::Utc::now().to_rfc3339())
        .header("ce-datacontenttype", "application/json")
        .header("ce-knativeerrorcode", 500)
        .header("ce-knativeerrordata", "")
        .header("ce-knativeerrordest", "http://acme.com")
        .body(r#"{"body":"hello world", "volume": 10}"#);

    req.reply(&filters).await
}
