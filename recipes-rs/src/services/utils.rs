use actix_web::web::Data;
use diesel_async::{
    pooled_connection::deadpool::{Object, Pool},
    AsyncPgConnection,
};
use serde::Serialize;

use super::errors;

#[derive(Serialize)]
pub struct ResponseBodyVec<T> {
    pub result: T,
}

pub async fn get_connection(
    pool: Data<Pool<AsyncPgConnection>>,
) -> Result<Object<AsyncPgConnection>, errors::ApiErrors> {
    return pool
        .get()
        .await
        .map_err(|_e| errors::ApiErrors::DatabaseConnectionError);
}
