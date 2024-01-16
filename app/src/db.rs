use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

pub async fn get_pool_from_env() -> Pool<Postgres> {
    let db_user = env::var("DB_USER").unwrap_or("postgres".to_owned());
    let db_password = env::var("DB_PASSWORD").unwrap_or("postgres".to_owned());
    let db_host = env::var("DB_HOST").unwrap_or("postgres".to_owned());
    let db_name = env::var("DB_NAME").unwrap_or("postgres".to_owned());
    let db_url = format!("postgres://{db_user}:{db_password}@{db_host}/{db_name}");

    let web_concurrency = env::var("WEB_CONCURRENCY")
        .unwrap_or("10".to_owned())
        .parse()
        .expect("Invalid WEB_CONCURRENCY");

    PgPoolOptions::new()
        .max_connections(web_concurrency)
        .connect(db_url.as_str())
        .await
        .expect("Unable to connect to DB")
}

pub async fn guarantee_db_components(db_pool: Pool<Postgres>, db_schema: &str, db_table: &str) {
    tracing::info!("Guaranteeing table {db_schema}.{db_table} exists");
    sqlx::query(
        format!(
            r#"
            CREATE TABLE IF NOT EXISTS {db_schema}.{db_table}
            (
                id text NOT NULL,
                source text NOT NULL,
                type text NOT NULL,
                time timestamp with time zone,
                knativeerrorcode integer,
                knativeerrordata text,
                knativeerrordest text,
                data text,
                created_on timestamp with time zone NOT NULL DEFAULT NOW()
            );
            "#
        )
        .as_str(),
    )
    .execute(&db_pool)
    .await
    .expect("Unable to create table");
}
