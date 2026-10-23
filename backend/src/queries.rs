#[cfg(test)]
mod test;

pub mod auth;
pub mod list;
pub mod map;
pub mod search;
pub mod util;
use super::schema_generated::*;
use super::schema_types::*;
use actix_web::{get, web, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Queryable, Debug, Serialize)]
pub struct CollectionsForPaths {
    pub id: i32,

    pub path: String,
    pub filename: String,

    pub title: Option<String>,
}

#[derive(Default, Serialize)]
pub struct CollectionsReturn {
    directories: Vec<String>,
    collections: Vec<CollectionsForPaths>,
}

pub fn get_collections_db(
    db_conn: &mut SqliteConnection,
    path: &str,
) -> Result<CollectionsReturn, diesel::result::Error> {
    let path_decoded = util::path_to_name(path);

    let db_return = collections::dsl::collections
        .select((
            collections::id,
            collections::path,
            collections::filename,
            collections::title,
        ))
        .filter(collections::path.like(path_decoded.to_owned() + r#"%"#))
        .order(collections::path.asc())
        .load::<CollectionsForPaths>(db_conn);

    match db_return {
        Ok(collections) => {
            let mut output: CollectionsReturn = Default::default();
            let mut directories: HashSet<String> = Default::default();
            for collection in collections {
                if collection.path == path_decoded {
                    output.collections.push(collection);
                } else {
                    // remove multi-level subdirectories (more than one '/' after our search directory)
                    let trimmed =
                        crate::import_db::rem_first_n(&collection.path, path_decoded.len());
                    if trimmed.matches('/').count() == 1 {
                        directories.insert(collection.path); // this is a hashset so we'll get paths de-duplicated here
                    } else {
                        // println!("excluding subdir {}", collection_path)
                    }
                }
            }

            let mut directories_vector = Vec::from_iter(directories);
            directories_vector.sort();
            output.directories = directories_vector;

            Ok(output)
        }
        Err(error) => Err(error),
    }
}

#[derive(Default, Serialize)]
pub struct CollectionReturn {
    collection: Option<Collection>,
    locations: Vec<Location>,
    items: Vec<CollectionItem>,
}

pub fn get_collection_db(
    db_conn: &mut SqliteConnection,
    path: &str,
) -> Result<CollectionReturn, diesel::result::Error> {
    println!("{}", path);

    // this could be done with rfind('/') or similar to get rid of the regex
    // todo: at least limit the length of the incoming text to protect the regex
    let slash_regex = Regex::new(r#"(.*)/(.*)"#).unwrap();

    let mut dir: String = Default::default();
    let mut file: String = Default::default();
    if let Some(matches) = slash_regex.captures(path) {
        if matches.len() >= 3 {
            if let Some(dir_match) = matches.get(1) {
                dir = dir_match.as_str().to_string();
            }
            if let Some(file_match) = matches.get(2) {
                file = file_match.as_str().to_string();
            }
        }
    } else {
        file = path.to_string();
    }
    dir.push('/');

    println!("{:#?} {:#?}", dir, file);

    let collection: Result<Collection, diesel::result::Error> = collections::dsl::collections
        .filter(collections::path.eq(dir))
        .filter(collections::filename.eq(file))
        .order(collections::id)
        .first(db_conn);

    match collection {
        Ok(collection) => {
            let mut output: CollectionReturn = Default::default();

            let locations = Location::belonging_to(&collection)
                .load::<Location>(db_conn)
                .expect("Error loading locations");

            let items = CollectionItem::belonging_to(&collection)
                .load::<CollectionItem>(db_conn)
                .expect("Error loading items");

            output.collection = Some(collection);
            output.locations = locations;
            output.items = items;
            Ok(output)
        }
        Err(error) => Err(error),
    }
}

#[derive(Deserialize)]
struct Path {
    path: String,
}

// /collections/path/ - get subdirectories starting at this path, and collection names at this path
// /collections/path/collection - get a single collection
// todo - move this under the search API
#[get("/api/collections/{path:.*}")] // the ":.*" part is a regex to get the entire tail of the path
async fn get_collections(
    path: web::Path<Path>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, actix_web::Error> {
    if path.path.is_empty() || path.path.ends_with('/') {
        // get all subdirectories and all collections at this path
        let collections = web::block(move || {
            let mut conn = pool.get().expect("couldn't get db connection from pool");
            get_collections_db(&mut conn, &path.path)
        })
        .await
        .unwrap(); // todo - blockingerror unwrap?

        let collections = match collections {
            Ok(collections) => collections,
            Err(e) => {
                eprintln!("{}", e);
                return Err(actix_web::error::ErrorInternalServerError(""));
            }
        };

        Ok(HttpResponse::Ok().json(collections))
    } else {
        // doesn't end in '/': get an individual collection

        let collection = web::block(move || {
            let mut conn = pool.get().expect("couldn't get db connection from pool");
            get_collection_db(&mut conn, &path.path)
        })
        .await
        .unwrap(); // todo - blockingerror unwrap?

        let collection = match collection {
            Ok(collection) => collection,
            Err(e) => {
                eprintln!("{}", e);
                return Err(actix_web::error::ErrorInternalServerError(""));
            }
        };

        Ok(HttpResponse::Ok().json(collection))
    }
}

#[derive(Default, Serialize)]
pub struct PlantsReturn {
    pub plants: Vec<BasePlant>,
    pub per_page: i32,
    pub count: i64,
    pub last_page: i64,
}

pub fn get_plants_db(
    db_conn: &mut SqliteConnection,
    type_: &str,
    page: Option<i32>,
) -> Result<PlantsReturn, diesel::result::Error> {
    const PER_PAGE: i32 = 50;

    let type_decoded: String = util::path_to_name(type_);

    let mut query_for_items = base_plants::table
        .filter(base_plants::type_.eq(type_decoded.clone()))
        .limit(PER_PAGE.into())
        .into_boxed();

    let query_for_count = base_plants::table
        .filter(base_plants::type_.eq(type_decoded.clone()))
        .into_boxed();

    if let Some(page) = page {
        query_for_items = query_for_items.offset((page * PER_PAGE).into());
    }

    // todo - apply other things: order by, asc/desc, etc.

    let plants: Result<Vec<BasePlant>, diesel::result::Error> = query_for_items.load(db_conn);
    let count = query_for_count.count().first::<i64>(db_conn).unwrap();

    println!("get plants: {} page {:?}", type_decoded, page);

    match plants {
        Ok(plants) => {
            let mut last_page =
                (count / i64::from(PER_PAGE)) + i64::from((count % i64::from(PER_PAGE)) != 0) - 1; // -1: pages are 0-referenced
            if last_page < 0 {
                last_page = 0;
            }

            Ok(PlantsReturn {
                plants,
                per_page: PER_PAGE,
                count,
                last_page,
            })
        }
        Err(error) => Err(error),
    }
}

#[derive(Default, Serialize)]
pub struct PlantReturn {
    base: Option<BasePlant>,
    collection: Vec<CollectionItem>,
}

pub fn get_plant_db(
    db_conn: &mut SqliteConnection,
    type_: &str,
    name: &str,
) -> Result<PlantReturn, diesel::result::Error> {
    let type_decoded: String = util::path_to_name(type_);
    let name_decoded: String = util::path_to_name(name);

    let plant: Result<BasePlant, diesel::result::Error> = base_plants::dsl::base_plants
        .filter(base_plants::type_.eq(type_decoded.clone()))
        .filter(base_plants::name.eq(name_decoded.clone()))
        .first(db_conn);

    println!("get plant: {} {}", type_decoded, name_decoded);

    match plant {
        Ok(plant) => {
            let collection_plants: Result<Vec<CollectionItem>, diesel::result::Error> =
                collection_items::dsl::collection_items
                    .filter(collection_items::type_.eq(type_decoded))
                    .filter(collection_items::name.eq(name_decoded))
                    .load(db_conn);

            // todo: limit number to maybe 10, ordered by significance, and return the total number if we were limited so we can show "see all"

            match collection_plants {
                Ok(collection_plants) => {
                    let output = PlantReturn {
                        base: Some(plant),
                        collection: collection_plants,
                    };
                    Ok(output)
                }
                Err(error) => Err(error),
            }
        }
        Err(error) => Err(error),
    }
}

#[derive(Deserialize)]
struct GetPlantPath {
    type_: String,
    plant: String,
}

#[derive(Deserialize)]
struct GetPlantQuery {
    page: Option<i32>,
}

// /plants/type/ - all plants of this type. paginated?
// /plants/type/plant name - this specific plant
#[get("/api/plants/{type_}/{plant:.*}")] // the ":.*" part is a regex to get the entire tail of the path
async fn get_plant(
    path: web::Path<GetPlantPath>,
    query: web::Query<GetPlantQuery>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, actix_web::Error> {
    println!("/plant/ {} page {:?}", path.type_, query.page);
    if path.plant.is_empty() {
        let plants = web::block(move || {
            let mut conn = pool.get().expect("couldn't get db connection from pool");
            get_plants_db(&mut conn, &path.type_, query.page)
        })
        .await
        .unwrap(); // todo - blockingerror unwrap?

        let plants = match plants {
            Ok(plants) => plants,
            Err(e) => {
                eprintln!("{}", e);
                return Err(actix_web::error::ErrorInternalServerError(""));
            }
        };

        Ok(HttpResponse::Ok().json(plants))
    } else {
        // one plant

        let plant = web::block(move || {
            let mut conn = pool.get().expect("couldn't get db connection from pool");
            get_plant_db(&mut conn, &path.type_, &path.plant)
        })
        .await
        .unwrap(); // todo - blockingerror unwrap?

        let plant = match plant {
            Ok(plant) => plant,
            Err(e) => {
                eprintln!("{}", e);
                return Err(actix_web::error::ErrorInternalServerError(""));
            }
        };

        Ok(HttpResponse::Ok().json(plant))
    }
}

#[derive(Serialize)]
struct RecentChanges {
    #[serde(rename = "buildInfo")]
    build_info: BuildInfo,
    #[serde(rename = "recentChanges")]
    recent_changes: RecentChangesDB,
}

#[derive(Serialize)]
struct BuildInfo {
    #[serde(rename = "gitHash")]
    git_hash: String,
    #[serde(rename = "gitUnixTime")]
    git_unix_time: String,
    #[serde(rename = "gitCommitCount")]
    git_commit_count: String,
}

#[derive(Default, Queryable, Debug, Serialize)]
pub struct CollectionChanges {
    pub id: i32,

    pub path: String,
    pub filename: String,

    pub title: Option<String>,

    #[serde(rename = "gitEditTime")]
    pub git_edit_time: Option<i64>,
}

#[derive(Default, Serialize)]
pub struct RecentChangesDB {
    #[serde(rename = "collectionChanges")]
    pub collection_changes: Vec<CollectionChanges>,
    #[serde(rename = "basePlantsCount")]
    pub base_plants_count: i64,
    #[serde(rename = "referencesCount")]
    pub references_count: i64,
}

pub fn get_recent_changes_db(db_conn: &mut SqliteConnection) -> Result<RecentChangesDB> {
    let mut output: RecentChangesDB = Default::default();

    let db_return = collections::dsl::collections
        .select((
            collections::id,
            collections::path,
            collections::filename,
            collections::title,
            collections::git_edit_time,
        ))
        .order(collections::git_edit_time.desc())
        .limit(10)
        .load::<CollectionChanges>(db_conn);

    match db_return {
        Ok(collections) => {
            output.collection_changes = collections;
        }
        Err(error) => {
            return Err(error.into());
        }
    }

    let collections_count = collections::dsl::collections.count().first::<i64>(db_conn);
    if let Ok(count) = collections_count {
        output.references_count = count;
    }

    let base_plants_count = base_plants::dsl::base_plants.count().first::<i64>(db_conn);
    if let Ok(count) = base_plants_count {
        output.base_plants_count = count;
    }

    Ok(output)
}

#[get("/api/recent_changes")]
async fn get_recent_changes(pool: web::Data<DbPool>) -> Result<HttpResponse, actix_web::Error> {
    let changes_db = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");
        get_recent_changes_db(&mut conn)
    })
    .await
    .unwrap(); // todo - blockingerror unwrap?

    let changes_db = match changes_db {
        Ok(changes_db) => changes_db,
        Err(e) => {
            eprintln!("{}", e);
            return Err(actix_web::error::ErrorInternalServerError(""));
        }
    };

    Ok(HttpResponse::Ok().json(RecentChanges {
        build_info: BuildInfo {
            git_hash: env!("GIT_HASH").to_string(),
            git_unix_time: env!("GIT_UNIX_TIME").to_string(),
            git_commit_count: env!("GIT_MAIN_COMMIT_COUNT").to_string(),
        },
        recent_changes: changes_db,
    }))
}

diesel::define_sql_function! {
    fn random();
}

pub fn get_fact_db(db_conn: &mut SqliteConnection) -> Result<Fact, diesel::result::Error> {
    facts::dsl::facts
        .order(random())
        .limit(1)
        .first::<Fact>(db_conn)
}

#[get("/api/fact")]
async fn get_fact(pool: web::Data<DbPool>) -> Result<HttpResponse, actix_web::Error> {
    let fact = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");
        get_fact_db(&mut conn)
    })
    .await
    .unwrap(); // todo - blockingerror unwrap?

    let fact = match fact {
        Ok(fact) => fact,
        Err(e) => {
            eprintln!("{}", e);
            return Err(actix_web::error::ErrorInternalServerError(""));
        }
    };

    Ok(HttpResponse::Ok().json(fact))
}
