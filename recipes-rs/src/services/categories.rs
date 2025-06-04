use actix_web::{
    delete, get,
    http::{header::ContentType, StatusCode},
    post, put, web, HttpResponse, Responder,
};
use diesel::prelude::*;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection, RunQueryDsl};

use crate::{
    models::category::{Category, ChangeCategory, NewCategory},
    schema::categories,
    services::{errors, utils},
};

#[get("")]
pub async fn categories_list(
    pool: web::Data<Pool<AsyncPgConnection>>,
) -> actix_web::Result<impl Responder> {
    let mut connection = utils::get_connection(pool).await?;

    let db_result = categories::table
        .select(Category::as_select())
        .load(&mut connection)
        .await;
    let categories_vec = db_result.map_err(|_e| errors::ApiErrors::InternalError)?;

    let response_body = utils::ResponseBodyVec {
        result: categories_vec,
    };
    let response_serialized =
        serde_json::to_string(&response_body).map_err(|_e| errors::ApiErrors::InternalError)?;

    return Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response_serialized));
}

#[get("/{id}")]
pub async fn categories_get(
    pool: web::Data<Pool<AsyncPgConnection>>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    use crate::schema::categories::dsl::*;
    let category_name = path.into_inner();

    let mut connection = utils::get_connection(pool).await?;

    let category: Category = categories
        .find(category_name)
        .first(&mut connection)
        .await
        .map_err(|_e| errors::ApiErrors::NotFound)?;
    let response_serialized =
        serde_json::to_string(&category).map_err(|_e| errors::ApiErrors::InternalError)?;

    return Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response_serialized));
}

#[post("")]
pub async fn categories_create(
    pool: web::Data<Pool<AsyncPgConnection>>,
    category_body: web::Json<NewCategory>,
) -> actix_web::Result<impl Responder> {
    use crate::schema::categories::dsl::*;

    let mut connection = utils::get_connection(pool).await?;

    let category: Category = diesel::insert_into(categories)
        .values(&category_body.into_inner())
        .returning(Category::as_returning())
        .get_result(&mut connection)
        .await
        .map_err(|_e| errors::ApiErrors::InternalError)?;
    let response_serialized =
        serde_json::to_string(&category).map_err(|_e| errors::ApiErrors::InternalError)?;

    return Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response_serialized));
}

#[put("/{id}")]
pub async fn categories_change(
    pool: web::Data<Pool<AsyncPgConnection>>,
    path: web::Path<String>,
    category_changeset: web::Json<ChangeCategory>,
) -> actix_web::Result<impl Responder> {
    use crate::schema::categories::dsl::*;
    let category_name = path.into_inner();
    let category_changeset = category_changeset.into_inner();

    let mut connection = utils::get_connection(pool).await?;

    let category: Category = diesel::update(categories.find(category_name))
        .set(&category_changeset)
        .returning(Category::as_returning())
        .get_result(&mut connection)
        .await
        .map_err(|_e| errors::ApiErrors::InternalError)?;
    let response_serialized =
        serde_json::to_string(&category).map_err(|_e| errors::ApiErrors::InternalError)?;

    return Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response_serialized));
}

#[delete("/{id}")]
pub async fn categories_delete(
    pool: web::Data<Pool<AsyncPgConnection>>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    use crate::schema::categories::dsl::*;
    use crate::schema::recipe_category::dsl::*;
    let category_name_path = path.into_inner();

    let mut connection = utils::get_connection(pool).await?;

    diesel::delete(recipe_category.filter(category_name.eq(&category_name_path)))
        .execute(&mut connection)
        .await
        .map_err(|_e| errors::ApiErrors::InternalError)?;

    diesel::delete(categories.find(category_name_path))
        .execute(&mut connection)
        .await
        .map_err(|_e| errors::ApiErrors::InternalError)?;

    return Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .status(StatusCode::NO_CONTENT)
        .finish());
}

pub fn categories_config(cfg: &mut web::ServiceConfig) {
    cfg.service(categories_list);
    cfg.service(categories_get);
    cfg.service(categories_create);
    cfg.service(categories_change);
    cfg.service(categories_delete);
}
