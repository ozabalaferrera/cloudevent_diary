use std::time::{Duration, Instant};
use std::{env, sync::Arc};
use warp::{Future, Reply};

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
        .header("ce-id", "370058fc-0d71-11ee-be56-0242ac120002")
        .header("ce-specversion", "1.0")
        .header("ce-time", "2023-07-02T00:00:00Z")
        .header("ce-datacontenttype", "application/json")
        .header("ce-something", "no nulls")
        .body(r#"{"body":"hello world", "volume": 10}"#);

    req.reply(&filters).await
}

#[tokio::test]
async fn time_entries() {
    dotenv::dotenv().expect("Could not load .env for testing.");

    let db_schema = Arc::new(env::var("DB_SCHEMA").unwrap_or("public".to_owned()));
    let db_table = Arc::new(env::var("DB_TABLE").unwrap_or("cloudevents_diary".to_owned()));
    let db_pool = crate::db::get_pool_from_env().await;
    crate::db::guarantee_db_components(db_pool.clone(), db_schema.as_str(), db_table.as_str())
        .await;
    let filters = super::filters(db_pool, db_schema, db_table);

    let (_, duration) = time_async(async {
        for _i in 0..10_000 {
            post_event(filters.clone()).await;
        }
    })
    .await;

    println!("Duration: {duration:?}");
}
