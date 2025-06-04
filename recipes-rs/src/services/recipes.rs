use actix_web::{
    delete, get,
    http::{header::ContentType, StatusCode},
    post, put, web, HttpResponse, Responder,
};
use diesel::expression::exists::exists;
use diesel::prelude::*;
use diesel::select;
use diesel_async::{
    pooled_connection::deadpool::{Object, Pool},
    AsyncPgConnection, RunQueryDsl,
};
use serde::{Deserialize, Serialize};

use crate::{
    models::{
        category::{Category, RecipeCategory},
        recipe::{ChangeRecipe, NewRecipe, Recipe},
    },
    schema::{
        categories::{self},
        recipe_category,
        recipes::{self},
    },
    services::{errors, utils},
};

#[derive(Serialize)]
struct RecipeBody {
    #[serde(flatten)]
    recipe: Recipe,
    categories: Vec<Category>,
}

#[derive(Deserialize)]
struct ChangeRecipeBody {
    #[serde(flatten)]
    recipe_change: ChangeRecipe,
    categories: Option<Vec<String>>,
}

async fn get_categories(
    recipe: &Recipe,
    connection: &mut Object<AsyncPgConnection>,
) -> Result<Vec<Category>, errors::ApiErrors> {
    return categories::table
        .select(Category::as_select())
        .inner_join(recipe_category::table)
        .filter(recipe_category::recipe_id.eq(recipe.id))
        .load(connection)
        .await
        .map_err(|_e| errors::ApiErrors::InternalError);
}

#[get("")]
pub async fn recipes_list(
    pool: web::Data<Pool<AsyncPgConnection>>,
) -> actix_web::Result<impl Responder> {
    // TODO: logging
    let mut connection = utils::get_connection(pool).await?;

    let recipes_db = recipes::table
        .select(Recipe::as_select())
        .load(&mut connection)
        .await;
    let recipes_vec = recipes_db.map_err(|_e| errors::ApiErrors::InternalError)?;

    let mut recipes_full: Vec<RecipeBody> = vec![];
    for recipe in recipes_vec {
        let categories = get_categories(&recipe, &mut connection)
            .await
            .map_err(|_e| errors::ApiErrors::InternalError)?;
        recipes_full.push(RecipeBody { recipe, categories });
    }

    let response_body = utils::ResponseBodyVec {
        result: recipes_full,
    };
    let response_serialized =
        serde_json::to_string(&response_body).map_err(|_e| errors::ApiErrors::InternalError)?;

    return Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response_serialized));
}

#[get("/{id}")]
pub async fn recipes_get(
    pool: web::Data<Pool<AsyncPgConnection>>,
    path: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    use crate::schema::recipes::dsl::*;
    let recipe_id = path.into_inner();

    let mut connection = utils::get_connection(pool).await?;
    let recipe: Recipe = recipes
        .find(recipe_id)
        .first(&mut connection)
        .await
        .map_err(|_e| errors::ApiErrors::NotFound)?;
    let categories = get_categories(&recipe, &mut connection)
        .await
        .map_err(|_e| errors::ApiErrors::InternalError)?;
    let response = RecipeBody { recipe, categories };

    let response_serialized =
        serde_json::to_string(&response).map_err(|_e| errors::ApiErrors::InternalError)?;

    return Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response_serialized));
}

#[derive(Deserialize)]
struct NewRecipeBody {
    #[serde(flatten)]
    recipe: NewRecipe,
    categories: Vec<String>,
}

#[post("")]
pub async fn recipes_create(
    pool: web::Data<Pool<AsyncPgConnection>>,
    recipe_body: web::Json<NewRecipeBody>,
) -> actix_web::Result<impl Responder> {
    use crate::schema::categories::dsl::{name as category_name, *};
    use crate::schema::recipe_category::dsl::*;
    use crate::schema::recipes::dsl::*;
    let recipe_body = recipe_body.into_inner();
    let mut connection = utils::get_connection(pool).await?;

    for category in &recipe_body.categories {
        let category_exists: bool = select(exists(categories.filter(category_name.eq(&category))))
            .get_result(&mut connection)
            .await
            .map_err(|_e| errors::ApiErrors::InternalError)?;
        if !category_exists {
            return Ok(HttpResponse::NotFound()
                .content_type(ContentType::json())
                .body("Category not found"));
        }
    }

    let recipe: Recipe = diesel::insert_into(recipes)
        .values(&recipe_body.recipe)
        .returning(Recipe::as_returning())
        .get_result(&mut connection)
        .await
        .map_err(|_e| errors::ApiErrors::InternalError)?;

    for category in recipe_body.categories {
        let cat_assoc = RecipeCategory {
            recipe_id: recipe.id,
            category_name: category,
        };
        diesel::insert_into(recipe_category)
            .values(&cat_assoc)
            .execute(&mut connection)
            .await
            .map_err(|_e| errors::ApiErrors::InternalError)?;
    }
    let response_serialized =
        serde_json::to_string(&recipe).map_err(|_e| errors::ApiErrors::InternalError)?;

    return Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response_serialized));
}

#[put("/{id}")]
pub async fn recipes_change(
    pool: web::Data<Pool<AsyncPgConnection>>,
    path: web::Path<i32>,
    recipe_changeset_body: web::Json<ChangeRecipeBody>,
) -> actix_web::Result<impl Responder> {
    use crate::schema::categories::dsl::{name as category_name, *};
    use crate::schema::recipe_category::dsl::*;
    use crate::schema::recipes::dsl::*;
    let recipe_id_path = path.into_inner();
    let recipe_changeset_body = recipe_changeset_body.into_inner();
    let recipe_changeset = recipe_changeset_body.recipe_change;

    let mut connection = utils::get_connection(pool).await?;

    if let Some(categories_str) = &recipe_changeset_body.categories {
        // check if requested categories exist
        for category in categories_str {
            let category_exists: bool =
                select(exists(categories.filter(category_name.eq(&category))))
                    .get_result(&mut connection)
                    .await
                    .map_err(|_e| errors::ApiErrors::InternalError)?;
            if !category_exists {
                return Ok(HttpResponse::NotFound()
                    .content_type(ContentType::json())
                    .body("Category not found"));
            }
        }
    }

    let recipe: Recipe = diesel::update(recipes.find(recipe_id_path))
        .set(&recipe_changeset)
        .returning(Recipe::as_returning())
        .get_result(&mut connection)
        .await
        .map_err(|_e| errors::ApiErrors::InternalError)?;

    if let Some(categories_str) = recipe_changeset_body.categories {
        diesel::delete(recipe_category.filter(recipe_id.eq(recipe.id)))
            .execute(&mut connection)
            .await
            .map_err(|_e| errors::ApiErrors::InternalError)?;
        for category in categories_str {
            let cat_assoc = RecipeCategory {
                recipe_id: recipe.id,
                category_name: category,
            };
            diesel::insert_into(recipe_category)
                .values(&cat_assoc)
                .execute(&mut connection)
                .await
                .map_err(|_e| errors::ApiErrors::InternalError)?;
        }
    }
    let categories_vec = get_categories(&recipe, &mut connection).await?;
    let recipe_body = RecipeBody {
        recipe,
        categories: categories_vec,
    };
    let response_serialized =
        serde_json::to_string(&recipe_body).map_err(|_e| errors::ApiErrors::InternalError)?;

    return Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response_serialized));
}

#[delete("/{id}")]
pub async fn recipes_delete(
    pool: web::Data<Pool<AsyncPgConnection>>,
    path: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    use crate::schema::recipe_category::dsl::*;
    use crate::schema::recipes::dsl::*;
    let recipe_id_path = path.into_inner();

    let mut connection = utils::get_connection(pool).await?;

    diesel::delete(recipe_category.filter(recipe_id.eq(&recipe_id_path)))
        .execute(&mut connection)
        .await
        .map_err(|_e| errors::ApiErrors::InternalError)?;

    diesel::delete(recipes.find(recipe_id_path))
        .execute(&mut connection)
        .await
        .map_err(|_e| errors::ApiErrors::InternalError)?;

    return Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .status(StatusCode::NO_CONTENT)
        .finish());
}

pub fn recipes_config(cfg: &mut web::ServiceConfig) {
    cfg.service(recipes_list);
    cfg.service(recipes_get);
    cfg.service(recipes_create);
    cfg.service(recipes_change);
    cfg.service(recipes_delete);
}
