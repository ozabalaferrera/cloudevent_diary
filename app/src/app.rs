mod config;
mod db;
mod endpoints;

#[cfg(test)]
mod tests;

use cloudevents::binding::warp::filter;
use config::*;
use envconfig::Envconfig;
use sqlx::{Pool, Postgres};
use tracing_panic::panic_hook;
use warp::{Filter, Reply};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    std::panic::set_hook(Box::new(panic_hook));

    if dotenv::dotenv().is_ok() {
        tracing::info!("Using dotenv file.");
    }

    let config = AppConfig::init_from_env().unwrap();

    // Use compile-time Cargo environment variables to set log string.
    tracing::info!(
        "Application {} version {} running on {}.",
        option_env!("CARGO_PKG_NAME").unwrap_or("[name]"),
        option_env!("CARGO_PKG_VERSION").unwrap_or("[version]"),
        config.hostname
    );

    // Use run-time container environment variables to set log string.
    tracing::info!(
        "Release {} revision {} of chart {} version {} in namespace {}.",
        config.helm_release_name,
        config.helm_release_revision,
        config.helm_chart_name,
        config.helm_chart_version,
        config.helm_release_namespace,
    );

    let pool = db::get_pool_from_config(&config).await;
    db::guarantee_db_components(
        pool.clone(),
        config.db_schema.clone(),
        config.db_table.clone(),
    )
    .await;

    warp::serve(filters(
        pool,
        config.db_schema.clone(),
        config.db_table.clone(),
    ))
    .run(([0, 0, 0, 0], config.web_port))
    .await;
}

/// Filter: Compose all of the app's filters with or().
fn filters(
    db_pool: Pool<Postgres>,
    db_schema: String,
    db_table: String,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    filter_entry(db_pool.clone(), db_schema, db_table).or(filter_health(db_pool))
}

/// Filter: POST /
///
/// Accepts only CloudEvents.
fn filter_entry(
    db_pool: Pool<Postgres>,
    db_schema: String,
    db_table: String,
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
    db_schema: String,
    db_table: String,
) -> impl Filter<Extract = (String, String), Error = std::convert::Infallible> + Clone {
    warp::any()
        .map(move || db_schema.clone())
        .and(warp::any().map(move || db_table.clone()))
}
