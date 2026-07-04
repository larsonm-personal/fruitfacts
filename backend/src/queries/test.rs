use super::collection_path_parts;
use super::get_reference_category_db;
use super::search::{distance_km_to_degrees, DistanceDegrees};
use crate::queries::map::{latitude_normalize, locations_query_limit};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel_migrations::MigrationHarness;

#[test]
fn test_latitude_normalize() {
    assert_eq!(latitude_normalize(0.0), 0.0);
    assert_eq!(latitude_normalize(180.0), -180.0);
    assert_eq!(latitude_normalize(-180.0), -180.0);
    assert_eq!(latitude_normalize(359.0), -1.0);
    assert_eq!(latitude_normalize(360.0), 0.0);
    assert_eq!(latitude_normalize(361.0), 1.0);
    assert_eq!(latitude_normalize(1000.0), -80.0);
    assert_eq!(latitude_normalize(-1000.0), 80.0);
    assert_eq!(latitude_normalize(-124.98151399125442), -124.98151399125442);
    assert_eq!(latitude_normalize(-116.2177107239959), -116.2177107239959);
    assert_eq!(latitude_normalize(-193.94874081126167), 166.0512591887383);
    assert_eq!(latitude_normalize(-175.0567314859299), -175.0567314859299);
}

#[test]
fn test_locations_query_limit() {
    assert_eq!(locations_query_limit(None), 5000);
    assert_eq!(locations_query_limit(Some(0)), 1);
    assert_eq!(locations_query_limit(Some(123)), 123);
    assert_eq!(locations_query_limit(Some(10_000)), 5000);
}

#[test]
fn test_distance_km_to_degrees() {
    // todo, once the formula is put in and we handle mi and km
    assert_eq!(distance_km_to_degrees("can't parse", 0.0), None);
    assert_eq!(
        distance_km_to_degrees("25", 0.0),
        Some(DistanceDegrees {
            lat: 0.22457891453524642,
            lon: 0.22457891453524642
        })
    );
    assert_eq!(
        distance_km_to_degrees("25", 45.0),
        Some(DistanceDegrees {
            lat: 0.22457891453524642,
            lon: 0.11228945726762321
        })
    );
    assert_eq!(
        distance_km_to_degrees("250.0", 90.0),
        Some(DistanceDegrees {
            lat: 2.245789145352464,
            lon: 0.0
        })
    );
}

#[test]
fn test_collection_path_parts_decodes_collection_slug() {
    assert_eq!(
        collection_path_parts(
            "Massachusetts/Fruit_Notes-_Selected_Minnesota_Cool_Climate_Red_Grape_Varieties_for_the_Northeast"
        ),
        (
            "Massachusetts/".to_string(),
            "Fruit Notes- Selected Minnesota Cool Climate Red Grape Varieties for the Northeast"
                .to_string()
        )
    );
}

#[test]
fn test_collection_path_parts_preserves_decoded_collection_path() {
    assert_eq!(
        collection_path_parts(
            "Massachusetts/Fruit Notes- Selected Minnesota Cool Climate Red Grape Varieties for the Northeast"
        ),
        (
            "Massachusetts/".to_string(),
            "Fruit Notes- Selected Minnesota Cool Climate Red Grape Varieties for the Northeast"
                .to_string()
        )
    );
}

#[test]
fn test_get_reference_category_db_returns_collections_and_labels() {
    let mut db_conn = SqliteConnection::establish(":memory:").unwrap();
    db_conn.run_pending_migrations(crate::MIGRATIONS).unwrap();

    db_conn
        .batch_execute(
        r#"
        INSERT INTO collections (
            id, path, filename, notoriety_type, notoriety_score,
            notoriety_score_explanation, ignore_for_nearby_searches, title, needs_help
        ) VALUES
            (1, 'Massachusetts/', 'Fruit Notes- A', 'journal article test', 10.0, 'test', 0, 'A', 0),
            (2, 'Massachusetts/', 'Fruit Notes- B', 'journal article test', 10.0, 'test', 0, 'B', 0),
            (3, 'Oregon/', 'Other Source', 'extension publication', 10.0, 'test', 0, 'Other', 0);

        INSERT INTO collection_categories (collection_id, category) VALUES
            (1, 'fruit-notes'),
            (1, 'fruit-notes-v85n2-spring-2020'),
            (2, 'fruit-notes'),
            (2, 'fruit-notes-v83n4-fall-2018'),
            (3, 'other-series');
        "#,
    )
    .unwrap();

    let output = get_reference_category_db(&mut db_conn, "fruit-notes").unwrap();

    assert_eq!(output.collections.len(), 2);
    assert_eq!(output.collections[0].collection.filename, "Fruit Notes- A");
    assert_eq!(
        output.collections[0].categories,
        vec![
            "fruit-notes".to_string(),
            "fruit-notes-v85n2-spring-2020".to_string()
        ]
    );
}
