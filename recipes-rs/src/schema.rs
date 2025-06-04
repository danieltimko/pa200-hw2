// @generated automatically by Diesel CLI.

diesel::table! {
    categories (name) {
        name -> Varchar,
    }
}

diesel::table! {
    ingredients (id) {
        id -> Int4,
        name -> Varchar,
    }
}

diesel::table! {
    recipe_category (recipe_id, category_name) {
        recipe_id -> Int4,
        category_name -> Varchar,
    }
}

diesel::table! {
    recipe_ingredient (recipe_id, ingredient_id) {
        recipe_id -> Int4,
        ingredient_id -> Int4,
        part -> Int2,
        quantity -> Numeric,
        unit -> Varchar,
    }
}

diesel::table! {
    recipes (id) {
        id -> Int4,
        name -> Varchar,
        instructions -> Text,
        cuisine -> Varchar,
        duration_min -> Int4,
        preparation_needed -> Bool,
        portions -> Int4,
        difficulty -> Int4,
    }
}

diesel::joinable!(recipe_category -> categories (category_name));
diesel::joinable!(recipe_category -> recipes (recipe_id));
diesel::joinable!(recipe_ingredient -> ingredients (ingredient_id));
diesel::joinable!(recipe_ingredient -> recipes (recipe_id));

diesel::allow_tables_to_appear_in_same_query!(
    categories,
    ingredients,
    recipe_category,
    recipe_ingredient,
    recipes,
);
