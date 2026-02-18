use color_eyre::{eyre::OptionExt, Result};
use libsql::params::IntoParams;
use serde::de::DeserializeOwned;

/// Fetch all rows and deserialize each into `T` via `libsql::de::from_row`.
pub async fn query_all<T: DeserializeOwned>(
    conn: &libsql::Connection,
    sql: &str,
    params: impl IntoParams,
) -> Result<Vec<T>> {
    let mut rows = conn.query(sql, params).await?;
    let mut results = Vec::new();
    while let Some(row) = rows.next().await? {
        results.push(libsql::de::from_row::<T>(&row)?);
    }
    Ok(results)
}

/// Fetch the first row and deserialize into `T`. Errors if no rows returned.
pub async fn query_one<T: DeserializeOwned>(
    conn: &libsql::Connection,
    sql: &str,
    params: impl IntoParams,
) -> Result<T> {
    let row = conn
        .query(sql, params)
        .await?
        .next()
        .await?
        .ok_or_eyre("expected a row but got none")?;
    Ok(libsql::de::from_row::<T>(&row)?)
}

/// Fetch the first row and deserialize into `T`, or return `None` if no rows.
pub async fn query_optional<T: DeserializeOwned>(
    conn: &libsql::Connection,
    sql: &str,
    params: impl IntoParams,
) -> Result<Option<T>> {
    match conn.query(sql, params).await?.next().await? {
        Some(row) => Ok(Some(libsql::de::from_row::<T>(&row)?)),
        None => Ok(None),
    }
}
