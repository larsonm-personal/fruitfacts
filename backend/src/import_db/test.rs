use crate::import_db::notoriety::BasePlantNotoriety;
use crate::import_db::notoriety::BasePlantNotorietyInput;
use crate::import_db::{
    notoriety::base_plant_notoriety_calc, util::uspp_number_to_expiration,
    util::uspp_number_to_release_year,
};
use diesel::connection::SimpleConnection;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use super::*;

const DEFAULT_TEST_REFERENCE_LOAD_SECONDS: u64 = 20;

fn test_env_u64(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

fn print_uncaptured_test_message(message: &str) {
    #[cfg(windows)]
    {
        if let Ok(mut console) = OpenOptions::new().write(true).open("CONOUT$") {
            let _ = writeln!(console, "{message}");
            return;
        }
    }

    #[cfg(not(windows))]
    {
        if let Ok(mut tty) = OpenOptions::new().write(true).open("/dev/tty") {
            let _ = writeln!(tty, "{message}");
            return;
        }
    }

    let mut stderr = std::io::stderr().lock();
    let _ = writeln!(stderr, "{message}");
}

struct TestProgress {
    stop_tx: Option<mpsc::Sender<()>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl TestProgress {
    fn start(label: &'static str) -> Self {
        let (stop_tx, stop_rx) = mpsc::channel();
        let started = Instant::now();
        print_uncaptured_test_message(&format!("{label} started"));

        let handle = thread::spawn(move || loop {
            match stop_rx.recv_timeout(Duration::from_secs(10)) {
                Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    print_uncaptured_test_message(&format!(
                        "{label} still running after {}s",
                        started.elapsed().as_secs()
                    ));
                }
            }
        });

        Self {
            stop_tx: Some(stop_tx),
            handle: Some(handle),
        }
    }
}

impl Drop for TestProgress {
    fn drop(&mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[test]
fn test_format_s_allele() {
    assert_eq!(
        format_s_allele(&Some("".to_string()), &Some("".to_string())),
        ""
    );
    assert_eq!(
        format_s_allele(&Some("".to_string()), &Some("S4'S9 [1]".to_string())),
        "S4'S9 [1]"
    );
    assert_eq!(
        format_s_allele(&Some("S1S4' [1]".to_string()), &Some("".to_string())),
        "S1S4' [1]"
    );
    assert_eq!(
        format_s_allele(&Some("S1S4 [1]".to_string()), &Some("S1S4 [2]".to_string())),
        "S1S4 [1,2]"
    );
    assert_eq!(
        format_s_allele(
            &Some("S1S4' [1]".to_string()),
            &Some("S1S4 [2]".to_string())
        ),
        "S1S4 [2] or S1S4' [1] (conflicting sources)"
    );
    assert_eq!(
        format_s_allele(
            &Some("S1S4 [2] or S1S4' [1] (conflicting sources)".to_string()),
            &Some("S1S4' [3]".to_string())
        ),
        "S1S4 [2] or S1S4' [1,3] (conflicting sources)"
    );
}

#[test]
fn test_is_a_midpoint() {
    assert!(!is_a_midpoint(""));
    assert!(!is_a_midpoint("mid"));
    assert!(!is_a_midpoint("Sep 15-30"));
    assert!(!is_a_midpoint("Sep 15"));

    assert!(is_a_midpoint("sep"));
    assert!(is_a_midpoint("mid October"));
}

#[test]
fn test_uspp_number_to_release_year() {
    assert_eq!(uspp_number_to_release_year(1), 1931);
    assert_eq!(uspp_number_to_release_year(4969), 1982);
    assert_eq!(uspp_number_to_release_year(4970), 1983);
    assert_eq!(uspp_number_to_release_year(4971), 1983);
    assert_eq!(uspp_number_to_release_year(33333), 2021);
}

#[test]
fn test_uspp_number_to_expiration() {
    assert_eq!(uspp_number_to_expiration(1), -694267200); // 1948
    assert_eq!(uspp_number_to_expiration(33333), 2145960000); // 2038 (todo - test 2039 I guess)
}

#[test]
fn test_month_location() {
    assert_eq!(month_location(""), MonthLocationType::NoMonth);
    assert_eq!(
        month_location("september 23"),
        MonthLocationType::MonthAtBeginning
    );
    assert_eq!(
        month_location("mid september"),
        MonthLocationType::MonthAtEnd
    );
    assert_eq!(month_location("Sep"), MonthLocationType::MonthAtBeginning);
}

#[test]
fn test_get_month() {
    assert_eq!(get_month("Sep"), "sep")
}

#[test]
fn test_dates() {
    assert_eq!(string_to_day_number("Jan 1"), Some(1));
    assert_eq!(string_to_day_number("February 28"), Some(59));
    assert_eq!(string_to_day_number("February 29"), Some(60));
    assert_eq!(string_to_day_number("March 1"), Some(61));
    assert_eq!(string_to_day_number("August 12"), Some(225));
    assert_eq!(string_to_day_number("Sep 20"), Some(264));
    assert_eq!(string_to_day_number("Dec 31"), Some(366));

    assert_eq!(string_to_day_number("early January"), Some(5));
    assert_eq!(string_to_day_number("early to mid jan"), Some(10));
    assert_eq!(string_to_day_number("early-mid jan"), Some(10));
    assert_eq!(string_to_day_number("mid jan"), Some(15));
    assert_eq!(string_to_day_number("mid-late jan"), Some(20));
    assert_eq!(string_to_day_number("late February"), Some(56));
    assert_eq!(string_to_day_number("early August"), Some(218));
    assert_eq!(string_to_day_number("mid-late August"), Some(233));
    assert_eq!(string_to_day_number("late August"), Some(238));

    assert_eq!(string_to_day_number("mar"), Some(75));
    assert_eq!(string_to_day_number("April"), Some(106));

    assert_eq!(string_to_day_number("eary Jun"), None);

    assert_eq!(
        string_to_day_number(" Around April 16 (Gainesville, FL)"),
        Some(107)
    );
    assert_eq!(
        string_to_day_number("Around May 4 (Gainesville, FL)"),
        Some(125)
    );

    assert_eq!(
        string_to_day_number("9/25"),
        string_to_day_number("September 25")
    );
    assert_eq!(
        string_to_day_number("10/15"),
        string_to_day_number("October 15")
    );
}

#[test]
fn test_parse_released() {
    assert_eq!(parse_released(""), None);
    assert_eq!(
        parse_released("WSU 2011*"),
        Some(ReleasedOutput {
            releaser: Some("WSU".to_string()),
            year: Some(2011),
            authoritative: true
        })
    );
    assert_eq!(
        parse_released("WSU Mt Vernon 2003"),
        Some(ReleasedOutput {
            releaser: Some("WSU Mt Vernon".to_string()),
            year: Some(2003),
            authoritative: false
        })
    );
    assert_eq!(
        parse_released("WSU"),
        Some(ReleasedOutput {
            releaser: Some("WSU".to_string()),
            year: None,
            authoritative: false
        })
    );
    assert_eq!(
        parse_released("WSU*"),
        Some(ReleasedOutput {
            releaser: Some("WSU".to_string()),
            year: None,
            authoritative: true
        })
    );
    assert_eq!(
        parse_released("2013"),
        Some(ReleasedOutput {
            releaser: None,
            year: Some(2013),
            authoritative: false
        })
    );
    assert_eq!(
        parse_released("2013*"),
        Some(ReleasedOutput {
            releaser: None,
            year: Some(2013),
            authoritative: true
        })
    );
}

#[test]
#[should_panic]
fn test_parse_released_panic() {
    parse_released("201");
}

#[test]
#[should_panic]
fn test_parse_released_panic_2() {
    parse_released("2101");
}
const TEST_WINDOW_SIZE: u32 = 10;
#[test]
fn test_day_range() {
    assert_eq!(
        string_to_day_range("eary Jun", TEST_WINDOW_SIZE), // misspelled - parse error
        None
    );
    assert_eq!(
        string_to_day_range("Early", TEST_WINDOW_SIZE).unwrap(), // this refers to "early within season" and we're not parsing it for now, just store the text
        DayRangeOutput {
            parse_type: DateParseType::Unparsed,
            start: None,
            end: None
        }
    );
    assert_eq!(
        string_to_day_range("Early August", TEST_WINDOW_SIZE).unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::StartOnly,
            start: Some(218),
            end: None
        }
    );
    assert_eq!(
        string_to_day_range("August 7", TEST_WINDOW_SIZE).unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::StartOnly,
            start: Some(220),
            end: None
        }
    );
    assert_eq!(
        string_to_day_range("August 15", TEST_WINDOW_SIZE).unwrap(), // not midpoint, even though it's mid month, because it's an exact date
        DayRangeOutput {
            parse_type: DateParseType::StartOnly,
            start: Some(228),
            end: None
        }
    );
    assert_eq!(
        string_to_day_range("mid-late August", TEST_WINDOW_SIZE).unwrap(), // not a recommended format because the '-' gets it parsed as two dates
        DayRangeOutput {
            parse_type: DateParseType::TwoDates,
            start: Some(228),
            end: Some(238)
        }
    );
    assert_eq!(
        string_to_day_range("mid to late August", TEST_WINDOW_SIZE).unwrap(), // same
        DayRangeOutput {
            parse_type: DateParseType::TwoDates,
            start: Some(228),
            end: Some(238)
        }
    );
    assert_eq!(
        string_to_day_range("August", TEST_WINDOW_SIZE).unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::TwoDates, // this becomes two dates because we build a window around it, +/- half the window size, in order to preserve it being listed like a midpoint in the reference
            start: Some(223),
            end: Some(233),
        }
    );
    assert_eq!(
        string_to_day_range("mid September", TEST_WINDOW_SIZE).unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::TwoDates, // same as above, this is a midpoint
            start: Some(254),
            end: Some(264),
        }
    );

    assert_eq!(
        string_to_day_range("early to late August", TEST_WINDOW_SIZE).unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::TwoDates,
            start: Some(218),
            end: Some(238)
        }
    );

    assert_eq!(
        string_to_day_range("late August to mid September", TEST_WINDOW_SIZE).unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::TwoDates,
            start: Some(238),
            end: Some(259)
        }
    );

    assert_eq!(
        string_to_day_range("late Aug to Oct", TEST_WINDOW_SIZE).unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::TwoDates,
            start: Some(238),
            end: Some(289)
        }
    );

    assert_eq!(
        string_to_day_range("Sep 20-30", TEST_WINDOW_SIZE).unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::TwoDates,
            start: Some(264),
            end: Some(274)
        }
    );
    assert_eq!(
        string_to_day_range("Sep 25-Oct 5", TEST_WINDOW_SIZE).unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::TwoDates,
            start: Some(269),
            end: Some(279)
        }
    );
    assert_eq!(
        string_to_day_range("June 21 to July 10", TEST_WINDOW_SIZE).unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::TwoDates,
            start: Some(173),
            end: Some(192)
        }
    );
    assert_eq!(
        string_to_day_range("Oct-Nov", TEST_WINDOW_SIZE).unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::TwoDates,
            start: Some(289),
            end: Some(320)
        }
    );
    assert_eq!(
        string_to_day_range("late Oct-Nov", TEST_WINDOW_SIZE).unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::TwoDates,
            start: Some(299),
            end: Some(320)
        }
    );
    assert_eq!(
        string_to_day_range("July 6", TEST_WINDOW_SIZE).unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::StartOnly,
            start: Some(188),
            end: None
        }
    );
    assert_eq!(
        string_to_day_range("June 29", TEST_WINDOW_SIZE).unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::StartOnly,
            start: Some(181),
            end: None
        }
    );
    assert_eq!(
        string_to_day_range("10/15", TEST_WINDOW_SIZE).unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::StartOnly,
            start: Some(289),
            end: None
        }
    );
    assert_eq!(
        string_to_day_range("Average of: July 6, June 29", TEST_WINDOW_SIZE).unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::StartOnly,
            start: Some(184),
            end: None
        }
    );
    assert_eq!(
        string_to_day_range(
            "First Harvest: Around April 30 (Gainesville, FL)",
            TEST_WINDOW_SIZE
        )
        .unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::StartOnly,
            start: Some(121),
            end: None
        }
    );
    assert_eq!(
        string_to_day_range(
            "50% Harvest: Around April 25 (Gainesville, FL)",
            TEST_WINDOW_SIZE
        )
        .unwrap(),
        DayRangeOutput {
            parse_type: DateParseType::TwoDates, // treated as a midpoint
            start: Some(111),
            end: Some(121),
        }
    );
}

#[test]
fn test_parse_relative_harvest() {
    assert_eq!(parse_relative_harvest("hi"), None);
    assert_eq!(
        parse_relative_harvest("Redhaven -32"),
        Some(HarvestRelativeParsed {
            name: "Redhaven".to_string(),
            type_: None,
            relative_days: -32
        })
    );
    assert_eq!(
        parse_relative_harvest("Redhaven +0"),
        Some(HarvestRelativeParsed {
            name: "Redhaven".to_string(),
            type_: None,
            relative_days: 0
        })
    );
    assert_eq!(
        parse_relative_harvest("Redhaven +45"),
        Some(HarvestRelativeParsed {
            name: "Redhaven".to_string(),
            type_: None,
            relative_days: 45
        })
    );
    assert_eq!(
        parse_relative_harvest("Redhaven -2 weeks"),
        Some(HarvestRelativeParsed {
            name: "Redhaven".to_string(),
            type_: None,
            relative_days: -14
        })
    );
    assert_eq!(
        parse_relative_harvest("Redhaven +0.5 weeks"),
        Some(HarvestRelativeParsed {
            name: "Redhaven".to_string(),
            type_: None,
            relative_days: 4
        })
    );
    assert_eq!(
        parse_relative_harvest("Redhaven -1.5 weeks"),
        Some(HarvestRelativeParsed {
            name: "Redhaven".to_string(),
            type_: None,
            relative_days: -11
        })
    );
    assert_eq!(
        parse_relative_harvest("Redhaven +9 weeks"),
        Some(HarvestRelativeParsed {
            name: "Redhaven".to_string(),
            type_: None,
            relative_days: 63
        })
    );
    assert_eq!(
        parse_relative_harvest(" with spaces-42"),
        Some(HarvestRelativeParsed {
            name: "with spaces".to_string(),
            type_: None,
            relative_days: -42
        })
    );
    assert_eq!(
        parse_relative_harvest("Bing -10 to -12"),
        Some(HarvestRelativeParsed {
            name: "Bing".to_string(),
            type_: None,
            relative_days: -11
        })
    );
    assert_eq!(
        parse_relative_harvest("Bing +14 to +15"),
        Some(HarvestRelativeParsed {
            name: "Bing".to_string(),
            type_: None,
            relative_days: 14
        })
    );
    assert_eq!(
        parse_relative_harvest("Concord -6 weeks"),
        Some(HarvestRelativeParsed {
            name: "Concord".to_string(),
            type_: None,
            relative_days: -42
        })
    );
    assert_eq!(
        parse_relative_harvest("Concord -4 to -5 weeks"),
        Some(HarvestRelativeParsed {
            name: "Concord".to_string(),
            type_: None,
            relative_days: -31
        })
    );
    assert_eq!(
        parse_relative_harvest("Red Delicious -5 to -4 weeks"),
        Some(HarvestRelativeParsed {
            name: "Red Delicious".to_string(),
            type_: None,
            relative_days: -31
        })
    );
    assert_eq!(
        parse_relative_harvest("Peach: Redhaven -32"),
        Some(HarvestRelativeParsed {
            name: "Redhaven".to_string(),
            type_: Some("Peach".to_string()),
            relative_days: -32
        })
    );
    assert_eq!(
        parse_relative_harvest("Apple: Red Delicious -5 to -4 weeks"),
        Some(HarvestRelativeParsed {
            name: "Red Delicious".to_string(),
            type_: Some("Apple".to_string()),
            relative_days: -31
        })
    );
}

#[test]
fn test_patent_parsing() {
    assert_eq!(
        string_to_patent_info(""),
        PatentInfo {
            uspp_number: None,
            uspp_expiration: None
        }
    );

    assert_eq!(
        string_to_patent_info("https://patents.google.com/patent/USPP9881 expired 2014"),
        PatentInfo {
            uspp_number: Some("9881".to_string()),
            uspp_expiration: Some(
                NaiveDate::from_ymd_opt(2014, 1, 1)
                    .unwrap()
                    .and_hms_opt(12, 0, 0)
                    .unwrap()
            )
        }
    );

    assert_eq!(
        string_to_patent_info("https://patents.google.com/patent/USPP30925 expires 2038-03-07"),
        PatentInfo {
            uspp_number: Some("30925".to_string()),
            uspp_expiration: Some(
                NaiveDate::from_ymd_opt(2038, 3, 7)
                    .unwrap()
                    .and_hms_opt(12, 0, 0)
                    .unwrap()
            )
        }
    );

    // without "expired/expires"
    assert_eq!(
        string_to_patent_info("https://patents.google.com/patent/USPP30925 2038-03-07"),
        PatentInfo {
            uspp_number: Some("30925".to_string()),
            uspp_expiration: Some(
                NaiveDate::from_ymd_opt(2038, 3, 7)
                    .unwrap()
                    .and_hms_opt(12, 0, 0)
                    .unwrap()
            )
        }
    );
}

#[test]
fn test_format_name_fts_string() {
    assert_eq!(format_name_fts_string(r#"tulare! (tm)"#), "tulare");
    assert_eq!(format_name_fts_string(r#"Santina ®"#), "santina");
}

#[test]
fn test_format_path() {
    assert_eq!(format_path(r#".\..\plant_database\references"#), "/");
    assert_eq!(
        format_path("./../plant_database/references/Oregon"),
        "Oregon/"
    );
    assert_eq!(
        format_path(r#".\..\plant_database\references\Oregon\Willamette Valley"#),
        "Oregon/Willamette Valley/"
    );
}

#[test]
fn test_base_plant_notoriety_calc() {
    assert_eq!(base_plant_notoriety_calc(&BasePlantNotorietyInput {
        notoriety_highest_collection_score: Some(50.0),
        notoriety_highest_collection_score_name: "collection name".to_string(),
        current_year: 2021,
        release_year: Some(1975),
        number_of_references: 1,
        uspp_number: None,
    }), BasePlantNotoriety{score: 34.0, explanation: "50 (collection name) *0.85 (>40 years old) *0.8 (1 references) *1.0 (no uspp number)".to_string()});
}

#[test]
#[ignore] // long runtime
fn test_database_loading() {
    let _progress = TestProgress::start("test_database_loading");
    let started = Instant::now();
    let reference_load_seconds = test_env_u64(
        "FRUITFACTS_TEST_REFERENCE_LOAD_SECONDS",
        DEFAULT_TEST_REFERENCE_LOAD_SECONDS,
    );

    print_uncaptured_test_message(&format!(
        "test_database_loading targeting {reference_load_seconds}s of new reference loading"
    ));

    let mut db_conn = super::establish_connection();
    super::ensure_database_schema(&mut db_conn);

    // speed up testing with sync = off (10% speedup) and a transaction (about 4x speedup)
    db_conn.batch_execute("PRAGMA synchronous = OFF").unwrap();

    let loaded_before = super::count_reference_collections(&mut db_conn);
    let mut items_loaded = Default::default();
    db_conn
        .immediate_transaction::<_, diesel::result::Error, _>(|db_conn| {
            items_loaded = super::load_all_with_options(
                db_conn,
                super::LoadAllOptions {
                    reference_load_time_budget: Some(Duration::from_secs(reference_load_seconds)),
                    skip_loaded_references: true,
                    static_data_load_mode: super::StaticDataLoadMode::IfEmpty,
                    post_load_processing_mode:
                        super::PostLoadProcessingMode::WhenNoReferencesLoaded,
                    write_generated_files: false,
                    ..Default::default()
                },
            );
            Ok(())
        })
        .unwrap();

    let loaded_after = super::count_reference_collections(&mut db_conn);
    print_uncaptured_test_message(&format!(
        "test_database_loading loaded {} references, skipped {}, database has {}/{} references in {}s",
        items_loaded.reference_items.reference_collections_loaded,
        items_loaded.reference_items.reference_collections_skipped,
        loaded_after,
        items_loaded.reference_items.reference_collections_available,
        started.elapsed().as_secs()
    ));
    println!("loaded: {:#?}", items_loaded);

    // update these every so often so we can check that a change doesn't cause fewer items than we expect
    assert_ge!(super::count_facts(&mut db_conn), 4);
    assert_ge!(super::count_base_plants(&mut db_conn), 22);
    assert_ge!(super::count_plant_types(&mut db_conn), 61);
    assert_ge!(loaded_after, loaded_before);
    assert_eq!(
        loaded_after - loaded_before,
        items_loaded.reference_items.reference_collections_loaded as i64
    );
    assert!(
        items_loaded.reference_items.reference_collections_loaded > 0
            || items_loaded.reference_items.reference_collections_skipped
                == items_loaded.reference_items.reference_collections_available
    );

    if items_loaded.reference_items.reference_collections_loaded > 0 {
        assert_gt!(items_loaded.reference_items.reference_locations_found, 0);
        assert_gt!(items_loaded.reference_items.reference_plants_added, 0);
    }
}
