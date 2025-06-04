use crate::schema::recipes;
use diesel::{prelude::AsChangeset, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = recipes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Recipe {
    pub id: i32, // TODO: should be u32
    pub name: String,
    pub instructions: String,
    pub cuisine: String,
    pub duration_min: i32,
    pub preparation_needed: bool,
    pub portions: i32,
    pub difficulty: i32,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = recipes)]
pub struct NewRecipe {
    pub name: String,
    pub instructions: String,
    pub cuisine: String,
    pub duration_min: i32,
    pub preparation_needed: bool,
    pub portions: i32,
    pub difficulty: i32,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = recipes)]
pub struct ChangeRecipe {
    pub name: Option<String>,
    pub instructions: Option<String>,
    pub cuisine: Option<String>,
    pub duration_min: Option<i32>,
    pub preparation_needed: Option<bool>,
    pub portions: Option<i32>,
    pub difficulty: Option<i32>,
}
