table! {
    base_plants (id) {
        id -> Integer,
        name -> Text,
        name_fts -> Text,
        #[sql_name = "type"]
        type_ -> Text,
        notoriety_score -> Nullable<Float>,
        notoriety_score_explanation -> Nullable<Text>,
        number_of_references -> Integer,
        notoriety_highest_collection_score -> Nullable<Float>,
        notoriety_highest_collection_score_id -> Nullable<Integer>,
        aka -> Nullable<Text>,
        aka_fts -> Nullable<Text>,
        marketing_name -> Nullable<Text>,
        description -> Nullable<Text>,
        uspp_number -> Nullable<Text>,
        uspp_expiration -> Nullable<BigInt>,
        uspp_expiration_estimated -> Nullable<Integer>,
        release_year -> Nullable<Integer>,
        release_year_note -> Nullable<Text>,
        released_by -> Nullable<Text>,
        release_collection_id -> Nullable<Integer>,
        ignore_unless_in_others -> Integer,
        s_allele -> Nullable<Text>,
        harvest_relative -> Nullable<Integer>,
        harvest_relative_to -> Nullable<Text>,
        harvest_relative_to_type -> Nullable<Text>,
        harvest_relative_explanation -> Nullable<Text>,
    }
}

table! {
    collection_categories (id) {
        id -> Integer,
        collection_id -> Integer,
        category -> Text,
    }
}

table! {
    collection_items (id) {
        id -> Integer,
        collection_id -> Nullable<Integer>,
        location_id -> Nullable<Integer>,
        location_number -> Integer,
        user_id -> Nullable<Integer>,
        public -> Nullable<Integer>,
        path_and_filename -> Nullable<Text>,
        marketing_name -> Nullable<Text>,
        name -> Text,
        #[sql_name = "type"]
        type_ -> Text,
        base_plant_id -> Nullable<Integer>,
        category -> Nullable<Text>,
        category_description -> Nullable<Text>,
        disease_resistance -> Nullable<Text>,
        chill -> Nullable<Text>,
        s_allele -> Nullable<Text>,
        description -> Nullable<Text>,
        harvest_text -> Nullable<Text>,
        harvest_relative -> Nullable<Text>,
        harvest_start -> Nullable<Integer>,
        harvest_end -> Nullable<Integer>,
        harvest_start_2 -> Nullable<Integer>,
        harvest_end_2 -> Nullable<Integer>,
        calc_harvest_relative -> Nullable<Integer>,
        calc_harvest_relative_to -> Nullable<Text>,
        calc_harvest_relative_to_type -> Nullable<Text>,
        calc_harvest_relative_round -> Nullable<Double>,
        calc_harvest_relative_explanation -> Nullable<Text>,
    }
}

table! {
    collections (id) {
        id -> Integer,
        git_edit_time -> Nullable<BigInt>,
        path -> Text,
        filename -> Text,
        notoriety_type -> Text,
        notoriety_score -> Float,
        notoriety_score_explanation -> Text,
        harvest_time_devalue_factor -> Nullable<Float>,
        ignore_for_nearby_searches -> Integer,
        title -> Nullable<Text>,
        author -> Nullable<Text>,
        description -> Nullable<Text>,
        url -> Nullable<Text>,
        published -> Nullable<Text>,
        reviewed -> Nullable<Text>,
        accessed -> Nullable<Text>,
        needs_help -> Integer,
    }
}

table! {
    facts (id) {
        id -> Integer,
        contributor -> Text,
        fact -> Text,
        reference -> Text,
    }
}

table! {
    locations (id) {
        id -> Integer,
        location_number -> Integer,
        collection_id -> Nullable<Integer>,
        user_id -> Nullable<Integer>,
        public -> Nullable<Integer>,
        location_name -> Nullable<Text>,
        latitude -> Nullable<Double>,
        longitude -> Nullable<Double>,
        notoriety_score -> Float,
        collection_path -> Nullable<Text>,
        collection_filename -> Nullable<Text>,
        collection_title -> Nullable<Text>,
        ignore_for_nearby_searches -> Integer,
        description -> Nullable<Text>,
    }
}

table! {
    plant_types (id) {
        id -> Integer,
        group_name -> Text,
        name -> Text,
        latin_name -> Nullable<Text>,
    }
}

table! {
    user_oauth_entries (id) {
        id -> Integer,
        user_id -> Integer,
        unique_id -> Text,
        oauth_info -> Nullable<Text>,
    }
}

table! {
    user_sessions (id) {
        id -> Integer,
        user_id -> Integer,
        session_value -> Text,
        created -> BigInt,
    }
}

table! {
    users (id) {
        id -> Integer,
        name -> Text,
        email -> Text,
        location_name -> Nullable<Text>,
        latitude -> Nullable<Double>,
        longitude -> Nullable<Double>,
    }
}

joinable!(collection_categories -> collections (collection_id));

allow_tables_to_appear_in_same_query!(
    base_plants,
    collection_categories,
    collection_items,
    collections,
    facts,
    locations,
    plant_types,
    user_oauth_entries,
    user_sessions,
    users,
);
