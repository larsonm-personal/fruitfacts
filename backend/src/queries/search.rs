use crate::session;

use super::super::schema_fts::*;
use super::super::schema_generated::*;
use super::super::schema_types::*;
use actix_web::{get, web, HttpRequest, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::fmt::Write as _; // for write!() macro

use anyhow::{anyhow, Result};

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct SearchQuery {
    #[serde(rename = "searchType")]
    pub search_type: Option<String>, // base plants or collection items. todo: user items, "search all"
    pub search: Option<String>, // search string like "PF 11"
    pub name: Option<String>, // exact plant name (allows getting a single base plant). probably doesn't make sense when used with search
    pub patents: Option<bool>,
    #[serde(rename = "type")]
    pub type_: Option<String>, // apple, peach, etc.
    pub page: Option<String>, // 1-referenced (default 1) 1-N or "mid" for the patent midpoint page if unknown (so our first patent page link can work)
    #[serde(rename = "perPage")]
    pub per_page: Option<i32>,
    #[serde(rename = "orderBy")]
    pub order_by: Option<String>, // sort options: notoriety, search quality, type then name, name then type, patent expiration (special case, also compute the middle patent page), harvest time
    pub order: Option<String>, // "asc" or "desc". desc if omitted
    #[serde(rename = "relativeHarvestMin")]
    pub relative_harvest_min: Option<i32>,
    #[serde(rename = "relativeHarvestMax")]
    pub relative_harvest_max: Option<i32>,

    // collection items search only (id or path, or I guess both)
    #[serde(rename = "collectionID")]
    pub collection_id: Option<String>,
    #[serde(rename = "collectionPath")]
    pub collection_path: Option<String>,

    // location items search only
    #[serde(rename = "locationID")]
    pub location_id: Option<String>,

    // for listing a user's locations
    pub user_id: Option<i32>,

    // base plants search only:
    #[serde(rename = "notorietyMin")]
    pub notoriety_min: Option<f32>,

    // todo:
    pub distance: Option<String>, // max distance, goes with "from"
    pub from: Option<String>, // goes with distance, either lat/lon like "45.687631,-122.824202" or zip code like "zip:97231"
}

// base plants search:
// 1. do name search if specified
// 2. do regular filter search: patents yes/no, type, relative harvest, notoriety, sort
// 3. intersection of these (either in database or out of database)
// 4. figure out pagination (total size, and pick a page to return)

// collection items search:
// 1. do name search if specified? or I could omit name filter on this for now
// 2. regular filter search: same as above? notoriety maybe doesn't make sense. big add is filter by collection yes/no
// 3. intersection?
// 4. figure out pagination (total size, and pick a page to return)
// 5. look up and return collection+location info, if specified

// how much overlap can be found between these two paths? and eventually user items search

// queries to bring under this:
// - plain variety search (against base plants) - base plants searchType, and search. limit to first N or whatever
// - fancy search and filter for an "advanced search" page, with pagination - same as above but no limit and paginate, sort by whatever, etc.
// - get a single variety (get a single base plant) - base plants searchType, name and type
// - get patents - base plants searchType, patents:true, probably sort by expiration
// - get paginated plants (browse plants) - base plants searchType, type=plant type, page set, etc.
// do this 2nd, it's the only non-base-plants one:
// - get a single collection - collection items searchType, and collection ID or path. would allow filtering or sorting from the collection page

#[skip_serializing_none]
#[derive(Default, Queryable, Serialize)]
pub struct SearchReturn {
    pub query: SearchQuery,

    pub count: Option<i64>, // total count of search results (if paginated)
    pub page: Option<i64>, // 1-referenced. we pass this out in case of a patent midpoint page and for completeness
    #[serde(rename = "lastPage")]
    pub last_page: Option<i64>, // 1-referenced
    #[serde(rename = "patentMidpointPage")]
    pub patent_midpoint_page: Option<i64>, // special case: if we did a patent search, which page has the transition from past to future expirations?
    #[serde(rename = "basePlants")]
    pub base_plants: Option<Vec<BasePlant>>,

    // only if constrained to one collection or location:
    pub collection: Option<Collection>,
    pub locations: Option<Vec<Location>>,
    #[serde(rename = "collectionItems")]
    pub collection_items: Option<Vec<CollectionItem>>,
}

#[derive(Debug, PartialEq)]
pub struct DistanceDegrees {
    pub lat: f64,
    pub lon: f64,
}

// given a string like "100", calculate a simple distance east/west and north/south
// for that point to create an X mile square on the map (which won't actually be square)
// the bounding box will be narrower near the poles (in terms of degrees) for a given distance
// this function is basically naive and should be replaced by something from spatialite or whatever
pub fn distance_km_to_degrees(distance: &str, start_latitude: f64) -> Option<DistanceDegrees> {
    // todo - google a formula for this. I think north/south will be fixed
    // and east/west will obviously depend on how far from the equator you are

    const EARTH_CIRCUMFERENCE_KM: u32 = 40075;

    let parsed = distance.parse::<f64>();
    if parsed.is_err() {
        return None;
    }
    let parsed = parsed.unwrap();

    // latitude is pretty constant in terms of km->degrees: have km, want degrees
    // km / 40075 km/circumference * 360*/circumference
    // longitude gets more narrow at the poles
    let lat: f64 = parsed * 360.0 / EARTH_CIRCUMFERENCE_KM as f64;

    let lon = if start_latitude.abs() >= 90.0 {
        0.0 // prevent divide by zero or other weirdness
    } else {
        // for 0 degrees starting latitude: lat * (90 - 0) / 90 = lat * 1
        // for 45 degrees: lat * (90 - 45) / 90 = lat * 45/90 = lat * 1/2 (half size at 45 degrees N/S)
        // for 80 degrees: lat * (90 - 80) / 90 = lat * 10/90 = lat * 1/9 (smaller near the poles)
        lat * (90.0 - start_latitude.abs()) / 90.0
    };

    Some(DistanceDegrees { lat, lon })
}

// given a set of location IDs, get all base plant IDs across all of them (de-duped)
// in order to use as a search filter (to only find plants mentioned within X miles of some spot)
fn get_base_plant_ids_from_locations(
    db_conn: &mut SqliteConnection,
    location_ids: Vec<i32>,
) -> Result<Vec<i32>, diesel::result::Error> {
    let base_ids = collection_items::dsl::collection_items
        .filter(collection_items::location_id.eq_any(location_ids))
        .select(collection_items::base_plant_id)
        .distinct()
        .load::<Option<i32>>(db_conn);

    if let Err(base_ids) = base_ids {
        return Err(base_ids);
    }
    let base_ids = base_ids.unwrap();

    let base_ids_no_none: Vec<i32> = base_ids.into_iter().flatten().collect(); // flatten() here removes the None entries

    Ok(base_ids_no_none)
}

// todo: I want to bring all of the search & filter queries into one API

pub fn search_db(
    session: &Option<UserSession>,
    query: &SearchQuery,
    db_conn: &mut SqliteConnection,
) -> Result<SearchReturn> {
    if query.search_type.is_none() {
        return Err(anyhow!("search_type not set"));
    }

    match query.search_type.as_ref().unwrap().as_str() {
        "base" => {
            // we make three queries because boxed diesel queries can't be copied, so we need to make running copies
            // so we can later use them for counts
            // see https://github.com/diesel-rs/diesel/issues/2277
            let mut base_query = base_plants::table.into_boxed();
            let mut pat_mid_q = base_plants::table.into_boxed(); // for patent count
            let mut total_count_q = base_plants::table.into_boxed(); // for overall count

            if let Some(input) = &query.search {
                // todo - support searching with just one or two letters (fts uses "trigraphs" and won't search for these)
                // just do a begins-with search on name or AKA columns

                // remove extra characters. leave spaces so we can treat separate words as separate
                // dashes get interpreted by fts. same with +*:^ AND OR NOT
                let re = Regex::new(r"[^0-9A-Za-z ]").unwrap();
                let cleaned = re.replace_all(input.trim(), ""); // also trim() for leading/trailing whitespace

                // replace multiple spaces with a single space
                let re = Regex::new(r"\s+").unwrap();
                let multiple_spaces_removed = re.replace_all(&cleaned, " ");

                let mut statement = multiple_spaces_removed.split(' ').collect::<Vec<&str>>();

                // if we have multiple search words, also add a search element which is all of them concatenated
                // allows searching for "pf 11" which would otherwise be two chars
                let split_for_count = multiple_spaces_removed.split(' ').collect::<Vec<&str>>();

                // look at the last element to see if one of our types starts with this - if so we'll restrict results to this type
                let mut restrict_to_type: Option<String> = None;
                if split_for_count.len() >= 2 {
                    for type_ in crate::import_db::types_generated::TYPES.iter() {
                        if *type_.to_lowercase() == split_for_count.last().unwrap().to_lowercase() {
                            restrict_to_type = Some(type_.to_string());
                            break;
                        }
                    }
                }

                if restrict_to_type.is_some() {
                    statement.pop(); // we got a type by matching on the last search element, remove it from the FTS search words
                }

                let mut statement_string = statement
                    .clone()
                    .into_iter()
                    .map(|x| format!("\"{x}\"")) // double quote each element - allows searching for special characters or keywords like "OR"
                    .collect::<Vec<String>>()
                    .join(" OR ");

                // if we have multiple words, try adding one last search term which is all of them concatenated
                // this helps us with "pf 1" for example
                if split_for_count.len() >= 2 {
                    let _ = write!(statement_string, " OR \"{}\"", statement.join(""));
                }

                println!("input {input} cleaned: {cleaned} ORed: {statement_string}");

                let search_query = fts_base_plants::table
                    .select(fts_base_plants::rowid)
                    .filter(fts_base_plants::whole_row.eq(statement_string))
                    .order(fts_base_plants::rank.asc())
                    .into_boxed();

                // todo - if we have restrict_to_type set, apply that (?) - or do we want to pull that logic out from the front end?

                // no limit within search - we may sort later, by something else

                let values = search_query.load::<i32>(db_conn);
                // println!("{:?}", values);

                // todo: order or limit by notoriety
                match values {
                    Ok(values) => {
                        if let Some(type_) = restrict_to_type {
                            base_query = base_query.filter(base_plants::type_.eq(type_.clone()));
                            pat_mid_q = pat_mid_q.filter(base_plants::type_.eq(type_.clone()));
                            total_count_q = total_count_q.filter(base_plants::type_.eq(type_));
                        };

                        let ids_only = values;

                        base_query = base_query.filter(base_plants::id.eq_any(ids_only.clone()));
                        pat_mid_q = pat_mid_q.filter(base_plants::id.eq_any(ids_only.clone()));
                        total_count_q = total_count_q.filter(base_plants::id.eq_any(ids_only));
                    }
                    Err(_error) => {
                        return Err(anyhow!("some kind of search problem (todo)"));
                    }
                }
            }

            if let Some(name) = &query.name {
                base_query = base_query.filter(base_plants::name.eq(name));
                pat_mid_q = pat_mid_q.filter(base_plants::name.eq(name));
                total_count_q = total_count_q.filter(base_plants::name.eq(name));
            }

            // patents: Option<bool>
            if let Some(patents) = &query.patents {
                if *patents {
                    base_query = base_query.filter(base_plants::uspp_expiration.is_not_null());
                    pat_mid_q = pat_mid_q.filter(base_plants::uspp_expiration.is_not_null());
                    total_count_q =
                        total_count_q.filter(base_plants::uspp_expiration.is_not_null());
                } else {
                    // finding items without patents is probably not useful but whatever
                    base_query = base_query.filter(base_plants::uspp_expiration.is_null());
                    pat_mid_q = pat_mid_q.filter(base_plants::uspp_expiration.is_null());
                    total_count_q = total_count_q.filter(base_plants::uspp_expiration.is_null());
                }
            }

            if let Some(type_) = &query.type_ {
                base_query = base_query.filter(base_plants::type_.eq(type_));
                pat_mid_q = pat_mid_q.filter(base_plants::type_.eq(type_));
                total_count_q = total_count_q.filter(base_plants::type_.eq(type_));
            }

            if let Some(sort) = &query.order_by {
                let order_asc;
                if let Some(order) = &query.order {
                    match order.as_str() {
                        "asc" => order_asc = true,
                        "desc" => order_asc = false,
                        _ => return Err(anyhow!("unknown order \"{order}\"")),
                    }
                } else {
                    // default sort order is different for each type
                    order_asc = match sort.as_str() {
                        "notoriety" => false, // highest notiriety first
                        "type_then_name" => false,
                        "name_then_type" => false,
                        "patent_expiration" => true, // oldest patents first
                        "harvest_time" => false,
                        "search_quality" => false, // highest search quality first
                        _ => false,
                    };
                }

                match sort.as_str() {
                    "notoriety" => {
                        if order_asc {
                            base_query = base_query.order(base_plants::notoriety_score.asc());
                            pat_mid_q = pat_mid_q.order(base_plants::notoriety_score.asc());
                            total_count_q = total_count_q.order(base_plants::notoriety_score.asc());
                        } else {
                            base_query = base_query.order(base_plants::notoriety_score.desc());
                            pat_mid_q = pat_mid_q.order(base_plants::notoriety_score.desc());
                            total_count_q =
                                total_count_q.order(base_plants::notoriety_score.desc());
                        }
                    }
                    "type_then_name" => {
                        if order_asc {
                            base_query = base_query.order(base_plants::type_.asc());
                            pat_mid_q = pat_mid_q.order(base_plants::type_.asc());
                            total_count_q = total_count_q.order(base_plants::type_.asc());

                            base_query = base_query.then_order_by(base_plants::name.asc());
                            pat_mid_q = pat_mid_q.then_order_by(base_plants::name.asc());
                            total_count_q = total_count_q.then_order_by(base_plants::name.asc());
                        } else {
                            base_query = base_query.order(base_plants::type_.desc());
                            pat_mid_q = pat_mid_q.order(base_plants::type_.desc());
                            total_count_q = total_count_q.order(base_plants::type_.desc());

                            base_query = base_query.then_order_by(base_plants::name.desc());
                            pat_mid_q = pat_mid_q.then_order_by(base_plants::name.desc());
                            total_count_q = total_count_q.then_order_by(base_plants::name.desc());
                        }
                    }
                    "name_then_type" => {
                        if order_asc {
                            base_query = base_query.order(base_plants::name.asc());
                            pat_mid_q = pat_mid_q.order(base_plants::name.asc());
                            total_count_q = total_count_q.order(base_plants::name.asc());

                            base_query = base_query.then_order_by(base_plants::type_.asc());
                            pat_mid_q = pat_mid_q.then_order_by(base_plants::type_.asc());
                            total_count_q = total_count_q.then_order_by(base_plants::type_.asc());
                        } else {
                            base_query = base_query.order(base_plants::name.desc());
                            pat_mid_q = pat_mid_q.order(base_plants::name.desc());
                            total_count_q = total_count_q.order(base_plants::name.desc());

                            base_query = base_query.then_order_by(base_plants::type_.desc());
                            pat_mid_q = pat_mid_q.then_order_by(base_plants::type_.desc());
                            total_count_q = total_count_q.then_order_by(base_plants::type_.desc());
                        }
                    }
                    "patent_expiration" => {
                        base_query = base_query.filter(base_plants::uspp_expiration.is_not_null());
                        pat_mid_q = pat_mid_q.filter(base_plants::uspp_expiration.is_not_null());
                        total_count_q =
                            total_count_q.filter(base_plants::uspp_expiration.is_not_null());

                        if order_asc {
                            base_query = base_query.order(base_plants::uspp_expiration.asc());
                            pat_mid_q = pat_mid_q.order(base_plants::uspp_expiration.asc());
                            total_count_q = total_count_q.order(base_plants::uspp_expiration.asc());
                        } else {
                            base_query = base_query.order(base_plants::uspp_expiration.desc());
                            pat_mid_q = pat_mid_q.order(base_plants::uspp_expiration.desc());
                            total_count_q =
                                total_count_q.order(base_plants::uspp_expiration.desc());
                        }
                    }
                    "harvest_time" => {
                        base_query = base_query.filter(base_plants::harvest_relative.is_not_null());
                        pat_mid_q = pat_mid_q.filter(base_plants::harvest_relative.is_not_null());
                        total_count_q =
                            total_count_q.filter(base_plants::harvest_relative.is_not_null());

                        if order_asc {
                            base_query = base_query.order(base_plants::harvest_relative.asc());
                            pat_mid_q = pat_mid_q.order(base_plants::harvest_relative.asc());
                            total_count_q =
                                total_count_q.order(base_plants::harvest_relative.asc());
                        } else {
                            base_query = base_query.order(base_plants::harvest_relative.desc());
                            pat_mid_q = pat_mid_q.order(base_plants::harvest_relative.desc());
                            total_count_q =
                                total_count_q.order(base_plants::harvest_relative.desc());
                        }
                    }
                    "search_quality" => {
                        // todo - this needs some fancy handling to carry through sort order in the query
                        // or otherwise sort the results after they're returned
                        return Err(anyhow!("search_quality sort type not implemented (todo)"));
                    }
                    _ => return Err(anyhow!("unknown sort type \"{sort}\"")),
                }
            }

            // relative_harvest (min and max)
            if let Some(relative_harvest_min) = &query.relative_harvest_min {
                base_query =
                    base_query.filter(base_plants::harvest_relative.ge(relative_harvest_min));
                pat_mid_q =
                    pat_mid_q.filter(base_plants::harvest_relative.ge(relative_harvest_min));
                total_count_q =
                    total_count_q.filter(base_plants::harvest_relative.ge(relative_harvest_min));
            }
            if let Some(relative_harvest_max) = &query.relative_harvest_max {
                base_query =
                    base_query.filter(base_plants::harvest_relative.le(relative_harvest_max));
                pat_mid_q =
                    pat_mid_q.filter(base_plants::harvest_relative.le(relative_harvest_max));
                total_count_q =
                    total_count_q.filter(base_plants::harvest_relative.le(relative_harvest_max));
            }

            // notoriety_min (base plants only)
            if let Some(notoriety_min) = &query.notoriety_min {
                base_query = base_query.filter(base_plants::notoriety_score.ge(notoriety_min));
                pat_mid_q = pat_mid_q.filter(base_plants::notoriety_score.ge(notoriety_min));
                total_count_q =
                    total_count_q.filter(base_plants::notoriety_score.ge(notoriety_min));
            }

            // todo distance, from (collection items only) - there may be value to a search filter for "mentioned within x miles of zip code"
            if (query.distance.is_some() && query.from.is_none())
                || (query.distance.is_none() && query.from.is_some())
            {
                return Err(anyhow!("got only one of \"distance\" and \"from\""));
            }

            if let Some(distance) = &query.distance {
                if let Some(from) = &query.from {
                    if let Ok(location) = crate::gazetteer_load::from_to_location(from) {
                        // todo - load all collection items that match, then filter for those within x miles
                        //    - but that won't be any different than a base items search?
                        // or - load all within x miles, then filter for matches?
                        // 1. mentioned in any reference within x miles
                        //    - possibly with a restriction to a type of reference or a minimum quality level
                        // 2. some sum of the references within x miles, and if it's above some total

                        // goal - do I want to return base plants, or some de-duped collection items thing?
                        // collection items would have the advantage of allowing a search for user items, I guess? todo later

                        // todo (when I have internet) - is there a "unique" filter to get sqlite to de-dupe for us?

                        let distance_parsed = distance_km_to_degrees(distance, location.lat);

                        if distance_parsed.is_none() {
                            return Err(anyhow!("couldn't parse \"distance\": {distance}"));
                        }
                        let distance = distance_parsed.unwrap();

                        // 1. find locations within x miles (same as the map search)
                        // todo - make the distance calc right, it's stubbed right now
                        let locations = super::map::locations_search_db(
                            db_conn,
                            &super::map::GetLocationsQuery {
                                min_lat: Some(location.lat - distance.lat),
                                max_lat: Some(location.lat + distance.lat),
                                min_lon: Some(location.lon - distance.lon),
                                max_lon: Some(location.lon + distance.lon),
                                limit: None,
                                filter_out_ignored_for_nearby_searches: Some(true),
                            },
                        );

                        if locations.is_err() {
                            return Err(anyhow!("error getting locations during search"));
                        }

                        let locations = locations.unwrap();

                        // then get an array of location IDs
                        let location_ids: Vec<i32> =
                            locations.into_iter().map(|entry| entry.id).collect();

                        // 2. load collection items matching those location IDs
                        //    - make this a free function too
                        // 3. de-dupe and then load base plants for each of those type/name pairs

                        let base_plant_ids =
                            get_base_plant_ids_from_locations(db_conn, location_ids);

                        if base_plant_ids.is_err() {
                            return Err(anyhow!("error getting base plants during search"));
                        }

                        let base_plant_ids = base_plant_ids.unwrap();

                        // todo - does .filter() replace the previous filter (from text search)?
                        base_query =
                            base_query.filter(base_plants::id.eq_any(base_plant_ids.clone()));
                        pat_mid_q =
                            pat_mid_q.filter(base_plants::id.eq_any(base_plant_ids.clone()));
                        total_count_q =
                            total_count_q.filter(base_plants::id.eq_any(base_plant_ids));
                    } else {
                        return Err(anyhow!("couldn't parse \"from\": {from}"));
                    }
                }
            }

            // stop using filters at this point - now we figure out the page and total count etc.

            // if we're sorting by patent expiration, figure out the patent midpoint page (will only exist if we have a per_page)
            // so we can show a link to it, or provide it directly if requested as "page=mid"

            let mut patent_midpoint_page = None;
            if let Some(sort) = &query.order_by {
                if sort == "patent_expiration" {
                    if let Some(per_page) = &query.per_page {
                        // special case - might also need to check that we're sorting by expiration

                        let now = chrono::Utc::now().timestamp(); // todo - make this a parameter?
                        pat_mid_q = pat_mid_q.filter(base_plants::uspp_expiration.lt(now));
                        let prior_patent_count_result = pat_mid_q.count().first::<i64>(db_conn);

                        if let Err(error) = prior_patent_count_result {
                            return Err(error.into());
                        }
                        let prior_patent_count = prior_patent_count_result.unwrap();

                        // if prior patents count is 50 and we have 50 per page, we want to show page 2 (1-referenced)
                        // if it's 51, show page 2
                        // if 49, show page 1
                        patent_midpoint_page = Some(1 + prior_patent_count / *per_page as i64);
                    }
                }
            }

            let mut page_output = None;
            if let Some(page) = &query.page {
                if let Some(per_page) = &query.per_page {
                    if page == "mid" {
                        if patent_midpoint_page.is_none() {
                            return Err(anyhow!("requested patent midpoint page but we didn't find one (maybe not sorting by patent_expiration?)"));
                        }

                        page_output = patent_midpoint_page;
                        base_query = base_query
                            .offset((patent_midpoint_page.unwrap() - 1) * *per_page as i64);
                    } else {
                        let page_i32_result = page.parse::<i32>();
                        if page_i32_result.is_err() {
                            return Err(anyhow!("\"page\" wasn't \"mid\" or an integer"));
                        }
                        let page_i32 = page_i32_result.unwrap();
                        page_output = Some(page_i32.into());

                        base_query = base_query.offset(((page_i32 - 1) * per_page).into());
                    }
                } else {
                    // special case: allow page=1 without per_page
                    if page == "1" {
                        page_output = Some(1);
                        // no offset - nothing added to the query
                    } else {
                        return Err(anyhow!("got \"page\" != \"1\" without \"per_page\""));
                    }
                }
            }

            let overall_count_result = total_count_q.count().first::<i64>(db_conn);
            if let Err(error) = overall_count_result {
                return Err(error.into());
            }
            let overall_count = overall_count_result.unwrap();

            let last_page: i64 = if let Some(per_page) = &query.per_page {
                (overall_count + (*per_page as i64 - 1)) / *per_page as i64 // round up
            } else {
                1
            };

            // todo - etc. - and we can have a path to omit this and get it from the items count if we have no limit, I guess

            // todo midpoint if getting patents, and sorted by expiration date

            // per_page can be used as a limit if page isn't specified
            if let Some(per_page) = &query.per_page {
                base_query = base_query.limit(*per_page as i64);
            }

            let base_plants: Result<Vec<BasePlant>, diesel::result::Error> =
                base_query.load(db_conn);

            match base_plants {
                Ok(base_plants) => Ok(SearchReturn {
                    query: query.clone(),
                    base_plants: Some(base_plants),
                    count: Some(overall_count),
                    page: page_output,
                    last_page: Some(last_page),
                    patent_midpoint_page,
                    ..Default::default()
                }),
                Err(error) => Err(error.into()),
            }
        }
        "coll" => {
            /*
            // todo - for collection items search only
            // todo collection (path or ID) - collection items only
            if let Some(collection_id) = &query.collection_id {
                // todo - limit to this collection ID (different search type though)
            }
            if let Some(collection_path) = &query.collection_path {
                // todo
            }
            */
            Err(anyhow!("collection search not implemented"))
        }
        "loc" => {
            // list items in a single location (users define single locations instead of collections)
            // check that this is either our location (matches session's user id) or public
            Err(anyhow!("location search not implemented"))
        }
        "user_loc" => {
            // list locations by user ID - similar to listing a collection (all locations with a collection ID)
            // todo - for each location, show a count of items in that location

            if query.user_id.is_none() {
                return Err(anyhow!("user_loc search without user_id specified"));
            }
            let query_user_id = query.user_id.unwrap();

            // given a user ID, we can either get it with a matching session user ID, or fall back to getting only public=true
            if let Some(session) = session {
                if session.user_id == query_user_id {
                    // getting our own sessions - no need for public=true

                    let locations = locations::dsl::locations
                        .filter(locations::user_id.eq(query_user_id))
                        .load::<Location>(db_conn);

                    return match locations {
                        Ok(locations) => Ok(SearchReturn {
                            query: query.clone(),
                            locations: Some(locations),
                            ..Default::default()
                        }),
                        Err(error) => Err(error.into()),
                    };
                }
            }

            // drop here either from no session or from session not matching
            // getting another user's locations - set public=true, but otherwise we can ask for any user ID
            let locations = locations::dsl::locations
                .filter(locations::user_id.eq(query_user_id))
                .filter(locations::public.eq(1))
                .load::<Location>(db_conn);

            match locations {
                Ok(locations) => Ok(SearchReturn {
                    query: query.clone(),
                    locations: Some(locations),
                    ..Default::default()
                }),
                Err(error) => Err(error.into()),
            }
        }
        _ => Err(anyhow!("unknown search type")),
    }
}

// search test cases:
// "red" -> "redhaven" "early redhaven" ...
// "pf 11" -> should find pf 11 peach, treating this as two words wouldn't find it because of fts searching based on trigraphs
// "pf 1" -> pf 1 peach
// "pf-11" -> pf 11 peach
// "pf 1 peach" -> pf 1 peach (multiple search terms, short, etc.)
// "liberty apple" -> should be an exact match, and not return "dapple dandy" (contains the word apple)
//    may also suggest the apple page
// "liberty peach" -> should be an exact match, not finding "burpeachwhatever"
// "delight cherry plum"
// "delight plum"
// "valor plum"

// todo: if we have an exact match for a type name (or type aka name) then remove that word, use it to suggest that type
// todo - this kind of type search plus a full text search on the collections json files
#[get("/api/search")]
async fn variety_search(
    req: HttpRequest,
    query: web::Query<SearchQuery>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let session: Option<UserSession>;
    let (session_value, _outgoing_cookie) = crate::queries::auth::get_session_value(req, false);
    if let Some(session_value) = session_value {
        let get_session_result = session::get_session(&mut conn, session_value);
        if let Ok(get_session_result) = get_session_result {
            session = Some(get_session_result);
        } else {
            session = None;
        }
    } else {
        session = None;
    }

    let results = web::block(move || search_db(&session, &query, &mut conn))
        .await
        .unwrap(); // todo - blockingerror unwrap?

    let results = match results {
        Ok(results) => results,
        Err(e) => {
            eprintln!("{}", e);
            return Err(actix_web::error::ErrorInternalServerError(""));
        }
    };

    Ok(HttpResponse::Ok().json(results))
}
