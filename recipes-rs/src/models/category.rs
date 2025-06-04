use crate::models::recipe::Recipe;
use crate::schema::{categories, recipe_category};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = categories)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(name))]
pub struct Category {
    pub name: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = categories)]
pub struct NewCategory {
    pub name: String,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = categories)]
pub struct ChangeCategory {
    pub name: Option<String>,
}

#[derive(Identifiable, Queryable, Selectable, Associations, Insertable)]
#[diesel(table_name = recipe_category)]
#[diesel(belongs_to(Recipe))]
#[diesel(belongs_to(Category, foreign_key = category_name))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(recipe_id, category_name))]
pub struct RecipeCategory {
    pub recipe_id: i32,
    pub category_name: String,
}
