use std::i16;

use crate::models::recipe::Recipe;
use crate::schema::{ingredients, recipe_ingredient};
use bigdecimal::BigDecimal;
use diesel::{Associations, Identifiable, Insertable, Queryable, Selectable};

#[derive(Queryable, Selectable)]
#[diesel(table_name = ingredients)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Ingredient {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable)]
#[diesel(table_name = ingredients)]
pub struct NewIngredient<'a> {
    pub name: &'a str,
}

#[derive(Identifiable, Queryable, Selectable, Associations, Insertable)]
#[diesel(table_name = recipe_ingredient)]
#[diesel(belongs_to(Recipe))]
#[diesel(belongs_to(Ingredient))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(recipe_id, ingredient_id))]
pub struct RecipeIngredient {
    pub recipe_id: i32,
    pub ingredient_id: i32,
    pub part: i16,
    pub quantity: BigDecimal,
    pub unit: String,
}
