use super::search::{distance_km_to_degrees, DistanceDegrees};
use crate::queries::map::{latitude_normalize, locations_query_limit};

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
