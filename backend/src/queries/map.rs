use super::super::schema_generated::*;
use super::super::schema_types::*;
use actix_web::{get, web, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
use serde::Deserialize;

const MAP_LOCATION_LIMIT: i32 = 5000;

// clamp latitude between -180 and +180
// this is a common function, see google results for an explanation
pub fn latitude_normalize(latitude: f64) -> f64 {
    let remainder = (latitude + 180.0) % 360.0;

    if remainder < 0.0 {
        remainder + 360.0 - 180.0
    } else {
        remainder - 180.0
    }
}

pub(crate) fn locations_query_limit(limit: Option<i32>) -> i32 {
    limit
        .unwrap_or(MAP_LOCATION_LIMIT)
        .clamp(1, MAP_LOCATION_LIMIT)
}

#[derive(Deserialize)]
pub struct GetLocationsQuery {
    pub min_lat: Option<f64>,
    pub max_lat: Option<f64>,
    pub min_lon: Option<f64>,
    pub max_lon: Option<f64>,
    pub limit: Option<i32>,
    pub filter_out_ignored_for_nearby_searches: Option<bool>,
}

// search for locations within a bounding box
// todo: limit results to categories like user locations, extension guides, u-picks, etc.
#[get("/api/locations")]
async fn locations_search(
    query: web::Query<GetLocationsQuery>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let results = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");
        locations_search_db(&mut conn, &query)
    })
    .await
    .unwrap(); // todo - blockingerror unwrap?

    let results = match results {
        Ok(results) => results,
        Err(e) => {
            eprintln!("{}", e);
            return Err(actix_web::error::ErrorInternalServerError(e));
        }
    };

    Ok(HttpResponse::Ok().json(results))
}

// we should be using spatialite but rust-diesel has no way to load modules currently
// so, we use a dumb bounding box instead

pub fn locations_search_db(
    db_conn: &mut SqliteConnection,
    query: &GetLocationsQuery,
) -> Result<Vec<Location>, diesel::result::Error> {
    // see https://stackoverflow.com/questions/15584000/how-to-search-predefined-locations-latitude-longitude-within-a-rectangular

    let mut db_query = locations::dsl::locations
        // todo filter etc.
        .into_boxed();

    if query.min_lat.is_some()
        && query.max_lat.is_some()
        && query.min_lon.is_some()
        && query.max_lon.is_some()
    {
        let min_lat = latitude_normalize(query.min_lat.unwrap());
        let max_lat = latitude_normalize(query.max_lat.unwrap());
        let min_lon = query.min_lon.unwrap();
        let max_lon = query.max_lon.unwrap();

        // example extents centered on oregon
        // min_lon/left: -124.98151399125442, min_lat/bottom: 41.51288497979664 (lower left)
        // max_lon/right: -116.2177107239959,  max_lat/top: 47.096207516124394 (upper right)

        // example extents centered on kiribati (crossing the 180* meridian and the equator)
        // min_lon/left: -193.94874081126167 (normalized to +166.05...), min_lat/bottom: -7.7353403623466335 (lower left)
        // max_lon/right: -175.0567314859299, max_lat/top: 9.041095775913107 (upper right)

        if min_lon < max_lon {
            // no wrapping - simple single box, greater than AND less than
            db_query = db_query.filter(locations::longitude.gt(min_lon));
            db_query = db_query.filter(locations::longitude.lt(max_lon));
        } else {
            // longitude wraps because it crosses the 180* meridian - we've normalized these
            // so we can flip to greater than OR less than
            db_query = db_query.filter(locations::longitude.gt(min_lon));
            db_query = db_query.or_filter(locations::longitude.lt(max_lon));
            // or_filter will get us "A OR B" with parenthesis, and we can continue with other filters
        }
        // these shouldn't wrap or be out of order. at least with the front end map library we're using
        db_query = db_query.filter(locations::latitude.gt(min_lat));
        db_query = db_query.filter(locations::latitude.lt(max_lat));
    }

    if let Some(value) = query.filter_out_ignored_for_nearby_searches {
        if value {
            db_query = db_query.filter(locations::ignore_for_nearby_searches.eq(0));
        }
    }

    db_query = db_query.order(locations::notoriety_score.desc());
    db_query = db_query.then_order_by(locations::location_name.asc());

    // todo filter for only extension pubs, u-picks, etc.

    let limit = locations_query_limit(query.limit);

    db_query = db_query.limit(limit as i64);

    db_query.load::<Location>(db_conn)
}
