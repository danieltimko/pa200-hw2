pub mod schema;

use actix_web::{error, get, web, App, HttpResponse, HttpServer, Responder};
use diesel::prelude::*;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use dotenvy::dotenv;
use schema::recipes;
use serde::Serialize;
use std::{env, io};

mod models {
    pub mod category;
    pub mod ingredient;
    pub mod recipe;
}
use models::recipe::Recipe;

mod services;
use services::{categories::categories_config, recipes::recipes_config};

#[derive(Serialize)]
struct ResponseBodyVec<T> {
    pub result: T,
}

#[get("/")]
async fn hello(pool: web::Data<Pool<AsyncPgConnection>>) -> actix_web::Result<impl Responder> {
    let mut connection = match pool.get().await {
        Ok(conn) => conn,
        Err(_) => {
            return Err(error::ErrorInternalServerError(
                "Unable to get connection from pool",
            ))
        }
    };

    let db_result = recipes::table
        .select(Recipe::as_select())
        .load(&mut connection)
        .await;
    let recipes = match db_result {
        Ok(recipes) => recipes,
        Err(_) => {
            return Err(error::ErrorInternalServerError(
                "Cannot get recipes from the database",
            ))
        }
    };

    let response_body = ResponseBodyVec { result: recipes };
    let respose_serialized = match serde_json::to_string(&response_body) {
        Ok(res) => res,
        Err(_) => return Err(error::ErrorInternalServerError("Cannot serialize recipes")),
    };
    return Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(respose_serialized));
}

#[get("/static")]
async fn static_images() -> impl Responder {
    let html_content = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <title>Recipe Images</title>
        <style>
            img {
                width: 300px;
                height: 200px;
                margin: 10px;
            }
            body {
                display: flex;
                justify-content: center;
                align-items: center;
                flex-direction: column;
            }
        </style>
    </head>
    <body>
        <h1>Static content from S3</h1>
        <img src="https://recipes-rs-static-content.s3.eu-west-1.amazonaws.com/recipe1.jpg" alt="Recipe 1">
        <img src="https://recipes-rs-static-content.s3.eu-west-1.amazonaws.com/recipe2.jpg" alt="Recipe 2">
        <img src="https://recipes-rs-static-content.s3.eu-west-1.amazonaws.com/recipe3.jpg" alt="Recipe 3">
    </body>
    </html>
    "#;

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_content)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool_manager =
        AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    let pool = Pool::builder(pool_manager)
        .build()
        .expect("Unable to build a connection pool");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(hello)
            .service(static_images)
            .service(web::scope("/recipes").configure(recipes_config))
            .service(web::scope("/categories").configure(categories_config))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
