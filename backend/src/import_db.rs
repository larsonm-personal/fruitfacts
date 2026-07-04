// import_db.rs: ETL for a set of json files that comprise all of the built-in reference plants, bringing them into the database
// this lets the database's files be viewed on github and edited by hand as text files,
// allowing a wider audience of contributors and easier database maintenance
// the ETL stuff gets complex in a few places. the goal is to allow the json references to be as simple and close to plain
// copy-paste imports as possible, with this file doing things like parsing a wide range of date formats
// and then doing follow-on data analysis like calculating relative harvest times a few different ways

#[cfg(test)]
mod test;

mod notoriety;
pub mod types_generated;
mod util;

use crate::git_info::GitModificationTimes;

use super::schema_generated::base_plants;
use super::schema_generated::collection_categories;
use super::schema_generated::collection_items;
use super::schema_generated::collections;
use super::schema_generated::facts;
use super::schema_generated::locations;
use super::schema_generated::plant_types;
use super::schema_types::*;
use crate::diesel_migrations::MigrationHarness;
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use itertools::Itertools;

use indexmap::IndexMap;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::fs;
use std::time::{Duration, Instant};
use walkdir::WalkDir;

extern crate pathdiff;
extern crate regex;
use regex::Regex;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
//use serde_json::Result;

#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize)]
struct BasePlantJson {
    name: String,
    #[serde(rename = "type")]
    type_: Option<String>, // optional because we can get type from the filename
    description: Option<String>,
    #[serde(rename = "AKA")]
    aka: Option<Vec<String>>,
    patent: Option<String>,
    released: Option<String>,
    s_allele: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
struct CollectionJson {
    title: String,
    author: Option<String>,
    description: Option<String>,
    url: Option<String>,
    published: Option<String>,
    reviewed: Option<String>,
    accessed: Option<String>,
    needs_help: Option<bool>,
    #[serde(rename = "type")] // notoriety type like "extension publication"
    type_: String,
    harvest_time_devalue_factor: Option<f32>, // an option to reduce the weight of harvest times because of an editorial decision that they're low quality
    ignore_unless_in_others: Option<bool>, // option to skip creating entries based on this file unless they're also in another file
    ignore_for_nearby_searches: Option<bool>, // if this has an arbitrary location like Washington, DC for the us patent list, ignore it so it doesn't appear "nearby" to users

    reference_categories: Option<Vec<String>>,
    locations: Vec<CollectionLocationJson>,
    categories: Option<Vec<CollectionCategoryJson>>,
    plants: Vec<CollectionPlantJson>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
struct CollectionLocationJson {
    short_name: Option<String>,
    name: String,
    latitude: f64,
    longitude: f64,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
struct CollectionCategoryJson {
    name: String,
    description: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
struct CollectionPlantJson {
    // only for lists of names like we see in some guides for "here's a list of scab-resistant apples"
    names: Option<Vec<String>>,
    category: Option<String>,
    category_description: Option<String>,

    name: Option<String>, // optional because we can get "names" or "name"
    #[serde(rename = "type")]
    type_: String,

    locations: Option<Vec<serde_json::Value>>, // this will be either a bare string or CollectionPlantLocationsJson if it includes ripening times

    // type 1: each element is a string
    //     "locations": ["I", "II", "III", "IV"],

    // type 2: each element is an object
    //     "locations": [
    //         {"San Joaquin Valley": "Oct-Nov"},
    //         {"Sacramento Valley": "late Oct-Nov"},
    //         ...
    //     ]
    description: Option<String>,
    harvest_time: Option<String>,
    harvest_time_relative: Option<String>,
    harvest_time_unparsed: Option<String>,
    disease_resistance: Option<HashMap<String, String>>,
    chill: Option<String>,
    s_allele: Option<String>,

    // top-level fields that may be lifted into base plants if they aren't already set
    #[serde(rename = "AKA")]
    aka: Option<Vec<String>>,
    patent: Option<String>,
    released: Option<String>,
}

pub fn rem_last_n(value: &str, n: usize) -> &str {
    let mut chars = value.chars();
    for _ in 0..n {
        chars.next_back();
    }
    chars.as_str()
}

pub fn rem_first_n(value: &str, n: usize) -> &str {
    let mut chars = value.chars();
    for _ in 0..n {
        chars.next();
    }
    chars.as_str()
}

#[derive(Debug, PartialEq, Eq, Default)]
enum DateParseType {
    #[default]
    Undefined,
    Unparsed,
    StartOnly,
    TwoDates,
}

#[derive(Default, Debug, PartialEq, Eq)]
struct DayRangeOutput {
    parse_type: DateParseType,
    start: Option<u32>,
    end: Option<u32>,
}

#[derive(Debug, PartialEq, Eq)]
enum MonthLocationType {
    NoMonth,
    MonthAtBeginning,
    MonthAtEnd,
}

fn month_location(input: &str) -> MonthLocationType {
    let month_at_beginning_regex =
        Regex::new(r#"^(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)"#).unwrap();

    match month_at_beginning_regex.captures(&input.to_lowercase()) {
        Some(_) => MonthLocationType::MonthAtBeginning,
        None => {
            let month_at_end_regex =
                Regex::new(r#"(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)[^0-9]*$"#).unwrap();

            match month_at_end_regex.captures(&input.to_lowercase()) {
                Some(_) => MonthLocationType::MonthAtEnd,
                None => MonthLocationType::NoMonth,
            }
        }
    }
}

fn get_month(input: &str) -> String {
    let month_names_regex =
        Regex::new(r#"(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)"#).unwrap();

    match month_names_regex.captures(&input.to_lowercase()) {
        Some(matches) => matches[1].to_owned(),
        None => panic!("no month found in string {}", input),
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
struct ReleasedOutput {
    releaser: Option<String>,
    year: Option<i32>,
    authoritative: bool,
}

fn parse_released(input: &str) -> Option<ReleasedOutput> {
    if input.is_empty() {
        return None;
    }

    let released_regex = Regex::new(r#"(.*\s)?([0-9]+)\*?$"#).unwrap();

    if let Some(matches) = released_regex.captures(input) {
        if matches.len() >= 3 {
            let mut output = ReleasedOutput::default();
            if let Some(releaser) = matches.get(1) {
                output.releaser = Some(releaser.as_str().trim().to_string());
            }
            if let Some(year) = matches.get(2) {
                output.year = Some(year.as_str().parse::<i32>().unwrap());

                assert_ge!(
                    output.year.unwrap(),
                    1800,
                    "parsed release year was <1800: {}",
                    input
                );
                assert_le!(
                    output.year.unwrap(),
                    2100,
                    "parsed release year was >2100: {}",
                    input
                );
            }

            if input.ends_with('*') {
                output.authoritative = true;
            }

            return Some(output);
        }
    } else {
        // for no-year strings like "WSU*" or "WSU"
        let mut output = ReleasedOutput::default();

        if input.ends_with('*') {
            output.authoritative = true;
            output.releaser = Some(rem_last_n(input.trim(), 1).to_string());
        } else {
            output.releaser = Some(input.trim().to_string());
        }

        return Some(output);
    }
    panic!("couldn't parse \"released\" string: {}", input);
}

// should this date string be treated as being centered on a single month?
// we accept either "mid september" or "september" for this
// we're looking to distinguish this from regular start dates which, when charted,
// would have a window *after* the date instead of *centered on* the date
fn is_a_midpoint(input: &str) -> bool {
    let no_whitespace: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let month: String = if no_whitespace.to_lowercase().starts_with("mid") {
        no_whitespace.chars().skip(3).collect()
    } else {
        input.to_string()
    };

    // month name abbreviations with any chars other than numbers after
    let month_names_regex =
        Regex::new(r#"^(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)[^0-9]*$"#).unwrap();

    month_names_regex.captures(&month.to_lowercase()).is_some()
}

fn average(numbers: &[u32]) -> u32 {
    numbers.iter().sum::<u32>() / numbers.len() as u32
}

// "early to late August" -> "early August" and "late August"
// "late August to mid September" -> "late August" and "mid September"
// "Sep 20-30" -> "Sep 20" and "Sep 30"
// "Sep 25-Oct 5" -> "Sep 25" and "Oct 5"
// if none of these match, maybe it's a single date, pass it through to string_to_day_number() unchanged
// window_size is how big of a window to build (half before and half after) if we parse this as a midpoint

// special case: "average of: July 6, June 29"
// this is for extension pubs that give a set of dates based on measurements in multiple years
// break it apart on the commas then parse each and average the start dates, and return a start only date

// report the way it was parsed:
// * as a start date (like peaches, "September 15", "early September")
// * two dates ("September 15-30") - this can also come out of a "midpoint" parse, which would take something like "september" and return Sept 10-20, the middle 10 days of September
// single dates get a window put after them (window size configured outside this import), two dates stay as they are

// todo: also parse "early/mid/late" and "0%,50%,100%" relative ripening times (return a percentage)
const DEFAULT_WINDOW_SIZE: u32 = 10;

fn string_to_day_range(input: &str, window_size: u32) -> Option<DayRangeOutput> {
    let mut output = DayRangeOutput::default();

    // escape hatch for "time within season" strings which we aren't parsing for now
    let time_within_season_regex =
        Regex::new(r#"^(very early|early-mid|mid-late|early|mid|late|early season|mid season|late season|very late)$"#)
            .unwrap();

    if time_within_season_regex
        .captures(&input.to_lowercase())
        .is_some()
    {
        output.parse_type = DateParseType::Unparsed;
        return Some(output);
    }

    // escape hatch for some kinda indefinite time ranges in some extension pubs
    let indefinite_times_regex = Regex::new(r#"^(summer)$"#).unwrap();

    if indefinite_times_regex
        .captures(&input.to_lowercase())
        .is_some()
    {
        output.parse_type = DateParseType::Unparsed;
        return Some(output);
    }

    // special case for a list of days
    let average_of_start = "average of: ";
    if input.to_lowercase().starts_with("average of: ") {
        let list = rem_first_n(input, average_of_start.len());
        let split = list.split(',').collect::<Vec<&str>>();

        let mut parsed_days = Vec::new();
        for item in split {
            if let Some(parsed_day) = string_to_day_number(item) {
                parsed_days.push(parsed_day);
            } else {
                panic!(r#"error parsing average date from {}"#, input);
            }
        }

        output.parse_type = DateParseType::StartOnly;
        output.start = Some(average(&parsed_days));
        return Some(output);
    }

    // special case for "first harvest:" or "50% harvest" which should have only one date
    if input.to_lowercase().starts_with("first harvest:") {
        if let Some(parsed) = string_to_day_number(input) {
            output.parse_type = DateParseType::StartOnly;
            output.start = Some(parsed);
            return Some(output);
        }
    }

    if input.to_lowercase().starts_with("50% harvest:") {
        if let Some(parsed) = string_to_day_number(input) {
            output.parse_type = DateParseType::TwoDates; // treat as a midpoint
            output.start = Some(parsed - window_size / 2);
            output.end = Some(parsed + window_size / 2);
            return Some(output);
        }
    }

    // does it have "to" or "-" in it? if so, split on that and see if the right side is only a number
    // if it is, it's something like sep 20-30, and we need to pull sep from the left side and give it to the right side
    // if not, feed both sides

    // could be: September 15-30, or mid-late September, or Oct 5 - Oct 30
    if input.contains('-') || input.contains(" to ") {
        let split;

        if input.contains('-') {
            split = input.split('-').collect::<Vec<&str>>();
            assert_eq!(
                split.len(),
                2,
                "date string had more than one '-': {}",
                input
            );
        } else if input.contains(" to ") {
            split = input.split(" to ").collect::<Vec<&str>>();
            assert_eq!(
                split.len(),
                2,
                "date string had more than one ' to ': {}",
                input
            );
        } else {
            panic!("shouldn't get here, '-' or ' to ' match")
        }

        // if one part parses and not the other, then add the month from the parsing one to the not-parsing one
        if string_to_day_number(split[0]).is_none() && string_to_day_number(split[1]).is_none() {
            return None;
        }

        if string_to_day_number(split[0]).is_some() && string_to_day_number(split[1]).is_none() {
            match month_location(split[0]) {
                MonthLocationType::MonthAtBeginning => {
                    output.parse_type = DateParseType::TwoDates;
                    output.start = Some(string_to_day_number(split[0]).unwrap());
                    output.end =
                        string_to_day_number(&format!("{} {}", get_month(split[0]), split[1]));
                    if output.end.is_none() {
                        panic!(
                            "couldn't parse {} as a shared-month, month at the beginning",
                            input
                        )
                    }
                    return Some(output);
                }
                MonthLocationType::MonthAtEnd => {
                    output.parse_type = DateParseType::TwoDates;
                    output.start = Some(string_to_day_number(split[0]).unwrap());
                    output.end =
                        string_to_day_number(&format!("{} {}", split[1], get_month(split[0])));
                    if output.end.is_none() {
                        panic!(
                            "couldn't parse {} as a shared-month, month at the end",
                            input
                        )
                    }
                    return Some(output);
                }
                MonthLocationType::NoMonth => panic!("no month found in string {}", split[0]),
            }
        }

        if string_to_day_number(split[0]).is_none() && string_to_day_number(split[1]).is_some() {
            match month_location(split[1]) {
                MonthLocationType::MonthAtBeginning => {
                    output.parse_type = DateParseType::TwoDates;
                    output.start =
                        string_to_day_number(&format!("{} {}", get_month(split[1]), split[0]));
                    if output.start.is_none() {
                        panic!(r#"couldn't parse date: {:}"#, input);
                    }
                    output.end = Some(string_to_day_number(split[1]).unwrap());
                    return Some(output);
                }
                MonthLocationType::MonthAtEnd => {
                    output.parse_type = DateParseType::TwoDates;
                    output.start =
                        string_to_day_number(&format!("{} {}", split[0], get_month(split[1])));
                    if output.start.is_none() {
                        panic!(r#"couldn't parse date: {:}"#, input);
                    }
                    output.end = Some(string_to_day_number(split[1]).unwrap());
                    return Some(output);
                }
                MonthLocationType::NoMonth => panic!("no month found in string {}", split[0]),
            }
        }

        // finally see if the two halves both parse ok as-is, if so return that
        // for Oct 5 - Oct 30 or mid September - mid October
        if (string_to_day_number(split[0])).is_some() && (string_to_day_number(split[1])).is_some()
        {
            output.parse_type = DateParseType::TwoDates;
            output.start = Some(string_to_day_number(split[0]).unwrap());
            output.end = Some(string_to_day_number(split[1]).unwrap());
            return Some(output);
        }

        return None;
    }

    // if no "to" or "-" then it's a single date
    // detect bare months or mid month to set the midpoint enum, otherwise it's a start only

    match string_to_day_number(input) {
        Some(start) => {
            if is_a_midpoint(input) {
                output.parse_type = DateParseType::TwoDates;
                output.start = Some(start - window_size / 2);
                output.end = Some(start + window_size / 2);
                Some(output)
            } else {
                output.parse_type = DateParseType::StartOnly;
                output.start = Some(start);
                Some(output)
            }
        }
        None => None,
    }
}

lazy_static! {
    static ref SPECIAL_CHARACTERS_REGEX: Regex = Regex::new(r#"[^a-zA-Z0-9]"#).unwrap();
    static ref TM_REGEX: Regex = Regex::new(r#"\(tm\)|\(r\)|™|®"#).unwrap();
    static ref MONTH_SLASH_DAY_REGEX: Regex = Regex::new(r#"([0-9]+)/([0-9]+)"#).unwrap();
}

fn matches_month_slash_day(input: &str) -> bool {
    MONTH_SLASH_DAY_REGEX
        .captures(&input.to_lowercase())
        .is_some()
}

// parse a single date to a day of the year
// "September"
// "late September"
// "early-mid October"
// "Sep 25"
// "9/25" (month/day) = September 25
// "Around May 4 (Gainesville, FL)" - should pull out "May 4" with a regex and parse that
fn string_to_day_number(input: &str) -> Option<u32> {
    if let Some(matches) = MONTH_SLASH_DAY_REGEX.captures(&input.to_lowercase()) {
        if matches.len() >= 3 {
            if let (Some(month_number), Some(day_number)) = (matches.get(1), matches.get(2)) {
                if let Ok(parsed) = NaiveDateTime::parse_from_str(
                    &("2020 ".to_owned()
                        + month_number.as_str()
                        + "/"
                        + day_number.as_str()
                        + " 12:01:01"),
                    "%Y %m/%d %H:%M:%S",
                ) {
                    return Some(parsed.ordinal());
                }
            }
        }
    }

    let mut month_and_day_string = input.to_string();

    if !(input).contains(char::is_whitespace) {
        // for bare month names
        month_and_day_string = format!("{} 15", input);
    } else {
        // for inputs like "early March"
        let text_and_month_regex =
            Regex::new(r#"(early to mid|mid to late|early-mid|mid-late|early|mid|late) (.*)"#)
                .unwrap();

        if let Some(matches) = text_and_month_regex.captures(&input.to_lowercase()) {
            let day_of_month;
            if matches.len() >= 3 {
                match &matches[1] {
                    "early" => day_of_month = 5,
                    "early to mid" | "early-mid" => day_of_month = 10,
                    "mid" => day_of_month = 15,
                    "mid to late" | "mid-late" => day_of_month = 20,
                    "late" => day_of_month = 25,
                    _ => panic!("matched a date prefix not in this match statement"),
                }

                month_and_day_string = format!("{} {}", &matches[2], day_of_month);
            }
        } else {
            // try to pull a month+day string out of the middle of a bunch of text. helps us parse things that were left in some original sentence format
            let month_and_day_regex =
                Regex::new(r#"(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)[^0-9]*([0-9]+)"#)
                    .unwrap();

            if let Some(matches) = month_and_day_regex.captures(&input.to_lowercase()) {
                month_and_day_string = format!("{} {}", &matches[1], &matches[2]);
            }
        }

        // default: parse the input as it came, for dates that already look like like "September 25"
    }

    // wrap this with a year and time of day so we can parse it, then get the day of the year back out
    // 2020 was a leap year
    match NaiveDateTime::parse_from_str(
        &("2020 ".to_owned() + &month_and_day_string + " 12:01:01"),
        "%Y %B %d %H:%M:%S",
    ) {
        Ok(parsed) => Some(parsed.ordinal()),
        Err(_) => {
            //    eprintln!(
            //        "date parsing: {} with input {}",
            //        e,
            //        "2020 ".to_owned() + &month_and_day_string + " 12:01:01"
            //    );
            None
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct PatentInfo {
    uspp_number: Option<String>,
    uspp_expiration: Option<NaiveDateTime>,
}

fn string_to_patent_info(input: &str) -> PatentInfo {
    let mut output = PatentInfo {
        uspp_number: None,
        uspp_expiration: None,
    };

    let uspp_regex = Regex::new(r#"USPP([0-9]+)"#).unwrap();
    let google_format_date_regex =
        Regex::new(r#"USPP[0-9]+.+([0-9]{4})-([0-9]{2})-([0-9]{2})$"#).unwrap();
    let plain_year_date_regex = Regex::new(r#"USPP[0-9]+.+([0-9]{4})$"#).unwrap();

    if let Some(matches) = uspp_regex.captures(input) {
        if matches.len() >= 2 {
            output.uspp_number = Some(matches[1].to_string());
        }
    }

    // date can be either "2017-01-02" or "2017" and we presume Jan 1.  year-only dates should be used for past dates only
    match google_format_date_regex.captures(input) {
        Some(matches) => {
            if matches.len() >= 4 {
                output.uspp_expiration = Some(
                    NaiveDate::from_ymd_opt(
                        matches[1].parse::<i32>().unwrap(),
                        matches[2].parse::<u32>().unwrap(),
                        matches[3].parse::<u32>().unwrap(),
                    )
                    .unwrap()
                    .and_hms_opt(12, 0, 0)
                    .unwrap(),
                );
            }
        }
        None => {
            if let Some(matches) = plain_year_date_regex.captures(input) {
                if matches.len() >= 2 {
                    output.uspp_expiration = Some(
                        NaiveDate::from_ymd_opt(matches[1].parse::<i32>().unwrap(), 1, 1)
                            .unwrap()
                            .and_hms_opt(12, 0, 0)
                            .unwrap(),
                    );
                }
            }
        }
    }

    output
}

#[derive(Debug, Default)]
pub struct LoadAllReturn {
    pub facts_found: isize,
    pub base_plants_found: isize,
    pub base_types_found: isize,
    pub reference_items: LoadReferencesReturn,
}

#[derive(Clone, Copy, Debug)]
struct ReferenceLoadSlice {
    start: usize,
    len: usize,
}

impl ReferenceLoadSlice {
    fn contains(&self, index: usize) -> bool {
        index >= self.start && index < self.start.saturating_add(self.len)
    }
}

#[derive(Clone, Copy, Debug)]
struct LoadAllOptions {
    reference_slice: Option<ReferenceLoadSlice>,
    reference_load_time_budget: Option<Duration>,
    skip_loaded_references: bool,
    static_data_load_mode: StaticDataLoadMode,
    post_load_processing_mode: PostLoadProcessingMode,
    write_generated_files: bool,
}

impl Default for LoadAllOptions {
    fn default() -> Self {
        Self {
            reference_slice: None,
            reference_load_time_budget: None,
            skip_loaded_references: false,
            static_data_load_mode: StaticDataLoadMode::Always,
            post_load_processing_mode: PostLoadProcessingMode::Always,
            write_generated_files: true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum StaticDataLoadMode {
    Always,
    IfEmpty,
}

#[derive(Clone, Copy, Debug)]
enum PostLoadProcessingMode {
    Always,
    WhenNoReferencesLoaded,
}

pub fn establish_connection() -> SqliteConnection {
    let database_url = "database.sqlite3";
    SqliteConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn ensure_database_schema(db_conn: &mut SqliteConnection) {
    db_conn.run_pending_migrations(crate::MIGRATIONS).unwrap();
}

pub fn reset_database(db_conn: &mut SqliteConnection) {
    ensure_database_schema(db_conn);

    let _ = diesel::delete(base_plants::dsl::base_plants).execute(db_conn);
    // skip dropping: users
    let _ = diesel::delete(plant_types::dsl::plant_types).execute(db_conn);
    let _ = diesel::delete(collection_categories::dsl::collection_categories).execute(db_conn);
    let _ = diesel::delete(collections::dsl::collections).execute(db_conn);
    let _ = diesel::delete(locations::dsl::locations.filter(locations::user_id.is_null()))
        .execute(db_conn);
    let _ = diesel::delete(
        collection_items::dsl::collection_items.filter(collection_items::user_id.is_null()),
    )
    .execute(db_conn);
    let _ = diesel::delete(facts::dsl::facts).execute(db_conn);

    ensure_database_schema(db_conn);
}

pub fn load_all(db_conn: &mut SqliteConnection) -> LoadAllReturn {
    load_all_with_options(db_conn, LoadAllOptions::default())
}

fn load_all_with_options(db_conn: &mut SqliteConnection, options: LoadAllOptions) -> LoadAllReturn {
    let database_dir = get_database_dir().unwrap();

    let facts_found = if matches!(options.static_data_load_mode, StaticDataLoadMode::Always)
        || count_facts(db_conn) == 0
    {
        load_facts(db_conn, database_dir.clone())
    } else {
        0
    };
    let base_plants_found = if matches!(options.static_data_load_mode, StaticDataLoadMode::Always)
        || count_base_plants(db_conn) == 0
    {
        load_base_plants(db_conn, database_dir.clone())
    } else {
        0
    };
    let base_types_found = if matches!(options.static_data_load_mode, StaticDataLoadMode::Always)
        || count_plant_types(db_conn) == 0
    {
        load_types(db_conn, database_dir.clone())
    } else {
        0
    };
    let load_references_return = load_references(db_conn, database_dir, options);

    let should_run_post_load_processing = match options.post_load_processing_mode {
        PostLoadProcessingMode::Always => true,
        PostLoadProcessingMode::WhenNoReferencesLoaded => {
            load_references_return.reference_collections_loaded == 0
                && load_references_return.reference_collections_skipped
                    == load_references_return.reference_collections_available
        }
    };

    if should_run_post_load_processing {
        run_post_load_processing(db_conn, options.write_generated_files);
    }

    LoadAllReturn {
        facts_found,
        base_plants_found,
        base_types_found,
        reference_items: load_references_return,
    }
}

fn run_post_load_processing(db_conn: &mut SqliteConnection, write_generated_files: bool) {
    println!("removing ignored base plants");
    remove_ignored_base_plants(db_conn);
    println!("calculating relative harvest times");
    relative_to_absolute_harvest_times(db_conn);
    calculate_relative_harvest_times(db_conn);
    calculate_years_from_patent(db_conn);
    println!("adding marketing names to collection items");
    add_marketing_names(db_conn);
    println!("calculating notoriety");
    calculate_notoriety(db_conn);
    if write_generated_files {
        calculate_and_write_relative_day_offsets(db_conn);
    }
    println!("adding base plant ID to collection items");
    add_base_id_to_collection_items(db_conn);
    if write_generated_files {
        write_needs_help_file(db_conn);
    }
    println!("rebuilding fts tables");
    rebuild_fts(db_conn);
    println!("checking database");
    check_database(db_conn);
}

pub fn get_database_dir() -> Option<std::path::PathBuf> {
    let max_up_traversal_levels = 4; // enough levels to work from the build dir on windows
    let mut i = 0;

    while i <= max_up_traversal_levels {
        let mut path = std::path::PathBuf::from(".");
        for _ in 0..i {
            path = path.join("..");
        }
        path = path.join("plant_database");
        match fs::metadata(path.clone()) {
            Ok(md) => {
                if md.is_dir() {
                    println!("found database dir: {}", path.display());
                    return Some(path);
                }
            }
            Err(_) => {
                // println!("not a dir")
            }
        }
        i += 1;
    }

    None
}

// given input like ".../references/Oregon/Willamette Valley"
// get the part after "references", change any '\' to '/' (for windows) and add a trailing '/'
// return "Oregon/Willamette Valley/"
fn format_path(input: &str) -> String {
    let v: Vec<&str> = input.split("references").collect();

    let after_references = v.last().unwrap();
    // println!("split result: {}", after_references);

    let mutable: &str = if after_references.is_empty() {
        after_references
    } else {
        match after_references.chars().next().unwrap() {
            '/' => rem_first_n(after_references, 1),
            '\\' => rem_first_n(after_references, 1),
            _ => after_references,
        }
    };

    let mut string_mutable = str::replace(mutable, r#"\"#, "/");
    string_mutable.push('/');
    string_mutable
}

pub struct AkaFormatted {
    pub aka: Option<String>,
    pub aka_fts: Option<String>,
    pub marketing_name: Option<String>,
}

// fts: full text search
fn format_name_fts_string(name: &str) -> String {
    let without_tm = TM_REGEX.replace_all(name, "");
    SPECIAL_CHARACTERS_REGEX
        .replace_all(&without_tm, "")
        .to_string()
        .to_lowercase()
}

fn does_name_contain_tm(name: &str) -> bool {
    TM_REGEX.is_match(name)
}

// turn an array like ["20th Century", "Twentieth Century"] into "aka" and "aka_fts" strings
// aka:      comma-separated list (remove commas in names)
// aka_fts:  same, but without characters like '-' and ' ' to make full text search work better
fn format_aka_strings(aka_array: &Option<Vec<String>>) -> AkaFormatted {
    if let Some(aka_array) = aka_array {
        let mut aka_string_builder = "".to_string();
        let mut aka_fts_string_builder = "".to_string();
        let mut marketing_name: Option<String> = None;

        let mut first_element = true;
        for aka_element in aka_array {
            let commas_regex = Regex::new(r",").unwrap();
            let without_commas = commas_regex.replace_all(aka_element, ""); // note - commas are also removed by format_name_fts_string() in is current version
            let fts_formatted = format_name_fts_string(aka_element);

            if first_element {
                first_element = false;
            } else {
                aka_string_builder += ",";
                aka_fts_string_builder += ",";
            }

            aka_string_builder += &without_commas;
            aka_fts_string_builder += &fts_formatted;

            if does_name_contain_tm(aka_element) {
                if marketing_name.is_none() {
                    marketing_name = Some(aka_element.to_string());
                } else {
                    // multiple marketing names are very rare, see rave/first kiss apple
                    marketing_name = Some(marketing_name.unwrap() + " and " + aka_element);
                }
            }
        }
        AkaFormatted {
            aka: Some(aka_string_builder),
            aka_fts: Some(aka_fts_string_builder),
            marketing_name,
        }
    } else {
        AkaFormatted {
            aka: None,
            aka_fts: None,
            marketing_name: None,
        }
    }
}

// splits apart a string like "20th Century,Twentieth Centry,..."
fn decode_aka_string(input: &str) -> Vec<&str> {
    input.split(',').collect::<Vec<_>>()
}

// given strings like "S1S4 [13]" or "S3S6 [12,13]" or "S1S4' [12] or S3S6 [13,14] (conflicting sources)"
// parse these and break them into:
// S-allele string -> [set of collection numbers]
fn parse_s_allele_string(input: &Option<String>) -> HashMap<String, HashSet<i32>> {
    let mut output = Default::default();

    if input.is_none() {
        return output;
    }
    // gets pairs of text-before-brackets plus the bracket contents
    let s_allele_regex_1 = Regex::new(r#"([a-zA-Z0-9']+) +\[([0-9,]+)\]"#).unwrap();

    // todo
    for cap in s_allele_regex_1.captures_iter(input.as_ref().unwrap()) {
        // todo - 2nd regex to break up multiple S-alleles in the first patch
        // println!("Month: {} Day: {} Year: {}", &cap[2], &cap[3], &cap[1]);

        let s_allele = &cap[1]; // like "S1S4'"
        let collections = &cap[2]; // like "12" or "12,13"

        let collections: Vec<i32> = collections
            .split(',')
            .map(|x| x.parse::<i32>().unwrap())
            .collect();

        output.insert(
            s_allele.to_string(),
            HashSet::from_iter(collections.iter().cloned()),
        );
    }

    output
}

// S-allele data is stored in a plain string in the database, during database loading we may need to combine data
// from multiple sources and build this string up. so it gets parsed into a struct, the struct edited, then written
// back to a string to put in the database
// examples:
// existing: "S3S6 [12]" new: "S1S4 [13]" -> "S3S6 [12] or S1S4 [13] (conflicting sources)"
// existing: "S3S6 [12]" new: "S3S6 [13]" -> "S3S6 [12,13]"
fn format_s_allele(existing: &Option<String>, new: &Option<String>) -> String {
    let mut existing = parse_s_allele_string(existing);
    let new = parse_s_allele_string(new);

    // for each S-allele in the new one, see if it's in the existing one, if so combine their collection lists

    // add new s-alleles to old
    for (key, value) in new.into_iter() {
        if let std::collections::hash_map::Entry::Vacant(e) = existing.entry(key.clone()) {
            e.insert(value);
        } else {
            let existing_value = existing.get_mut(&key).unwrap();

            existing_value.extend(&value);
        }
    }

    let mut strings: Vec<String> = Default::default();
    for (key, value) in existing.into_iter() {
        strings.push(format!(
            "{key} [{}]",
            value
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .iter()
                .sorted()
                .join(",")
        ));
    }

    let mut output = strings.iter().sorted().join(" or ");
    // write output back as a string
    // add "(conflicting sources)" if there are multiple S-alleles
    // todo
    if strings.len() > 1 {
        output += " (conflicting sources)"
    }

    output
}

// check a new value from a collection item against something already in the database
// the new value should be either an exact match for the database value, or it should be brand new (replacing a "None")
fn new_or_old<T: std::cmp::PartialEq + std::fmt::Debug>(
    old: Option<T>,
    new: Option<T>,
    plant: &BasePlantJson,
    field_name_for_error_message: &str,
) -> Option<T> {
    if let Some(old) = old {
        if let Some(new) = new {
            assert_eq!(
                old, new,
                "tried to update field {} for plant but it was already set {:?}",
                field_name_for_error_message, plant
            );
        }
        Some(old)
    } else {
        new
    }
}

fn new_or_old_patent_expiration(
    old: Option<i64>,
    old_estimated: Option<i32>,
    new: Option<i64>,
    plant: &BasePlantJson,
) -> (Option<i64>, Option<i32>) {
    if let Some(new) = new {
        if let Some(old) = old {
            if old == new {
                let old_estimated = if old_estimated == Some(1) {
                    Some(0)
                } else {
                    old_estimated
                };
                return (Some(old), old_estimated);
            }

            if old_estimated == Some(1) {
                return (Some(new), Some(0));
            }

            assert_eq!(
                old, new,
                "tried to update field uspp_expiration for plant but it was already set {:?}",
                plant
            );
            (Some(old), old_estimated)
        } else {
            (Some(new), Some(0))
        }
    } else {
        (old, old_estimated)
    }
}

// we allow references to set some top-level fields, as long as they're either previously unset or an exact match
fn apply_top_level_fields(
    db_conn: &mut SqliteConnection,
    plant: &BasePlantJson,
    plant_type: String,
    current_collection_id: Option<i32>,
) {
    // find existing base plant (must exist)
    let existing_base_plant = base_plants::dsl::base_plants
        .filter(base_plants::name.eq(&plant.name))
        .filter(base_plants::type_.eq(plant_type.clone()))
        .first::<BasePlant>(db_conn)
        .unwrap_or_else(|_| {
            panic!(
                r#"couldn't find existing base plant to apply top level fields to: {:?}"#,
                plant
            )
        });

    // make an s-allele update if we have one
    let s_allele = if plant.s_allele.is_some() && current_collection_id.is_some() {
        let new_s_allele = format!(
            "{} [{}]",
            plant.s_allele.as_ref().unwrap(),
            current_collection_id.unwrap()
        );
        Some(format_s_allele(
            &existing_base_plant.s_allele,
            &Some(new_s_allele),
        ))
    } else {
        existing_base_plant.s_allele
    };

    let aka_formatted = format_aka_strings(&plant.aka);

    // check existing values: new value should be either an exact match, or not yet set in the database
    let aka = new_or_old(existing_base_plant.aka, aka_formatted.aka, plant, "aka");

    let aka_fts = new_or_old(
        existing_base_plant.aka_fts,
        aka_formatted.aka_fts,
        plant,
        "aka_fts",
    );

    let marketing_name = new_or_old(
        existing_base_plant.marketing_name,
        aka_formatted.marketing_name,
        plant,
        "marketing_name",
    );

    let mut patent_info = Default::default();
    if let Some(patent_string) = &plant.patent {
        patent_info = string_to_patent_info(patent_string);
    }

    let mut uspp_expiration_i64 = None;
    if let Some(uspp_expiration) = patent_info.uspp_expiration {
        uspp_expiration_i64 = Some(uspp_expiration.timestamp());
    }

    let uspp_number = new_or_old(
        existing_base_plant.uspp_number,
        patent_info.uspp_number,
        plant,
        "uspp_number",
    );

    let (uspp_expiration, uspp_expiration_estimated) = new_or_old_patent_expiration(
        existing_base_plant.uspp_expiration,
        existing_base_plant.uspp_expiration_estimated,
        uspp_expiration_i64,
        plant,
    );

    let mut release_parsed = None;
    if let Some(released) = &plant.released {
        release_parsed = parse_released(released);
    }

    let mut release_year = None;
    let mut releaser = None;
    let mut release_authoritative = false;
    if let Some(release_parsed) = release_parsed {
        release_year = release_parsed.year;
        releaser = release_parsed.releaser;
        release_authoritative = release_parsed.authoritative;
    }

    let release_year_is_patent_derived =
        existing_base_plant.release_year_note.as_deref() == Some("derived from patent number");
    let incoming_release_year = release_year;
    let release_year = if release_year_is_patent_derived && incoming_release_year.is_some() {
        incoming_release_year
    } else {
        new_or_old(
            existing_base_plant.release_year,
            incoming_release_year,
            plant,
            "release_year",
        )
    };
    let release_year_note = if release_year_is_patent_derived && incoming_release_year.is_some() {
        None
    } else {
        existing_base_plant.release_year_note
    };

    let released_by = new_or_old(
        existing_base_plant.released_by,
        releaser,
        plant,
        "released_by",
    );

    // only update this if our parsed release string tells us this is the authoritative collection
    let mut release_collection_id = None;
    if release_authoritative {
        release_collection_id = current_collection_id;
    }

    let release_collection_id = new_or_old(
        existing_base_plant.release_collection_id,
        release_collection_id,
        plant,
        "release_collection_id",
    );

    let updated_row_count =
        diesel::update(base_plants::dsl::base_plants.filter(base_plants::name.eq(&plant.name)))
            .filter(base_plants::type_.eq(plant_type))
            .set((
                base_plants::aka.eq(&aka),
                base_plants::aka_fts.eq(&aka_fts),
                base_plants::marketing_name.eq(&marketing_name),
                base_plants::uspp_number.eq(uspp_number.clone()),
                base_plants::uspp_expiration.eq(uspp_expiration),
                base_plants::uspp_expiration_estimated.eq(uspp_expiration_estimated),
                base_plants::release_year.eq(release_year),
                base_plants::release_year_note.eq(release_year_note),
                base_plants::released_by.eq(released_by),
                base_plants::release_collection_id.eq(release_collection_id),
                base_plants::s_allele.eq(s_allele),
            ))
            .execute(db_conn);
    assert_eq!(
        Ok(1),
        updated_row_count,
        "failed inserting {} {:?}",
        plant.name,
        uspp_number
    );
}

// todo: remove the notion of a set of base plants files, we can cover all of them through collections instead
pub fn load_base_plants(db_conn: &mut SqliteConnection, database_dir: std::path::PathBuf) -> isize {
    let mut plants_found = 0;

    let file_paths = fs::read_dir(database_dir.join("plants")).unwrap();

    for file_path in file_paths {
        let path_ = file_path.unwrap().path();

        if fs::metadata(path_.clone()).unwrap().is_file()
            && path_.extension().unwrap().to_str().unwrap() == "json5"
        {
            println!("loading: {}", path_.display());

            let contents = fs::read_to_string(path_.clone()).unwrap();

            let plants: Vec<BasePlantJson> = json5::from_str(&contents).unwrap_or_else(|_| {
                panic!(
                    "Error parsing as a json array of base plant entries: {}",
                    path_.display()
                )
            });

            let filename = rem_last_n(
                path_.as_path().file_name().unwrap().to_str().unwrap(),
                ".json5".len(),
            );

            for plant in &plants {
                // for the "Oddball.json5" file, get type from each item's json
                // all others get type from the filename
                let plant_type = if filename.starts_with("Oddball") {
                    plant.type_.clone()
                } else {
                    Some(filename.to_string())
                };

                // println!("inserting");
                let rows_inserted = diesel::insert_into(base_plants::dsl::base_plants)
                    .values((
                        base_plants::name.eq(&plant.name),
                        base_plants::name_fts.eq(format_name_fts_string(&plant.name)),
                        base_plants::type_.eq(&plant_type.clone().unwrap()),
                        base_plants::description.eq(&plant.description),
                        base_plants::s_allele.eq(&plant.s_allele),
                        base_plants::number_of_references.eq(0),
                        base_plants::ignore_unless_in_others.eq(0),
                    ))
                    .execute(db_conn);
                assert_eq!(Ok(1), rows_inserted);
                plants_found += 1;

                apply_top_level_fields(db_conn, plant, plant_type.clone().unwrap(), None);
            }
        }
    }

    plants_found
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
struct TypeGroupsJson {
    group_name: String,
    types: Vec<TypeJson>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
struct TypeJson {
    name: String,
    latin_name: Option<String>,
}

fn load_types(db_conn: &mut SqliteConnection, database_dir: std::path::PathBuf) -> isize {
    let mut types_found = 0;

    let types_path = database_dir.join("types.json5");
    if !fs::metadata(types_path.clone()).unwrap().is_file() {
        panic!("didn't find types.json5");
    }

    let contents = fs::read_to_string(types_path).unwrap();

    let types_groups_parsed: Vec<TypeGroupsJson> = json5::from_str(&contents).unwrap();

    let mut types_for_generated: Vec<String> = Default::default();

    for type_group in &types_groups_parsed {
        for type_element in &type_group.types {
            let rows_inserted = diesel::insert_into(plant_types::dsl::plant_types)
                .values((
                    plant_types::group_name.eq(&type_group.group_name),
                    plant_types::name.eq(&type_element.name),
                    plant_types::latin_name.eq(&type_element.latin_name),
                ))
                .execute(db_conn);
            assert_eq!(Ok(1), rows_inserted);
            types_found += 1;

            types_for_generated.push(type_element.name.clone());
        }
    }

    let header = r#"// created by import_db.rs - don't edit

pub const TYPES: &[&str] = &[
"#;

    let data: String = types_for_generated
        .into_iter()
        .map(|type_| format!("    \"{type_}\",\n"))
        .collect::<String>();

    let footer = r#"];
"#;

    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path = path.join("./src/import_db/types_generated.rs");

    fs::write(path, header.to_owned() + &data + footer)
        .expect("Unable to write import_db/types_generated.rs");

    types_found
}

fn get_category_description(
    category: &Option<String>,
    category_description: &Option<String>,
    categories: &Option<Vec<CollectionCategoryJson>>,
) -> Option<String> {
    // if category_description is empty, see if we can get it from the top-level list of categories
    // this is supported to save typing out the same category description a bunch of times
    if category.is_some() && category_description.is_none() && categories.is_some() {
        for category_top_level in categories.as_ref().unwrap() {
            if &category_top_level.name == category.as_ref().unwrap() {
                return category_top_level.description.clone();
            }
        }
    }

    category_description.clone()
}

struct AddCollectionPlantType<'a> {
    collection_id: i32,
    location_id: Option<i32>,
    location_number: i32,
    path_and_filename: &'a str,
    harvest_time: &'a Option<String>,
    plant_name: &'a str,
    plant: &'a CollectionPlantJson,
    category_description: &'a Option<String>,
    db_conn: &'a mut SqliteConnection,
}

fn add_collection_plant(input: AddCollectionPlantType) -> isize {
    let mut harvest_start = None;
    let mut harvest_end = None;
    let mut harvest_start_2 = None; // fig breba+main
    let mut harvest_end_2 = None; // fig breba+main
    if let Some(harvest_time) = input.harvest_time {
        if harvest_time.is_empty() {
            panic!(
                r#"harvest time was an empty string for {:?}: {}"#,
                input.plant, harvest_time
            );
        }

        // for harvest times like "Jun/Sep" which are for fig breba+main crops
        if harvest_time.contains('/') && !matches_month_slash_day(harvest_time) {
            let split = harvest_time.split('/').collect::<Vec<&str>>();
            assert_eq!(
                split.len(),
                2,
                r#"date string had multiple '/' {:?}: {}"#,
                input.plant,
                harvest_time
            );

            match string_to_day_range(split[0], DEFAULT_WINDOW_SIZE) {
                Some(day_range) => {
                    if let Some(start) = day_range.start {
                        harvest_start = Some(i32::try_from(start).unwrap());
                    }
                    if let Some(end) = day_range.end {
                        harvest_end = Some(i32::try_from(end).unwrap());
                    }
                }
                None => {
                    panic!(
                        r#"couldn't parse date for {:?}: {}"#,
                        input.plant, harvest_time
                    );
                }
            }
            match string_to_day_range(split[1], DEFAULT_WINDOW_SIZE) {
                Some(day_range) => {
                    if let Some(start) = day_range.start {
                        harvest_start_2 = Some(i32::try_from(start).unwrap());
                    }
                    if let Some(end) = day_range.end {
                        harvest_end_2 = Some(i32::try_from(end).unwrap());
                    }
                }
                None => {
                    panic!(
                        r#"couldn't parse date for {:?}: {}"#,
                        input.plant, harvest_time
                    );
                }
            }
        } else {
            match string_to_day_range(harvest_time, DEFAULT_WINDOW_SIZE) {
                Some(day_range) => {
                    if let Some(start) = day_range.start {
                        harvest_start = Some(i32::try_from(start).unwrap());
                    }
                    if let Some(end) = day_range.end {
                        harvest_end = Some(i32::try_from(end).unwrap());
                    }
                }
                None => {
                    panic!(
                        r#"couldn't parse date for {:?}: {}"#,
                        input.plant, harvest_time
                    );
                }
            }
        }
    }

    // we may get "harvest_time_unparsed" in some cases with no "harvest_time". save "harvest_time_unparsed" for the helper text
    let harvest_time_helper_text =
        if input.harvest_time.is_none() && input.plant.harvest_time_unparsed.is_some() {
            Some(input.plant.harvest_time_unparsed.as_ref().unwrap())
        } else {
            input.harvest_time.as_ref()
        };

    // println!(
    //     "inserting {} C {} L {:?}",
    //     input.plant_name, input.collection_id, input.location_id
    // );

    let result = diesel::insert_into(collection_items::dsl::collection_items)
        .values((
            collection_items::collection_id.eq(input.collection_id),
            collection_items::location_id.eq(input.location_id),
            collection_items::location_number.eq(input.location_number),
            collection_items::path_and_filename.eq(input.path_and_filename),
            collection_items::name.eq(input.plant_name),
            collection_items::type_.eq(&input.plant.type_),
            collection_items::category.eq(&input.plant.category),
            collection_items::category_description.eq(input.category_description),
            collection_items::disease_resistance
                .eq(serde_json::to_string(&input.plant.disease_resistance).unwrap()),
            collection_items::disease_resistance.eq(&input.plant.chill),
            collection_items::s_allele.eq(&input.plant.s_allele),
            collection_items::description.eq(&input.plant.description),
            collection_items::harvest_relative.eq(&input.plant.harvest_time_relative),
            collection_items::harvest_text.eq(harvest_time_helper_text),
            collection_items::harvest_start.eq(harvest_start),
            collection_items::harvest_end.eq(harvest_end),
            collection_items::harvest_start_2.eq(harvest_start_2),
            collection_items::harvest_end_2.eq(harvest_end_2),
        ))
        .execute(input.db_conn);
    assert_eq!(
        Ok(1),
        result,
        "failed inserting {} {:?} {:?} C {} L {:?}",
        input.plant_name,
        result,
        input.plant,
        input.collection_id,
        input.location_id
    );

    1
}

// get collection name from collection ID. used for the notoriety text description
fn get_collection_name(collection_id: Option<i32>, db_conn: &mut SqliteConnection) -> String {
    if let Some(collection_id) = collection_id {
        if let Ok(collection) = collections::dsl::collections
            .filter(collections::id.eq(collection_id))
            .first::<Collection>(db_conn)
        {
            if let Some(title) = collection.title {
                title
            } else {
                format!("collection {}", collection_id)
            }
        } else {
            format!("collection {} not found", collection_id)
        }
    } else {
        "no collection".to_string()
    }
}

// if we're given a location like "I" and we have "short_name" matching "I" in our top-level locations list,
// return "name" from the top level
fn get_location_name(
    plant_location_name: Option<String>,
    locations: &[CollectionLocationJson],
) -> Option<String> {
    // println!("getting location name: {:?} {:?}", plant_location_name, locations);

    match plant_location_name {
        Some(plant_location_name) => {
            for location_top_level in locations {
                if let Some(short_name) = &location_top_level.short_name {
                    if short_name.eq(&plant_location_name) {
                        return Some(location_top_level.name.clone());
                    }
                }
            }

            // no translation (this will be most of the time)
            Some(plant_location_name)
        }
        None => {
            // no name given: if we have a single top-level location, return that
            if locations.len() == 1 {
                Some(locations[0].name.clone())
            } else {
                // if we have multiple top-level locations, return none
                None
            }
        }
    }
}

struct LocationNumbers {
    location_id: Option<i32>,
    location_number: i32,
}

fn get_location_numbers(
    collection_id: i32,
    location_name: Option<String>,
    db_conn: &mut SqliteConnection,
) -> LocationNumbers {
    // either look up this location ID by (collection ID + name) or look it up with only collection ID and expect only one result
    let locations = locations::dsl::locations
        .filter(locations::collection_id.eq(collection_id))
        .filter(locations::location_name.eq(&location_name))
        .load::<Location>(db_conn);

    // println!("{:#?} {:#?}", location_name, locations); // todo remove

    if let Ok(locations) = locations {
        if locations.len() == 1 {
            return LocationNumbers {
                location_id: Some(locations[0].id),
                location_number: locations[0].location_number,
            };
        }
    }

    LocationNumbers {
        location_id: None,
        location_number: 0,
    }
}

fn update_or_add_base_plant(
    plant_name: &str,
    plant: &CollectionPlantJson,
    db_conn: &mut SqliteConnection,
    current_collection_id: i32,
    current_collection_score: f32,
    ignore_unless_in_others: Option<bool>,
) -> isize {
    let existing_base_plant = base_plants::dsl::base_plants
        .filter(base_plants::name.eq(&plant_name))
        .filter(base_plants::type_.eq(&plant.type_))
        .first::<BasePlant>(db_conn);

    let mut new_ignore_unless_in_others: i32;
    if let Some(ignore_unless_in_others) = ignore_unless_in_others {
        if ignore_unless_in_others {
            new_ignore_unless_in_others = 1;
        } else {
            new_ignore_unless_in_others = 0;
        }
    } else {
        new_ignore_unless_in_others = 0
    }

    let num_added = if existing_base_plant.is_err() {
        // a plant in a reference that isn't already in base_plants - need to add

        let rows_inserted = diesel::insert_into(base_plants::dsl::base_plants)
            .values((
                base_plants::name.eq(&plant_name),
                base_plants::name_fts.eq(format_name_fts_string(plant_name)),
                base_plants::type_.eq(&plant.type_),
                base_plants::number_of_references.eq(1),
                base_plants::notoriety_highest_collection_score.eq(current_collection_score),
                base_plants::notoriety_highest_collection_score_id.eq(current_collection_id),
                base_plants::ignore_unless_in_others.eq(new_ignore_unless_in_others),
            ))
            .execute(db_conn);
        assert_eq!(
            Ok(1),
            rows_inserted,
            "inserting base plant {} {}",
            plant_name,
            plant.type_
        );

        1
    } else {
        let existing_base_plant = existing_base_plant.as_ref().unwrap();

        let new_references_count = existing_base_plant.number_of_references + 1;

        let new_collection_score;
        let new_collection_score_id;
        if let Some(existing_score) = existing_base_plant.notoriety_highest_collection_score {
            if current_collection_score > existing_score {
                new_collection_score = current_collection_score;
                new_collection_score_id = Some(current_collection_id);
            } else {
                new_collection_score = existing_score;
                new_collection_score_id = existing_base_plant.notoriety_highest_collection_score_id;
            }
        } else {
            new_collection_score = current_collection_score;
            new_collection_score_id = Some(current_collection_id);
        }

        // ignore state: don't overwrite an existing "false" with a new "true"
        if new_ignore_unless_in_others == 1 && existing_base_plant.ignore_unless_in_others == 0 {
            new_ignore_unless_in_others = 0;
        }

        let updated_row_count = diesel::update(
            base_plants::dsl::base_plants.filter(base_plants::id.eq(existing_base_plant.id)),
        )
        .set((
            base_plants::number_of_references.eq(new_references_count),
            base_plants::notoriety_highest_collection_score.eq(new_collection_score),
            base_plants::notoriety_highest_collection_score_id.eq(new_collection_score_id),
            base_plants::ignore_unless_in_others.eq(new_ignore_unless_in_others),
        ))
        .execute(db_conn);
        assert_eq!(Ok(1), updated_row_count);

        0
    };

    let base_plant = BasePlantJson {
        name: plant_name.to_string(),
        type_: Some(plant.type_.clone()),
        description: None,
        aka: plant.aka.clone(),
        patent: plant.patent.clone(),
        released: plant.released.clone(),
        s_allele: plant.s_allele.clone(),
    };
    apply_top_level_fields(
        db_conn,
        &base_plant,
        plant.type_.clone(),
        Some(current_collection_id),
    );
    num_added
}

fn add_collection_plant_by_location(
    collection_id: i32,
    path_and_filename: String,
    plant_name: &str,
    plant: &CollectionPlantJson,
    category_description: &Option<String>,
    collection_locations: &[CollectionLocationJson],
    db_conn: &mut SqliteConnection,
) -> isize {
    // see if plant.locations exists

    let mut plants_added: isize = 0;
    if plant.locations.is_some() {
        for location in plant.locations.clone().unwrap() {
            if location.is_string() {
                // type 1: "locations": ["I", "II", "III", "IV"] - this type has the same ripening time for all locations
                // (which is surely inaccurate, but it's what we get with some extension publications)

                // we get harvest time for each location from the base harvest time values

                let location_numbers = get_location_numbers(
                    collection_id,
                    get_location_name(
                        Some(location.as_str().unwrap().to_string()), // the .as_str()... nastiness is because serde wants to carry the "it's a json string!!" idea to the point of printing it a certain way in rust. as_str() tells it not to
                        collection_locations,
                    ),
                    db_conn,
                );
                plants_added += add_collection_plant(AddCollectionPlantType {
                    collection_id,
                    location_id: location_numbers.location_id,
                    location_number: location_numbers.location_number,
                    path_and_filename: &path_and_filename,
                    harvest_time: &plant.harvest_time,
                    plant_name,
                    plant,
                    category_description,
                    db_conn,
                });
            } else {
                // deserialize to type II like
                //     "locations": [
                //         {"San Joaquin Valley": "Oct-Nov"},
                //         {"Sacramento Valley": "late Oct-Nov"},
                //         ...
                //     ]

                // we get harvest time from the locations array (and it's absolute only, not relative)

                let location_objects: HashMap<String, String> =
                    serde_json::from_value(location).unwrap();

                for (location_name, harvest_time) in location_objects {
                    if harvest_time == "NA" {
                        // skip this - see the UC Davis charts for example
                        continue;
                    }

                    let location_numbers = get_location_numbers(
                        collection_id,
                        get_location_name(Some(location_name), collection_locations),
                        db_conn,
                    );

                    plants_added += add_collection_plant(AddCollectionPlantType {
                        collection_id,
                        location_id: location_numbers.location_id,
                        location_number: location_numbers.location_number,
                        path_and_filename: &path_and_filename,
                        harvest_time: &Some(harvest_time),
                        plant_name,
                        plant,
                        category_description,
                        db_conn,
                    });
                }
            }
        }

        // the plant needs to match one of our locations, either name or short_name
    } else {
        // no location given in the plant json

        let location_numbers = get_location_numbers(
            collection_id,
            get_location_name(None, collection_locations),
            db_conn,
        );

        plants_added += add_collection_plant(AddCollectionPlantType {
            collection_id,
            location_id: location_numbers.location_id,
            location_number: location_numbers.location_number,
            path_and_filename: &path_and_filename,
            harvest_time: &plant.harvest_time,
            plant_name,
            plant,
            category_description,
            db_conn,
        });
    }

    plants_added
}

#[derive(Debug, Default)]
pub struct LoadReferencesReturn {
    pub reference_collections_available: isize,
    pub reference_collections_loaded: isize,
    pub reference_collections_skipped: isize,
    pub reference_locations_found: isize,
    pub reference_base_plants_added: isize,
    pub reference_plants_added: isize,
}

fn load_references(
    db_conn: &mut SqliteConnection,
    database_dir: std::path::PathBuf,
    options: LoadAllOptions,
) -> LoadReferencesReturn {
    let mut collection_id = max_collection_id(db_conn);

    let mut reference_collections_available = 0;
    let mut reference_collections_loaded = 0;
    let mut reference_collections_skipped = 0;
    let mut reference_locations_found = 0;
    let mut reference_base_plants_added = 0;
    let mut reference_plants_added = 0;

    let git_info = GitModificationTimes::new(&database_dir.join("..")).unwrap();
    // git_info.print();

    // traverse /plant_database/references/
    // create a collections table entry for each location in this reference, or only one if there's only one location
    let mut reference_paths = WalkDir::new(database_dir.join("references"))
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|entry| entry.into_path())
        .filter(|path_| {
            fs::metadata(path_).unwrap().is_file()
                && path_.extension().and_then(|extension| extension.to_str()) == Some("json5")
        })
        .collect::<Vec<_>>();

    if options.reference_slice.is_some() || options.skip_loaded_references {
        reference_paths.sort();
    }

    let loaded_references = if options.skip_loaded_references {
        loaded_reference_keys(db_conn)
    } else {
        Default::default()
    };

    let started = Instant::now();

    for (reference_index, path_) in reference_paths.iter().enumerate() {
        if options
            .reference_slice
            .map(|slice| !slice.contains(reference_index))
            .unwrap_or_default()
        {
            continue;
        }

        reference_collections_available += 1;
        let filename = rem_last_n(path_.file_name().unwrap().to_str().unwrap(), ".json5".len());
        let path = format_path(path_.parent().unwrap().to_str().unwrap());
        let reference_key = (path.clone(), filename.to_string());

        if loaded_references.contains(&reference_key) {
            reference_collections_skipped += 1;
            continue;
        }

        println!("loading reference: {}", path_.display());
        reference_collections_loaded += 1;

        let absolute_path_git = fs::canonicalize(database_dir.join("..")).unwrap();
        let absolute_path_file = fs::canonicalize(path_).unwrap();
        let file_git_path = pathdiff::diff_paths(absolute_path_file, absolute_path_git).unwrap();

        let path_git_info = git_info.for_path(&file_git_path);
        if path_git_info.is_none() {
            println!("no git mod time for: {}", file_git_path.display());
        }

        let git_edit_time = path_git_info.map(|path_git_info| path_git_info.seconds());

        let contents = fs::read_to_string(path_).unwrap();

        let collection: CollectionJson = json5::from_str(&contents).unwrap_or_else(|error| {
            panic!("couldn't parse json in file {} {}", path_.display(), error);
        });

        let notoriety_info = notoriety::collection_notoriety_text_decoder(&collection.type_);

        collection_id += 1;

        let needs_help = if collection.needs_help.is_some() {
            collection.needs_help.unwrap()
        } else {
            false
        };

        let ignore_for_nearby_searches = collection.ignore_for_nearby_searches.unwrap_or_default();

        //    println!("inserting");
        let rows_inserted = diesel::insert_into(collections::dsl::collections)
            .values((
                collections::id.eq(collection_id),
                collections::git_edit_time.eq(git_edit_time),
                collections::path.eq(&path),
                collections::filename.eq(&filename),
                collections::title.eq(&collection.title),
                collections::author.eq(&collection.author),
                collections::description.eq(&collection.description),
                collections::url.eq(&collection.url),
                collections::published.eq(&collection.published),
                collections::reviewed.eq(&collection.reviewed),
                collections::accessed.eq(&collection.accessed),
                collections::needs_help.eq(needs_help as i32),
                collections::notoriety_type.eq(&collection.type_.to_lowercase()),
                collections::notoriety_score.eq(notoriety_info.score),
                collections::notoriety_score_explanation.eq(notoriety_info.explanation),
                collections::harvest_time_devalue_factor.eq(collection.harvest_time_devalue_factor),
                collections::ignore_for_nearby_searches.eq(ignore_for_nearby_searches as i32),
            ))
            .execute(db_conn);
        assert_eq!(Ok(1), rows_inserted);

        for category in collection.reference_categories.unwrap_or_default() {
            let category = category.trim();
            if category.is_empty() {
                panic!("empty reference category in {}", path_.display());
            }

            let rows_inserted =
                diesel::insert_into(collection_categories::dsl::collection_categories)
                    .values((
                        collection_categories::collection_id.eq(collection_id),
                        collection_categories::category.eq(category),
                    ))
                    .execute(db_conn);
            assert_eq!(Ok(1), rows_inserted);
        }

        for (i, location) in collection.locations.iter().enumerate() {
            //    println!("inserting");
            let rows_inserted = diesel::insert_into(locations::dsl::locations)
                .values((
                    locations::collection_id.eq(collection_id),
                    locations::location_number.eq((i + 1) as i32),
                    locations::location_name.eq(&location.name),
                    locations::latitude.eq(&location.latitude),
                    locations::longitude.eq(&location.longitude),
                    // stuff from the collection
                    locations::notoriety_score.eq(notoriety_info.score),
                    locations::collection_path.eq(&path),
                    locations::collection_filename.eq(&filename),
                    locations::collection_title.eq(&collection.title),
                    locations::ignore_for_nearby_searches.eq(ignore_for_nearby_searches as i32),
                ))
                .execute(db_conn);
            assert_eq!(Ok(1), rows_inserted);
            reference_locations_found += 1;
        }

        for plant in collection.plants {
            if plant.names.is_none() && plant.name.is_none() {
                panic!(r#"plant missing both "name" and "names" {:?}"#, plant)
            }
            if plant.names.is_some() && plant.name.is_some() {
                panic!(r#"plant has both "name" and "names" {:?}"#, plant)
            }

            let category_description = get_category_description(
                &plant.category,
                &plant.category_description,
                &collection.categories,
            );

            if plant.names.is_some() {
                for plant_name in plant.names.clone().unwrap() {
                    // multi-plant lists are used for extension guides that give, for example,
                    // a list of "all of the scab resistant apples" but don't tie that to one location
                    // or give descriptions for each apple
                    // we want to preserve the list so it can be displayed off in a corner or whatever
                    reference_base_plants_added += update_or_add_base_plant(
                        &plant_name,
                        &plant,
                        db_conn,
                        collection_id,
                        notoriety_info.score,
                        collection.ignore_unless_in_others,
                    );

                    reference_plants_added += add_collection_plant_by_location(
                        collection_id,
                        format!("{}{}", path, filename),
                        &plant_name,
                        &plant,
                        &category_description,
                        &collection.locations,
                        db_conn,
                    );
                }
            } else if plant.name.is_some() {
                // todo - if this plant has its own unique location (as seen in "list of elberta ripening dates")
                // then add this location first and then use it just for this plant

                reference_base_plants_added += update_or_add_base_plant(
                    plant.name.as_ref().unwrap(),
                    &plant,
                    db_conn,
                    collection_id,
                    notoriety_info.score,
                    collection.ignore_unless_in_others,
                );

                reference_plants_added += add_collection_plant_by_location(
                    collection_id,
                    format!("{}{}", path, filename),
                    plant.name.as_ref().unwrap(),
                    &plant,
                    &category_description,
                    &collection.locations,
                    db_conn,
                );
            }
        }

        if let Some(time_budget) = options.reference_load_time_budget {
            if reference_collections_loaded > 0 && started.elapsed() >= time_budget {
                break;
            }
        }
    }

    // plant category existince is checked later in check_database()

    // for each plant, create an entry in the collection_items database for each location, with a foreign key to that location's collections table entry
    LoadReferencesReturn {
        reference_collections_available,
        reference_collections_loaded,
        reference_collections_skipped,
        reference_locations_found,
        reference_base_plants_added,
        reference_plants_added,
    }
}

fn get_relative_days(
    plus_or_minus: &str,
    number: Result<f32, std::num::ParseFloatError>,
    weeks: bool,
) -> Option<i32> {
    if let Ok(number) = number {
        match plus_or_minus {
            "+" => {
                if weeks {
                    return Some((number * 7.0).round() as i32);
                } else {
                    return Some(number.round() as i32);
                }
            }
            "-" => {
                if weeks {
                    return Some((number * -7.0).round() as i32);
                } else {
                    return Some((-number).round() as i32);
                }
            }
            _ => return None,
        }
    }
    None
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct HarvestRelativeParsed {
    pub name: String,
    pub type_: Option<String>, // for types other than the base type, for example Nectarines typically use redhaven peach
    pub relative_days: i32,
}

fn parse_relative_harvest(input: &str) -> Option<HarvestRelativeParsed> {
    // first, see if there's a ':' character which would mean we have a type like "Peach:" at the beginning
    // this is optional and if omitted the type of the relative plant will be assumed to be the same as this plant
    let split = input.split(':').collect::<Vec<&str>>();
    let edited;
    let type_: Option<String>;
    if split.len() == 2 {
        type_ = Some(split[0].to_string());
        edited = split[1];
    } else {
        type_ = None;
        edited = input;
    }

    let relative_harvest_x_to_y_regex =
        Regex::new(r#"(.+)([-+])([0-9.]+)(?: to )([-+])([0-9.]+)(.*(?:week|Week))?"#).unwrap();
    let relative_harvest_regex = Regex::new(r#"(.+)([-+])([0-9.]+)(.*(?:week|Week))?"#).unwrap();

    if let Some(matches) = relative_harvest_x_to_y_regex.captures(edited) {
        let weeks;
        if matches.len() >= 7 {
            if let Some(week_match) = matches.get(6) {
                weeks = week_match.as_str().to_lowercase().trim() == "week";
            } else {
                weeks = false;
            }

            let mut output = HarvestRelativeParsed {
                name: matches[1].trim().to_string(),
                type_: type_.clone(),
                ..Default::default()
            };

            let plus_or_minus_1 = &matches[2];
            let number_1 = matches[3].parse::<f32>();
            let plus_or_minus_2 = &matches[4];
            let number_2 = matches[5].parse::<f32>();

            let relative_days_1 = get_relative_days(plus_or_minus_1, number_1, weeks);
            let relative_days_2 = get_relative_days(plus_or_minus_2, number_2, weeks);

            if let (Some(relative_days_1), Some(relative_days_2)) =
                (relative_days_1, relative_days_2)
            {
                output.relative_days = (relative_days_1 + relative_days_2) / 2;
                return Some(output);
            }
        }
    }

    if let Some(matches) = relative_harvest_regex.captures(edited) {
        let weeks;
        if matches.len() >= 5 {
            if let Some(week_match) = matches.get(4) {
                weeks = week_match.as_str().to_lowercase().trim() == "week";
            } else {
                weeks = false;
            }

            let mut output = HarvestRelativeParsed {
                name: matches[1].trim().to_string(),
                type_,
                ..Default::default()
            };

            let plus_or_minus = &matches[2];
            let number = matches[3].parse::<f32>();

            let relative_days = get_relative_days(plus_or_minus, number, weeks);
            if let Some(relative_days) = relative_days {
                output.relative_days = relative_days;
                return Some(output);
            }
        }
    }

    None
}

fn add_relative_value(base_value: Option<i32>, adjustment: i32) -> Option<i32> {
    if let Some(base_value) = base_value {
        return Some(base_value + adjustment);
    }
    None
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
struct FactJson {
    contributor: String,
    fact: String,
    reference: String,
}

pub fn load_facts(db_conn: &mut SqliteConnection, database_dir: std::path::PathBuf) -> isize {
    let mut facts_found = 0;

    let contents = fs::read_to_string(database_dir.join("facts.json5")).unwrap();

    let facts: Vec<FactJson> = json5::from_str(&contents).unwrap();

    for fact in &facts {
        let rows_inserted = diesel::insert_into(facts::dsl::facts)
            .values((
                facts::contributor.eq(&fact.contributor),
                facts::fact.eq(&fact.fact),
                facts::reference.eq(&fact.reference),
            ))
            .execute(db_conn);
        assert_eq!(Ok(1), rows_inserted);
        facts_found += 1;
    }

    facts_found
}

fn remove_ignored_base_plants(db_conn: &mut SqliteConnection) {
    let _ = diesel::delete(
        base_plants::dsl::base_plants.filter(base_plants::ignore_unless_in_others.eq(1)),
    )
    .execute(db_conn);

    // todo - should we also delete the collection items? I think no, they could be nice to look at in the specific references
}

#[skip_serializing_none]
#[derive(Serialize, Queryable)]
pub struct CollectionItemRelative {
    pub collection_item_id: i32,
    pub location_id: Option<i32>,
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,

    pub harvest_relative: Option<String>,
    pub harvest_start: Option<i32>,
}

fn relative_to_absolute_harvest_times(db_conn: &mut SqliteConnection) {
    // look for all plants with only a relative harvest time and try to fill in their absolute times
    // example is an extension publication listing peaches as redhaven+5 or whatever,
    // but also giving an absolute time for redhaven in the same pub

    let all_plants = collection_items::dsl::collection_items
        .filter(collection_items::user_id.is_null())
        .select((
            collection_items::id,
            collection_items::location_id,
            collection_items::type_,
            collection_items::name,
            collection_items::harvest_relative,
            collection_items::harvest_start,
        ))
        .load::<CollectionItemRelative>(db_conn)
        .unwrap();

    // if harvest_start is unset and harvest_relative is set, parse harvest_relative
    // and see if it refers to another plant in the same location. if so, create absolute dates
    // relative to the base plant and store those

    for plant in all_plants {
        if plant.harvest_relative.is_some() && plant.harvest_start.is_none() {
            if let Some(harvest_relative) = parse_relative_harvest(&plant.harvest_relative.unwrap())
            {
                // look for this variety name in the same location
                // todo: look at harvest_relative's "type" field in case it's pointing across types

                if let Ok(relative_plant) = collection_items::dsl::collection_items
                    .filter(collection_items::location_id.eq(plant.location_id))
                    .filter(collection_items::name.eq(harvest_relative.name))
                    .filter(collection_items::type_.eq(plant.type_))
                    .first::<CollectionItem>(db_conn)
                {
                    let harvest_start = add_relative_value(
                        relative_plant.harvest_start,
                        harvest_relative.relative_days,
                    );
                    let harvest_end = add_relative_value(
                        relative_plant.harvest_end,
                        harvest_relative.relative_days,
                    );
                    let harvest_start_2 = add_relative_value(
                        relative_plant.harvest_start_2,
                        harvest_relative.relative_days,
                    );
                    let harvest_end_2 = add_relative_value(
                        relative_plant.harvest_end_2,
                        harvest_relative.relative_days,
                    );

                    let updated_row_count = diesel::update(
                        collection_items::dsl::collection_items
                            .filter(collection_items::id.eq(plant.collection_item_id)),
                    )
                    .set((
                        collection_items::harvest_start.eq(harvest_start),
                        collection_items::harvest_end.eq(harvest_end),
                        collection_items::harvest_start_2.eq(harvest_start_2),
                        collection_items::harvest_end_2.eq(harvest_end_2),
                    ))
                    .execute(db_conn);
                    assert_eq!(Ok(1), updated_row_count);
                }
            }
        }
    }
}

fn calculate_years_from_patent(db_conn: &mut SqliteConnection) {
    // todo - for each base plant, if the release year isn't filled in, guess at it from the patent number if available

    // put in a note in a new column about how the release year was guessed at

    let all_plants = base_plants::dsl::base_plants
        .load::<BasePlant>(db_conn)
        .unwrap();

    for plant in all_plants {
        // if release year is unset but patent number is set, fill in release year from a patent->year table

        if plant.release_year.is_none() && plant.uspp_number.is_some() {
            let updated_row_count =
                diesel::update(base_plants::dsl::base_plants.filter(base_plants::id.eq(plant.id)))
                    .set((
                        base_plants::release_year.eq(util::uspp_number_to_release_year(
                            plant.uspp_number.clone().unwrap().parse::<i32>().unwrap(),
                        )),
                        base_plants::release_year_note.eq("derived from patent number"),
                    ))
                    .execute(db_conn);
            assert_eq!(Ok(1), updated_row_count);
        }

        // estimate patent expiration
        if plant.uspp_expiration.is_none() && plant.uspp_number.is_some() {
            let updated_row_count =
                diesel::update(base_plants::dsl::base_plants.filter(base_plants::id.eq(plant.id)))
                    .set((
                        base_plants::uspp_expiration.eq(util::uspp_number_to_expiration(
                            plant.uspp_number.unwrap().parse::<i32>().unwrap(),
                        )),
                        base_plants::uspp_expiration_estimated.eq(1),
                    ))
                    .execute(db_conn);
            assert_eq!(Ok(1), updated_row_count);
        }
    }
}

fn add_marketing_names(db_conn: &mut SqliteConnection) {
    // for each collection item, look up the base plant and copy in the marketing name if set
    let all_collection_items = collection_items::dsl::collection_items
        .filter(collection_items::user_id.is_null())
        .load::<CollectionItem>(db_conn)
        .unwrap();

    for collection_item in &all_collection_items {
        if let Ok(base_plant) = base_plants::dsl::base_plants
            .filter(base_plants::name.eq(&collection_item.name))
            .filter(base_plants::type_.eq(&collection_item.type_))
            .first::<BasePlant>(db_conn)
        {
            let updated_row_count = diesel::update(
                collection_items::dsl::collection_items
                    .filter(collection_items::id.eq(collection_item.id)),
            )
            .set((collection_items::marketing_name.eq(base_plant.marketing_name),))
            .execute(db_conn);
            assert_eq!(Ok(1), updated_row_count);
        }
    }
}

fn set_relative_vs_existing(
    db_conn: &mut SqliteConnection,
    plant: &CollectionItem,
    base_plant: &CollectionItem,
    round: f64,
    new_relative_harvest: i32,
    explanation_text_prefix: String,
) {
    // todo - get a better absolute date that just looking at harvest start (incorporate "is midpoint" if available)

    let updated_row_count = diesel::update(
        collection_items::dsl::collection_items.filter(collection_items::id.eq(plant.id)),
    )
    .set((
        collection_items::calc_harvest_relative.eq(new_relative_harvest),
        collection_items::calc_harvest_relative_to.eq(base_plant.calc_harvest_relative_to.clone()),
        collection_items::calc_harvest_relative_to_type
            .eq(base_plant.calc_harvest_relative_to_type.clone()),
        collection_items::calc_harvest_relative_round.eq(round), // set in the 0th round of searches
        collection_items::calc_harvest_relative_explanation.eq(format!(
            "{explanation_text_prefix}relative to {} in this location",
            base_plant.name
        )),
    ))
    .execute(db_conn);
    assert_eq!(Ok(1), updated_row_count);
}

fn calculate_relative_harvest_times(db_conn: &mut SqliteConnection) {
    let all_plants = collection_items::dsl::collection_items
        .filter(collection_items::user_id.is_null())
        .load::<CollectionItem>(db_conn)
        .unwrap();

    // if harvest_start is unset and harvest_relative is set, parse harvest_relative
    // and see if it refers to another plant in the same location. if so, create absoluate dates
    // relative to the base plant and store those

    for plant in all_plants {
        if util::is_standard_candle(&plant.type_, &plant.name) {
            // make sure the standard candles themselves get tagged as +0 in the 0th round
            // (even if they don't have a relative harvest field)

            let updated_row_count = diesel::update(
                collection_items::dsl::collection_items.filter(collection_items::id.eq(plant.id)),
            )
            .set((
                collection_items::calc_harvest_relative.eq(0),
                collection_items::calc_harvest_relative_to.eq(plant.name),
                collection_items::calc_harvest_relative_to_type.eq(plant.type_.clone()),
                collection_items::calc_harvest_relative_round.eq(0.0), // set in the 0th round of searches
                collection_items::calc_harvest_relative_explanation.eq("standard candle"),
            ))
            .execute(db_conn);
            assert_eq!(Ok(1), updated_row_count);
        }
        if plant.harvest_relative.is_some() {
            if let Some(harvest_relative) = parse_relative_harvest(&plant.harvest_relative.unwrap())
            {
                // round 0: for each collection item, see if it has an imported harvest_relative field, and check that against
                // the list of standard candles using type_to_standard_candle() and parse the days/weeks. then put an integer
                // into the calculated relative harvest column, and put a note that it was a direct parse

                let type_maybe_parsed = if let Some(type_) = harvest_relative.type_ {
                    type_ // type overridden, maybe this is a nectarine referencing redhaving peach or something
                } else {
                    plant.type_ // no type in harvest_relative field, use the type of the plant
                };

                if util::is_standard_candle(&type_maybe_parsed, &harvest_relative.name) {
                    let updated_row_count = diesel::update(
                        collection_items::dsl::collection_items
                            .filter(collection_items::id.eq(plant.id)),
                    )
                    .set((
                        collection_items::calc_harvest_relative.eq(harvest_relative.relative_days),
                        collection_items::calc_harvest_relative_to.eq(harvest_relative.name),
                        collection_items::calc_harvest_relative_to_type.eq(type_maybe_parsed),
                        collection_items::calc_harvest_relative_round.eq(0.0), // set in the 0th round of searches
                        collection_items::calc_harvest_relative_explanation
                            .eq("parsed directly from collection entry"),
                    ))
                    .execute(db_conn);
                    assert_eq!(Ok(1), updated_row_count);
                }
            }
        }
    }

    for round in 1..=10 {
        println!("relative harvest inference round {round}");

        let all_plants = collection_items::dsl::collection_items
            .filter(collection_items::user_id.is_null())
            .filter(collection_items::calc_harvest_relative.is_null())
            .load::<CollectionItem>(db_conn)
            .unwrap();

        let mut num_inferred = 0;
        for plant in all_plants {
            if plant.calc_harvest_relative.is_none() && plant.harvest_start.is_some() {
                if let Some(candle) = util::type_to_standard_candle(&plant.type_) {
                    // round 2-N: for every collection item, if it has an absolute time but not a relative time,
                    // see if its collection includes any same-type with an absolute time AND a relative time
                    // if it does, calculate a relative time based on asbsolute1 - absolute2 + relative2
                    // create a note that it was calculated this way
                    // todo

                    // first look for the candle itself, prefer that
                    // I don't think I can remove this with an order-by-round thing, unless I make standard candles 0.0 and parsed directly 0.1 or something
                    let standard_candle = collection_items::dsl::collection_items
                        .filter(collection_items::user_id.is_null())
                        .filter(collection_items::location_id.eq(plant.location_id))
                        .filter(collection_items::harvest_start.is_not_null())
                        .filter(collection_items::name.eq(&candle.name))
                        .filter(collection_items::type_.eq(&candle.type_))
                        .first::<CollectionItem>(db_conn);

                    if let Ok(standard_candle) = standard_candle {
                        set_relative_vs_existing(
                            db_conn,
                            &plant,
                            &standard_candle,
                            round as f64,
                            plant.harvest_start.unwrap() - standard_candle.harvest_start.unwrap()
                                + standard_candle.calc_harvest_relative.unwrap(), // todo - account for midpoints
                            "".to_string(),
                        );
                        num_inferred += 1;
                        continue;
                    }

                    // if we don't have a standard candle then just pick any same-location plant
                    let alternative_relative_plant = collection_items::dsl::collection_items
                        .filter(collection_items::user_id.is_null())
                        .filter(collection_items::location_id.eq(plant.location_id))
                        .filter(collection_items::calc_harvest_relative_to.eq(&candle.name))
                        .filter(collection_items::calc_harvest_relative_to_type.eq(&candle.type_))
                        .filter(collection_items::harvest_start.is_not_null())
                        .filter(collection_items::calc_harvest_relative.is_not_null())
                        .order(collection_items::calc_harvest_relative_round.asc()) // best results first
                        .first::<CollectionItem>(db_conn);

                    if let Ok(alternative_relative_plant) = alternative_relative_plant {
                        set_relative_vs_existing(
                            db_conn,
                            &plant,
                            &alternative_relative_plant,
                            round as f64 + 0.1, // 0.1: give a small penalty for it not being a standard candle reference
                            plant.harvest_start.unwrap()
                                - alternative_relative_plant.harvest_start.unwrap()
                                + alternative_relative_plant.calc_harvest_relative.unwrap(), // todo - account for midpoints
                            "".to_string(),
                        );
                        num_inferred += 1;
                        continue;
                    }
                }
            }
        }

        // we need a 2nd loop so we can pick up plants that have already been tagged as standard candles above
        // I think this should only be run in round 1 as it won't pick up anything additional later, it relies on harvest_relative fields only
        let all_plants2 = collection_items::dsl::collection_items
            .filter(collection_items::user_id.is_null())
            .filter(collection_items::calc_harvest_relative.is_null())
            .load::<CollectionItem>(db_conn)
            .unwrap();

        'outer: for plant in all_plants2 {
            // a special case for collections which use a different relative plant (like vaughn nursery using elberta for peaches, when we'd like to use redhaven)
            // if we have a calculated relative harvest tagged into that collection (as we will in round 1 for redhaven)
            // then when we compare a variety to redhaven in that collection, detect that redhaven has elberta-28 and allow elberta+/- for the others in that collection
            if plant.harvest_relative.is_some() {
                if let Some(harvest_relative) =
                    parse_relative_harvest(&plant.harvest_relative.clone().unwrap())
                {
                    // todo - make sure this also works for the euro burlat cherries, etc.
                    let alt_references = collection_items::dsl::collection_items
                        .filter(collection_items::user_id.is_null())
                        .filter(collection_items::location_id.eq(plant.location_id))
                        .filter(collection_items::harvest_relative.is_not_null())
                        .filter(collection_items::calc_harvest_relative.is_not_null())
                        .filter(collection_items::calc_harvest_relative_to.is_not_null())
                        .filter(collection_items::calc_harvest_relative_to_type.is_not_null())
                        .order(collection_items::calc_harvest_relative_round.asc()) // this will prefer standard candles and higher quality things in general
                        .load::<CollectionItem>(db_conn)
                        .unwrap();

                    // see if any of these has a harvest_relative that parses to the same thing as our plant in question
                    for alt_reference in alt_references {
                        if let Some(alt_reference_harvest_relative) =
                            parse_relative_harvest(&alt_reference.harvest_relative.clone().unwrap())
                        {
                            if alt_reference_harvest_relative.name == harvest_relative.name
                                && alt_reference_harvest_relative.type_ == harvest_relative.type_
                            {
                                set_relative_vs_existing(
                                    db_conn,
                                    &plant,
                                    &alt_reference,
                                    round as f64,
                                    // example: we're importing elberta which is "elberta+0"
                                    // we already have redhaven, tagged as a standard candle "redhaven+0", but it's "elberta-42" in the reference
                                    // we want to label elberta as "redhaven+42" so we take "redhaven+0", add "elberta+0", and subtract "elberta-42" => "redhaven+42"
                                    alt_reference.calc_harvest_relative.unwrap()
                                        + harvest_relative.relative_days
                                        - alt_reference_harvest_relative.relative_days,
                                    "relative-to-relative: ".to_string(),
                                );
                                num_inferred += 1;
                                continue 'outer;
                            }
                        }
                    }
                }
            }
        }

        // todo: if any plants are STILL untagged, and have a harvest_relative field but no calced fields,
        // see if the base plant value can be used to fill in their calced fields
        // this would rely on the base plant calc from the previous round
        let un_calced_plants = collection_items::dsl::collection_items
            .filter(collection_items::user_id.is_null())
            .filter(collection_items::calc_harvest_relative.is_null())
            .filter(collection_items::harvest_relative.is_not_null())
            .load::<CollectionItem>(db_conn)
            .unwrap();

        for un_calced_plant in un_calced_plants {
            // if this plant's harvest_time_relative parses
            if let Some(harvest_relative) =
                parse_relative_harvest(&un_calced_plant.harvest_relative.clone().unwrap())
            {
                // if base_plants has this name+type and has calc_harvest_relative, use it to get a value here
                // todo: probably move this logic into a helper and apply it everywhere parse_relative_harvest() is used
                let type_maybe_parsed = if let Some(type_) = harvest_relative.type_ {
                    type_ // type overridden, maybe this is a nectarine referencing redhaving peach or something
                } else {
                    un_calced_plant.type_.clone() // no type in harvest_relative field, use the type of the plant
                };

                // look for the thing pointed to by this orphaned relative harvest text. if found, use its value pointing to the standard candle
                // to get an orphan->standard candle value
                let base_plant = base_plants::dsl::base_plants
                    .filter(base_plants::name.eq(&harvest_relative.name))
                    .filter(base_plants::type_.eq(&type_maybe_parsed))
                    .first::<BasePlant>(db_conn);

                // the thing pointed to in the relative harvest text might not exist
                if let Ok(base_plant) = base_plant {
                    if base_plant.harvest_relative.is_some()
                        && base_plant.harvest_relative_to.is_some()
                        && base_plant.harvest_relative_to_type.is_some()
                    {
                        // we have a value for "relative to this base plant"
                        // and then we have a value for "base plant relative to standard candle"
                        // so we just add the two: uncalced -> base plant -> standard candle

                        let updated_row_count = diesel::update(
                            collection_items::dsl::collection_items
                                .filter(collection_items::id.eq(un_calced_plant.id)),
                        )
                        .set((
                            collection_items::calc_harvest_relative
                                .eq(base_plant.harvest_relative.unwrap()
                                    + harvest_relative.relative_days),
                            collection_items::calc_harvest_relative_to
                                .eq(base_plant.harvest_relative_to.unwrap()),
                            collection_items::calc_harvest_relative_to_type
                                .eq(base_plant.harvest_relative_to_type.unwrap()),
                            collection_items::calc_harvest_relative_round.eq(round as f64 + 0.95),
                            collection_items::calc_harvest_relative_explanation
                                .eq(&format!("round {round}: set vs. base plant average")
                                    .to_string()),
                        ))
                        .execute(db_conn)
                        .expect("updating collection items with calculated harvest relative");
                        assert_eq!(1, updated_row_count);
                        num_inferred += 1;
                    }
                }
            }
        }

        // next:
        // 1. calc the best time for each base plant from the existing relative values
        // 2. propagate the base plant values out to collections for any plants that are as-yet unmarked, and mark them as such
        // 3. end of round, repeat for N times (need to test to see how many rounds make sense)

        let base_plants = base_plants::dsl::base_plants
            .load::<BasePlant>(db_conn)
            .unwrap();

        for base_plant in base_plants {
            if let Some(calculated) =
                calculate_relative_harvest_from_references(&base_plant, round, db_conn)
            {
                let updated_row_count = diesel::update(
                    base_plants::dsl::base_plants.filter(base_plants::id.eq(base_plant.id)),
                )
                .set((
                    base_plants::harvest_relative.eq(calculated.harvest_relative),
                    base_plants::harvest_relative_to.eq(calculated.harvest_relative_to.clone()),
                    base_plants::harvest_relative_to_type
                        .eq(calculated.harvest_relative_to_type.clone()),
                    base_plants::harvest_relative_explanation
                        .eq(calculated.harvest_relative_explanation.clone()),
                ))
                .execute(db_conn);
                assert_eq!(Ok(1), updated_row_count);
                num_inferred += 1;

                // todo: also set this on all previously un-set collection items (of this type+name)
                // and note the round as round + offset to devalue it vs. simpler forms
                let _updated_row_count = diesel::update(
                    collection_items::dsl::collection_items
                        .filter(collection_items::user_id.is_null())
                        .filter(collection_items::type_.eq(base_plant.type_))
                        .filter(collection_items::name.eq(base_plant.name))
                        .filter(collection_items::calc_harvest_relative.is_null()),
                )
                .set((
                    collection_items::calc_harvest_relative.eq(calculated.harvest_relative),
                    collection_items::calc_harvest_relative_to.eq(calculated.harvest_relative_to),
                    collection_items::calc_harvest_relative_to_type
                        .eq(calculated.harvest_relative_to_type),
                    collection_items::calc_harvest_relative_round.eq(round as f64 + 0.9),
                    collection_items::calc_harvest_relative_explanation
                        .eq(calculated.harvest_relative_explanation),
                ))
                .execute(db_conn)
                .expect("updating collection items with calculated harvest relative");
                // the above query will commonly affect zero rows - don't check return row count
            }
        }

        println!("added {num_inferred} relative values in round {round}");
        // todo - maybe quit early if num_inferred is 0 for this round?
        if num_inferred == 0 {
            println!("ending early because the previous round had 0 changes");
            break;
        }
    }

    //     todo: or associated-type plant (like nectarine->peach) with an absolute time
}

struct RelativeHarvest {
    pub harvest_relative: i32,
    pub harvest_relative_to: String,
    pub harvest_relative_to_type: String,
    pub harvest_relative_explanation: String,
}

fn calculate_relative_harvest_from_references(
    base_plant: &BasePlant,
    current_round: i32,
    db_conn: &mut SqliteConnection,
) -> Option<RelativeHarvest> {
    // get all collection items matching this type+name
    let all_references = collection_items::dsl::collection_items
        .filter(collection_items::user_id.is_null())
        .filter(collection_items::type_.eq(&base_plant.type_))
        .filter(collection_items::name.eq(&base_plant.name))
        .load::<CollectionItem>(db_conn)
        .unwrap();

    let mut sum = 0.0;
    let mut divisor = 0.0;
    let mut any_reference_updated = false;
    let mut num_references_used = 0;
    let mut harvest_relative_explanation = format!("round {current_round}: ");

    if let Some(candle) = util::type_to_standard_candle(&base_plant.type_) {
        for reference in all_references {
            // if their calculated collection stuff matches the standard candle for this type,
            if reference.calc_harvest_relative.is_some()
                && reference.calc_harvest_relative_to == Some(candle.name.clone())
                && reference.calc_harvest_relative_to_type == Some(candle.type_.clone())
                && reference.calc_harvest_relative_round.is_some()
            {
                if reference.calc_harvest_relative_round.unwrap() >= current_round as f64 {
                    any_reference_updated = true; // todo - we could do one loop looking for updated references then a 2nd loop to do the calculations if we have any. could save a little time
                }

                num_references_used += 1;

                let notoriety_and_devalue_score =
                    get_notoriety_and_devalue_scores(reference.collection_id.unwrap(), db_conn);
                let devalue_score = notoriety_and_devalue_score
                    .harvest_time_devalue_factor
                    .unwrap_or(0.0);

                // add the value to an average based on a weight of the round score * the reference notoriety score
                // the devalue_score get treated as being additional rounds behind
                let round_score = 1.0
                    / (2.0_f64).powf(
                        reference.calc_harvest_relative_round.unwrap() + devalue_score as f64,
                    ); // round 0: 1, round 1: .5, round 2: .25 etc. (give a sharply reduced score as rounds progress)

                let overall_score =
                    notoriety_and_devalue_score.notoriety_score as f64 * round_score; // todo - maybe copy notoriety into each reference to save time here?

                sum += reference.calc_harvest_relative.unwrap() as f64 * overall_score;
                divisor += overall_score;

                harvest_relative_explanation += &format!(
                    " [{:?}] {} ({:.2}*{:.2})",
                    reference.collection_id,
                    reference.calc_harvest_relative.unwrap(),
                    notoriety_and_devalue_score.notoriety_score,
                    round_score,
                )
                .to_string();

                // then figure the average and return it
                // create a summary of the average calculation. [L16]: -23 (93*.5) [L17]: -33 (93*.5) etc. with some length cutoff
                // only run this 1. in the first round or 2. if one of the collection items has been updated in the current round
                // that could help avoid progressive averaging of the base plant's date without having new data
            }
        }

        if num_references_used > 0 && (current_round <= 1 || any_reference_updated) {
            return Some(RelativeHarvest {
                harvest_relative: (sum / divisor).round() as i32,
                harvest_relative_to: candle.name.clone(),
                harvest_relative_to_type: candle.type_.clone(),
                harvest_relative_explanation,
            });
        } else {
            return None;
        }
    }

    None
}

#[skip_serializing_none]
#[derive(Serialize, Queryable, Debug)]
pub struct NotorietyAndDevalueScores {
    pub notoriety_score: f32,
    pub harvest_time_devalue_factor: Option<f32>,
}

fn get_notoriety_and_devalue_scores(
    collection_id: i32,
    db_conn: &mut SqliteConnection,
) -> NotorietyAndDevalueScores {
    collections::dsl::collections
        .select((
            collections::notoriety_score,
            collections::harvest_time_devalue_factor,
        ))
        .filter(collections::id.eq(collection_id))
        .first::<NotorietyAndDevalueScores>(db_conn)
        .unwrap_or_else(|_| panic!("no collection found {}", collection_id))
}

#[skip_serializing_none]
#[derive(Serialize, Queryable, Debug)]
pub struct BasePlantsItemForDedupe {
    pub name_fts: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub aka_fts: Option<String>,
}

// for all base plants, ensure none of the names match an "AKA" name which would be a duplicate
// a more stringent check could look at the full text search versions of the names (lower case without special characters)
fn check_aka_duplicates(db_conn: &mut SqliteConnection) {
    let all_plant_types = plant_types::dsl::plant_types
        .load::<PlantType>(db_conn)
        .unwrap();

    let mut aka_map = HashMap::new();

    for plant_type in all_plant_types {
        aka_map.insert(plant_type.name, HashSet::<String>::new());
    }

    // read all AKA entries into a map of plant type -> set of AKA names
    let all_base_plants = base_plants::dsl::base_plants
        .select((
            base_plants::name_fts,
            base_plants::type_,
            base_plants::aka_fts,
        ))
        .load::<BasePlantsItemForDedupe>(db_conn)
        .unwrap();

    let mut warning_count = 0;

    for plant in &all_base_plants {
        if let Some(plant_aka_fts) = plant.aka_fts.clone() {
            for aka in decode_aka_string(&plant_aka_fts) {
                if aka.is_empty() {
                    continue;
                }
                if !aka_map
                    .get_mut(&plant.type_)
                    .unwrap()
                    .insert(aka.to_string())
                {
                    warning_count += 1;
                    println!("warning: found a duplicate AKA entry {:?}", plant)
                }
            }
        }
    }

    // then for each base plant names make sure there isn't an AKA name in the set in that type
    for plant in &all_base_plants {
        if aka_map.get(&plant.type_).unwrap().contains(&plant.name_fts) {
            warning_count += 1;
            println!("warning: found a plant named with an AKA entry {:?}", plant)
        }
    }

    if warning_count > 0 {
        println!("warning: found {warning_count} AKA consistency issues")
    }
}

// put the base plants ID into each collection item that matches - simplifies later queries
fn add_base_id_to_collection_items(db_conn: &mut SqliteConnection) {
    let base_plants = base_plants::dsl::base_plants
        .load::<BasePlant>(db_conn)
        .unwrap();

    for plant in &base_plants {
        let _updated_row_count = diesel::update(
            collection_items::dsl::collection_items
                .filter(collection_items::user_id.is_null())
                .filter(collection_items::type_.eq(&plant.type_))
                .filter(collection_items::name.eq(&plant.name)),
        )
        .set((collection_items::base_plant_id.eq(plant.id),))
        .execute(db_conn)
        .expect("updating collection items with base plant id");
    }
}

fn check_database(db_conn: &mut SqliteConnection) {
    // find all types and make sure each is in the types table
    let types_from_plants = base_plants::dsl::base_plants
        .select(base_plants::type_)
        .distinct()
        .load::<String>(db_conn);

    for type_from_plants in &types_from_plants.unwrap() {
        let _ = plant_types::dsl::plant_types
            .filter(plant_types::name.eq(type_from_plants))
            .first::<PlantType>(db_conn)
            .unwrap_or_else(|_| {
                panic!(
                    r#"imported a plant with a category not in types.json: "{}""#,
                    type_from_plants
                )
            });
    }

    check_aka_duplicates(db_conn);
}

fn rebuild_fts(db_conn: &mut SqliteConnection) {
    let _result1 =
        diesel::sql_query("INSERT INTO fts_base_plants(fts_base_plants) VALUES('rebuild')")
            .execute(db_conn);

    let _result2 =
        diesel::sql_query("INSERT INTO fts_base_plants(fts_base_plants) VALUES('optimize')")
            .execute(db_conn);
}

fn calculate_notoriety(db_conn: &mut SqliteConnection) {
    let all_base_plants = base_plants::dsl::base_plants
        .load::<BasePlant>(db_conn)
        .unwrap();

    let current_year = chrono::Utc::now().year();
    for plant in &all_base_plants {
        let notoriety_score =
            notoriety::base_plant_notoriety_calc(&notoriety::BasePlantNotorietyInput {
                notoriety_highest_collection_score: plant.notoriety_highest_collection_score,
                notoriety_highest_collection_score_name: get_collection_name(
                    plant.notoriety_highest_collection_score_id,
                    db_conn,
                ),
                current_year,
                release_year: plant.release_year,
                number_of_references: plant.number_of_references,
                uspp_number: plant.uspp_number.as_ref(),
            });

        let updated_row_count =
            diesel::update(base_plants::dsl::base_plants.filter(base_plants::id.eq(plant.id)))
                .set((
                    base_plants::notoriety_score.eq(notoriety_score.score),
                    base_plants::notoriety_score_explanation.eq(notoriety_score.explanation),
                ))
                .execute(db_conn);
        assert_eq!(Ok(1), updated_row_count);
    }
}

pub fn count_base_plants(db_conn: &mut SqliteConnection) -> i64 {
    base_plants::dsl::base_plants
        .select(diesel::dsl::count(base_plants::id))
        .first(db_conn)
        .unwrap()
}

fn count_facts(db_conn: &mut SqliteConnection) -> i64 {
    facts::dsl::facts
        .select(diesel::dsl::count(facts::id))
        .first(db_conn)
        .unwrap()
}

fn count_plant_types(db_conn: &mut SqliteConnection) -> i64 {
    plant_types::dsl::plant_types
        .select(diesel::dsl::count(plant_types::id))
        .first(db_conn)
        .unwrap()
}

fn count_reference_collections(db_conn: &mut SqliteConnection) -> i64 {
    collections::dsl::collections
        .select(diesel::dsl::count(collections::id))
        .first(db_conn)
        .unwrap()
}

fn max_collection_id(db_conn: &mut SqliteConnection) -> i32 {
    collections::dsl::collections
        .select(diesel::dsl::max(collections::id))
        .first::<Option<i32>>(db_conn)
        .unwrap()
        .unwrap_or(0)
}

fn loaded_reference_keys(db_conn: &mut SqliteConnection) -> HashSet<(String, String)> {
    collections::dsl::collections
        .select((collections::path, collections::filename))
        .load::<(String, String)>(db_conn)
        .unwrap()
        .into_iter()
        .collect()
}

pub fn calculate_and_write_relative_day_offsets(db_conn: &mut SqliteConnection) {
    // looks at the database and tries to figure out the relative days between all of the standard candles,
    // using locations that have multiple candles to do the math
    // it starts with the location with the most candles and adds more candle->candle times as it steps through other locations
    // the new gaps are averaged with previous gaps. the whole thing is pretty naive but it works ok
    // because of the naivete, we use sorting and IndexMap<> to keep some order and get a more repeatable result

    #[derive(Debug, Default)]
    struct AverageOffset {
        pub sum: f64,
        pub divisor: f64,
    }

    impl std::fmt::Display for AverageOffset {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                f,
                "sum {:.1} divisor {:.1} average {:.1}",
                self.sum,
                self.divisor,
                self.sum / self.divisor
            )
        }
    }

    #[derive(Debug, Default, Clone, Copy)]
    struct AverageDay {
        pub sum: i32,
        pub divisor: i32,
        pub average: f64,
        pub used: bool,
    }

    #[derive(Debug, Default, Clone)]
    struct Location {
        pub _location_id: i32,
        pub averages: IndexMap<util::Candle, AverageDay>,
    }

    let candle_list = util::get_standard_candles();
    let mut location_averages = IndexMap::new();
    let mut all_locations = Vec::new();
    for value in candle_list {
        location_averages.insert(value, AverageDay::default());
    }

    fn print_candles(candles_output: &IndexMap<util::Candle, AverageOffset>) {
        for candle in candles_output {
            println!("{:?} {}", candle.0, candle.1);
        }
    }

    // for each location, get the average absolute day for each standard candle and read it into a memory structure
    // (this is based on a plant entry having both a relative-to-candle entry and an absolute date of its own, and taking the difference)
    // (in most locations these will all be the same date because of the way the relative days were derived in the first place)
    let location_ids = collection_items::dsl::collection_items
        .filter(collection_items::user_id.is_null())
        .filter(collection_items::location_id.is_not_null())
        .select(collection_items::location_id)
        .distinct()
        .order(collection_items::location_id.asc())
        .load::<Option<i32>>(db_conn)
        .unwrap();

    for location_id in location_ids {
        let plants_in_location = collection_items::dsl::collection_items
            .filter(collection_items::location_id.eq(location_id))
            .order(collection_items::type_.asc())
            .then_order_by(collection_items::name.asc())
            .load::<CollectionItem>(db_conn)
            .unwrap();

        let mut this_location_averages = IndexMap::new();

        for plant in plants_in_location {
            if plant.calc_harvest_relative_to.is_some()
                && plant.calc_harvest_relative_to_type.is_some()
                && plant.calc_harvest_relative.is_some()
                && plant.harvest_start.is_some()
            {
                let candle = util::Candle {
                    type_: plant.type_,
                    name: plant.name,
                };
                if location_averages.contains_key(&candle) {
                    if !this_location_averages.contains_key(&candle) {
                        this_location_averages.insert(candle.clone(), AverageDay::default());
                    }

                    let entry = this_location_averages.get_mut(&candle).unwrap();
                    entry.sum +=
                        plant.harvest_start.unwrap() - plant.calc_harvest_relative.unwrap();
                    entry.divisor += 1;
                }
            }
        }

        for average in this_location_averages.values_mut() {
            average.average = average.sum as f64 / average.divisor as f64;
        }

        if !this_location_averages.is_empty() {
            all_locations.push(Location {
                _location_id: location_id.unwrap(),
                averages: this_location_averages.clone(),
            });
        }
    }

    // sort locations by number of candle entries and process the most first, it'll be a scaffold for the rest
    all_locations.sort_by(|a, b| {
        b.averages.len().cmp(&a.averages.len()) // b.cmp(a) will sort most-first
    });

    println!("{:#?}", all_locations);

    let mut candles_output = IndexMap::new();
    let mut initial_values_set = false;

    for round in 1..=10 {
        println!("relative->relative round {}", round);
        let mut num_changed = 0;

        for location in all_locations.iter_mut() {
            if !initial_values_set {
                initial_values_set = true;
                for (candle, average_day) in location.averages.iter_mut() {
                    candles_output.insert(
                        candle.clone(),
                        AverageOffset {
                            sum: average_day.average,
                            divisor: 1.0,
                        },
                    );
                    average_day.used = true;
                }
                continue;
            }

            let mut first_location_average = None;
            for average in location.averages.iter_mut() {
                if first_location_average.is_none() {
                    if !average.1.used {
                        first_location_average = Some(average);
                    }
                    continue;
                }

                if average.1.used {
                    continue;
                }

                // if we get here we have a pair of unused relative times
                let candle_a = first_location_average.as_ref().unwrap().0;
                let average_a = &first_location_average.as_ref().unwrap().1;
                let candle_b = average.0;
                let average_b = average.1;

                if candles_output.contains_key(candle_a) && !candles_output.contains_key(candle_b) {
                    let existing_entry = candles_output.get(candle_a).unwrap();
                    let existing_day = existing_entry.sum / existing_entry.divisor;

                    // example: A is bing, B is redhaven. redhaven (B) is bing (A) +30 or something
                    // we have an existing bing (A) but we're missing redhaven (B)
                    // take the existing bing day and add (redhaven (B) - bing (A)) which will be a positive number
                    let difference = average_b.average - average_a.average;
                    candles_output.insert(
                        candle_b.clone(),
                        AverageOffset {
                            sum: existing_day + difference,
                            divisor: 1.0,
                        },
                    );
                    first_location_average.unwrap().1.used = true;
                    first_location_average = None;
                    // average_b.used = true; // invalidate only one - maybe using the other in other possible pairs will lead to better averages
                    println!(
                        "added a new relative->relative value: {} is {} + {} days",
                        candle_b.name, candle_a.name, difference
                    );
                    print_candles(&candles_output);
                    num_changed += 1;
                    continue;
                }

                if !candles_output.contains_key(candle_a) && candles_output.contains_key(candle_b) {
                    let existing_entry = candles_output.get(candle_b).unwrap();
                    let existing_day = existing_entry.sum / existing_entry.divisor;

                    // example: A is bing, B is redhaven. redhaven (B) is bing (A) +30 or something
                    // we have an existing redhaven (B) but we're missing bing (A)
                    // take the existing redhaven day and add (bing (A) - redhaven (B)) which will be a negative number
                    let difference = average_a.average - average_b.average;
                    candles_output.insert(
                        candle_a.clone(),
                        AverageOffset {
                            sum: existing_day + difference,
                            divisor: 1.0,
                        },
                    );
                    first_location_average.unwrap().1.used = true;
                    first_location_average = None;
                    // average_b.used = true; // invalidate only one - maybe using the other in other possible pairs will lead to better averages

                    println!(
                        "added a new relative->relative value: {} is {} + {} days",
                        candle_a.name, candle_b.name, difference
                    );
                    print_candles(&candles_output);
                    num_changed += 1;
                    continue;
                }

                if candles_output.contains_key(candle_a) && candles_output.contains_key(candle_b) {
                    let existing_entry_a = candles_output.get(candle_a).unwrap();
                    let existing_day_a = existing_entry_a.sum / existing_entry_a.divisor;
                    let existing_entry_b = candles_output.get(candle_b).unwrap();
                    let existing_day_b = existing_entry_b.sum / existing_entry_b.divisor;
                    let existing_difference = existing_day_b - existing_day_a;

                    let new_difference = average_b.average - average_a.average;

                    println!(
                        "{} to {}: was {:.1}, new value {:.1}",
                        candle_a.name, candle_b.name, existing_difference, new_difference
                    );

                    // if new_difference bigger or equal to existing_difference: add half to B and subtract half from A
                    // if the difference is negative then add/subtract will get flipped automatically

                    let adjustment = new_difference - existing_difference;

                    let subtract_from_a = adjustment * 0.5;
                    let new_day_to_average_with_a = existing_day_a * 0.5 - subtract_from_a;

                    let add_to_b = adjustment * 0.5;
                    let new_day_to_average_with_b = existing_day_b * 0.5 + add_to_b;

                    {
                        let existing_entry_a = candles_output.get_mut(candle_a).unwrap();
                        println!("changing {existing_entry_a}");
                        existing_entry_a.sum += new_day_to_average_with_a;
                        existing_entry_a.divisor += 0.5;
                        println!("to {existing_entry_a}");
                    }

                    {
                        let existing_entry_b = candles_output.get_mut(candle_b).unwrap();
                        println!("changing {existing_entry_b}");
                        existing_entry_b.sum += new_day_to_average_with_b;
                        existing_entry_b.divisor += 0.5;
                        println!("to {existing_entry_b}");
                    }

                    first_location_average.unwrap().1.used = true;
                    first_location_average = None;
                    //   average_b.used = true; // invalidate only one - maybe using the other in other possible pairs will lead to better averages

                    print_candles(&candles_output);
                    num_changed += 1;
                    continue;
                }
            }
        }

        if num_changed == 0 {
            println!("ending relative->relative rounds, none changed");
            break;
        } else {
            println!("round {} changed {}", round, num_changed);
        }
        // loop until we make no progress
    }

    // normalize to the earliest standard candle
    let mut lowest_day = None;
    for offset in candles_output.values() {
        let day = (offset.sum / offset.divisor) as i32;
        if let Some(lowest_day) = lowest_day {
            if day > lowest_day {
                continue;
            }
        }
        lowest_day = Some(day);
    }
    if lowest_day.is_none() {
        lowest_day = Some(0); // error - didn't find anything
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct CandleOutput {
        #[serde(rename = "type")]
        pub type_: String,
        pub name: String,
        pub day: i32,
    }
    let mut output = Vec::new();
    for (candle, offset) in &candles_output {
        output.push(CandleOutput {
            type_: candle.type_.clone(),
            name: candle.name.clone(),
            day: (offset.sum / offset.divisor) as i32 - lowest_day.unwrap(),
        });
    }

    output.sort_by_key(|a| a.day);

    println!("calculated relative-relative times: {:#?}", output);

    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path = path.join("./generated/relative-relative.json");

    // this file is used by the frontend to make relative-only charts, and we might use it for predictions in the future
    fs::write(path, serde_json::to_string_pretty(&output).unwrap())
        .expect("Unable to write relative-relative file");
}

// write a github markdown file listing all collections with the "needs_help" tag
// so contributors can see this on github without needing an api call or whatever
pub fn write_needs_help_file(db_conn: &mut SqliteConnection) {
    let all_collections = collections::dsl::collections
        .filter(collections::needs_help.eq(1))
        .load::<Collection>(db_conn)
        .unwrap();

    let mut formatted: Vec<String> = all_collections
        .into_iter()
        .map(|c| {
            format!(
                "* [{}{}](references/{}.json5)",
                c.path,
                c.filename,
                urlencoding::encode(&format!("{}{}", c.path, c.filename))
            )
        })
        .collect();
    formatted.sort();

    let header = r#"# list of collections tagged "needs_help"
generated by [import_db.rs](../backend/src/import_db.rs) - don't edit

"#;

    let data: String = formatted
        .into_iter()
        .map(|line| line + "\n")
        .collect::<String>();

    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path = path.join("../plant_database/help_needed.md");

    fs::write(path, header.to_owned() + &data).expect("Unable to write help needed file");
}
