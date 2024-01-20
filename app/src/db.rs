use crate::AppConfig;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub async fn get_pool_from_config(config: &AppConfig) -> Pool<Postgres> {
    let db_url = format!(
        "postgres://{}:{}@{}/{}",
        config.db_user, config.db_password, config.db_host, config.db_name
    );

    PgPoolOptions::new()
        .max_connections(config.db_pool_size)
        .connect(db_url.as_str())
        .await
        .unwrap()
}

pub async fn guarantee_db_components(db_pool: Pool<Postgres>, db_schema: String, db_table: String) {
    tracing::info!("Guaranteeing table {db_schema}.{db_table} exists");
    sqlx::query(
        format!(
            r#"
            CREATE TABLE IF NOT EXISTS {db_schema}.{db_table}
            (
                specversion text NOT NULL,
                id text NOT NULL,
                source text NOT NULL,
                type text NOT NULL,
                datacontenttype text,
                dataschema text,
                subject text,
                time timestamp with time zone,
                extensions json,
                data text,
                created_on timestamp with time zone NOT NULL DEFAULT NOW()
            );
            "#
        )
        .as_str(),
    )
    .execute(&db_pool)
    .await
    .unwrap();
}
