use cloudevents::{event::ExtensionValue, AttributesReader, Data, Event};
use sqlx::{Pool, Postgres, Row};
use std::convert::Infallible;
use warp::{http::StatusCode, reply::Reply};

pub async fn started() -> Result<impl Reply, Infallible> {
    Ok(StatusCode::ACCEPTED)
}

pub async fn ready() -> Result<impl Reply, Infallible> {
    Ok(StatusCode::ACCEPTED)
}

pub async fn live(db_pool: Pool<Postgres>) -> Result<impl Reply, Infallible> {
    let res = sqlx::query(r#"SELECT TO_CHAR(CURRENT_TIMESTAMP, 'YYYY-MM-DD"T"HH24:MI:SS');"#)
        .fetch_one(&db_pool)
        .await;

    match res {
        Ok(row) => match row.get(0) {
            Some(time) => {
                tracing::debug!(time);
                Ok(warp::reply::with_status(time, StatusCode::ACCEPTED))
            }
            None => {
                let msg = "Could not convert time.";
                tracing::error!(msg);
                Ok(warp::reply::with_status(
                    msg.to_owned(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        },
        Err(err) => {
            tracing::error!("{err}");
            Ok(warp::reply::with_status(
                err.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

pub async fn ce_entry(
    event: Event,
    db_pool: Pool<Postgres>,
    db_schema: String,
    db_table: String,
) -> Result<impl Reply, Infallible> {
    tracing::debug!("{}", &event);

    let extensions: sqlx::types::JsonValue = sqlx::types::JsonValue::Object(
        event
            .iter_extensions()
            .map(|(k, v)| {
                (
                    k.to_owned(),
                    match v {
                        ExtensionValue::Boolean(b) => sqlx::types::JsonValue::Bool(*b),
                        ExtensionValue::Integer(i) => sqlx::types::JsonValue::Number((*i).into()),
                        ExtensionValue::String(s) => sqlx::types::JsonValue::String(s.to_owned()),
                    },
                )
            })
            .collect(),
    );

    let data_string: String;
    let data = match event.data() {
        Some(Data::Json(v)) => {
            data_string = v.to_string();
            Some(data_string.as_str())
        }
        Some(Data::String(s)) => Some(s.as_str()),
        Some(Data::Binary(b)) => match String::from_utf8(b.to_owned()) {
            Ok(base64) => {
                data_string = base64;
                Some(data_string.as_str())
            }
            Err(e) => {
                tracing::warn!("Could not parse cloudevent binary data to string: {e}");
                None
            }
        },
        None => None,
    };

    let res = sqlx::query(
        format!(
            r#"
        INSERT INTO {db_schema}.{db_table}
        (specversion, id, source, type, datacontenttype, dataschema, subject, time, extensions, data)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#
        )
        .as_str(),
    )
    .bind(event.specversion().to_string())
    .bind(event.id())
    .bind(event.source())
    .bind(event.ty())
    .bind(event.datacontenttype())
    .bind(event.dataschema().map(|d| d.to_string()))
    .bind(event.subject())
    .bind(event.time())
    .bind(extensions)
    .bind(data)
    .execute(&db_pool)
    .await;

    match res {
        Ok(_) => {
            let msg = format!("Inserted row in {db_schema}.{db_table}.");
            tracing::debug!(msg);
            Ok(warp::reply::with_status(
                msg.to_owned(),
                StatusCode::ACCEPTED,
            ))
        }
        Err(err) => {
            tracing::error!("{err}");
            Ok(warp::reply::with_status(
                err.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}
