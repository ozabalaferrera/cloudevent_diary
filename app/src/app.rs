mod db;
mod endpoints;

#[cfg(test)]
mod tests;

use cloudevents::binding::warp::filter;
use sqlx::{Pool, Postgres};
use std::{env, sync::Arc};
use warp::{Filter, Reply};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    if dotenv::dotenv().is_ok() {
        tracing::info!("Using dotenv file.");
    }

    let hostname = env::var("HOSTNAME").unwrap_or("[hostname]".to_owned());
    let db_schema = Arc::new(env::var("DB_SCHEMA").unwrap_or("public".to_owned()));
    let db_table = Arc::new(env::var("DB_TABLE").unwrap_or("cloudevents_diary".to_owned()));

    let web_port = env::var("WEB_PORT")
        .unwrap_or("8080".to_owned())
        .parse()
        .expect("Invalid WEB_PORT");

    let addr = format!("0.0.0.0:{web_port}");

    // Use compile-time Cargo environment variables to set log string.
    tracing::info!(
        "Application {} version {} running on {}.",
        option_env!("CARGO_PKG_NAME").unwrap_or("[name]"),
        option_env!("CARGO_PKG_VERSION").unwrap_or("[version]"),
        hostname
    );

    // Use run-time container environment variables to set log string.
    tracing::info!(
        "Release {} revision {} of chart {} version {} in namespace {}.",
        env::var("HELM_RELEASE_NAME").unwrap_or("[name]".to_owned()),
        env::var("HELM_RELEASE_REVISION").unwrap_or("[version]".to_owned()),
        env::var("HELM_CHART_NAME").unwrap_or("[name]".to_owned()),
        env::var("HELM_CHART_VERSION").unwrap_or("[version]".to_owned()),
        env::var("HELM_RELEASE_NAMESPACE").unwrap_or("[namespace]".to_owned()),
    );

    let pool = db::get_pool_from_env().await;
    db::guarantee_db_components(pool.clone(), db_schema.as_str(), db_table.as_str()).await;

    tracing::info!("Listening on: {}", addr);
    // Start server.
    warp::serve(filters(pool, db_schema, db_table))
        .run(([0, 0, 0, 0], web_port))
        .await;
}

/// Filter: Compose all of the app's filters with or().
fn filters(
    db_pool: Pool<Postgres>,
    db_schema: Arc<String>,
    db_table: Arc<String>,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    filter_entry(db_pool.clone(), db_schema, db_table).or(filter_health(db_pool))
}

/// Filter: POST /
///
/// Accepts only CloudEvents.
fn filter_entry(
    db_pool: Pool<Postgres>,
    db_schema: Arc<String>,
    db_table: Arc<String>,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path::end())
        .and(filter::to_event())
        .and(with_pool(db_pool))
        .and(with_table_details(db_schema, db_table))
        .and_then(endpoints::ce_entry)
}

/// Filter: GET /health/*
///
/// Respond to various health probes.
fn filter_health(
    db_pool: Pool<Postgres>,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("health" / "started"))
        .and_then(endpoints::started)
        .or(warp::get()
            .and(warp::path!("health" / "ready"))
            .and_then(endpoints::ready))
        .or(warp::get()
            .and(warp::path!("health" / "live"))
            .and(with_pool(db_pool))
            .and_then(endpoints::live))
}

fn with_pool(
    pool: Pool<Postgres>,
) -> impl Filter<Extract = (Pool<Postgres>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

fn with_table_details(
    db_schema: Arc<String>,
    db_table: Arc<String>,
) -> impl Filter<Extract = (Arc<String>, Arc<String>), Error = std::convert::Infallible> + Clone {
    warp::any()
        .map(move || db_schema.clone())
        .and(warp::any().map(move || db_table.clone()))
}
