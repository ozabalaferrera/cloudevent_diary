use cloudevents::{event::ExtensionValue, AttributesReader, Data, Event};
use sqlx::{Pool, Postgres, Row};
use std::{convert::Infallible, sync::Arc};
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
    db_schema: Arc<String>,
    db_table: Arc<String>,
) -> Result<impl Reply, Infallible> {
    tracing::debug!("{}", &event);

    let id = event.id();
    let source = event.source();
    let ty = event.ty();
    let time = event.time();

    let knativeerrorcode = match event.extension("knativeerrorcode") {
        Some(ExtensionValue::Integer(i)) => Some(*i),
        Some(ExtensionValue::String(s)) => match s.parse() {
            Ok(i) => Some(i),
            Err(e) => {
                tracing::warn!(
                    "Could not parse cloudevent attribute 'knativeerrorcode' to integer: {e}"
                );
                None
            }
        },
        _ => None,
    };

    let knativeerrordata_string: String;
    let knativeerrordata = match event.extension("knativeerrordata") {
        Some(ExtensionValue::String(s)) => Some(s.as_str()),
        Some(ExtensionValue::Integer(i)) => {
            knativeerrordata_string = i.to_string();
            Some(knativeerrordata_string.as_str())
        }
        Some(ExtensionValue::Boolean(b)) => {
            knativeerrordata_string = b.to_string();
            Some(knativeerrordata_string.as_ref())
        }
        _ => None,
    };

    let knativeerrordest_string: String;
    let knativeerrordest = match event.extension("knativeerrordest") {
        Some(ExtensionValue::String(s)) => Some(s.as_str()),
        Some(ExtensionValue::Integer(i)) => {
            knativeerrordest_string = i.to_string();
            Some(knativeerrordest_string.as_str())
        }
        Some(ExtensionValue::Boolean(b)) => {
            knativeerrordest_string = b.to_string();
            Some(knativeerrordest_string.as_ref())
        }
        _ => None,
    };

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
        (id, source, type, time, knativeerrorcode, knativeerrordata, knativeerrordest, data)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#
        )
        .as_str(),
    )
    .bind(id)
    .bind(source)
    .bind(ty)
    .bind(time)
    .bind(knativeerrorcode)
    .bind(knativeerrordata)
    .bind(knativeerrordest)
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
