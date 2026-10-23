use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};

use chrono::prelude::*;

// output is unix timestamp. same as uspp_number_to_release_year() but it adds 17 years
// plant patent period has varied over time but it's always been about 20 years from application or 17 years from grant
// because we only know grant time, use 17 years
pub fn uspp_number_to_expiration(uspp_number_input: i32) -> i64 {
    const YEARS_AFTER_ISSUE: i32 = 17;

    NaiveDate::from_ymd_opt(
        uspp_number_to_release_year(uspp_number_input) + YEARS_AFTER_ISSUE,
        1,
        1,
    )
    .unwrap()
    .and_hms_opt(12, 0, 0)
    .unwrap()
    .and_utc()
    .timestamp()
}

pub fn is_standard_candle(type_input: &str, name: &str) -> bool {
    if let Some(candle) = type_to_standard_candle(type_input) {
        if candle.type_ != type_input {
            return false; // todo - figure out exactly what to do with nectarines - this ignores them
        }

        return name == candle.name;
    }
    false
}

pub struct CandleTarget {
    pub type_: String,
    pub name: String,
}

lazy_static! {
    static ref TYPE_TO_CANDLE: HashMap<&'static str, CandleTarget> = {

    HashMap::from([
        (
            "Peach",
            CandleTarget {
                type_: "Peach".to_string(),
                name: "Redhaven".to_string(),
            },
        ),
        (
            "Nectarine",
            CandleTarget {
                type_: "Peach".to_string(),
                name: "Redhaven".to_string(),
            },
        ), // note this points at redhaven peach, not a nectarine
        (
            "Japanese Plum",
            CandleTarget {
                type_: "Japanese Plum".to_string(),
                name: "Santa Rosa".to_string(),
            },
        ),
        (
            "Plumcot",
            CandleTarget {
                type_: "Japanese Plum".to_string(),
                name: "Santa Rosa".to_string(), // note this is a different type
            },
        ),
        (
            "Apriplum",
            CandleTarget {
                type_: "Japanese Plum".to_string(),
                name: "Santa Rosa".to_string(), // note this is a different type
            },
        ),
        (
            "Euro Plum",
            CandleTarget {
                type_: "Euro Plum".to_string(),
                name: "Italian".to_string(),
            },
        ),
        (
            "Sweet Cherry",
            CandleTarget {
                type_: "Sweet Cherry".to_string(),
                name: "Bing".to_string(),
            },
        ),
        (
            "Sour Cherry",
            CandleTarget {
                type_: "Sour Cherry".to_string(),
                name: "Montmorency".to_string(),
            },
        ),
        (
            "Apple",
            CandleTarget {
                type_: "Apple".to_string(),
                name: "Red Delicious".to_string(), // note this is just "Delicious" in many references and I've picked the more common name
            },
        ),
        (
            "Grape",
            CandleTarget {
                type_: "Grape".to_string(),
                name: "Concord".to_string(),
            },
        ),
        (
            "Euro Pear",
            CandleTarget {
                type_: "Euro Pear".to_string(),
                name: "Bartlett".to_string(),
            },
        ),
        (
            "Apricot",
            CandleTarget {
                type_: "Apricot".to_string(),
                name: "Blenheim".to_string(),
            },
        ),
        (
            "Almond",
            CandleTarget {
                type_: "Almond".to_string(),
                name: "Nonpareil".to_string(),
            },
        ),
        (
            "Walnut",
            CandleTarget {
                type_: "Walnut".to_string(),
                name: "Chandler".to_string(),
            },
        ),
        (
            "Blueberry",
            CandleTarget {
                type_: "Blueberry".to_string(),
                name: "Legacy".to_string(),
            },
        ),
        (
            "Blackberry",
            CandleTarget {
                type_: "Blackberry".to_string(),
                name: "Marion".to_string(),
            },
        ),
        (
            "Asian Pear",
            CandleTarget {
                type_: "Asian Pear".to_string(),
                name: "Shinseiki".to_string(),
            },
        ),
        (
            "Raspberry",
            CandleTarget {
                type_: "Raspberry".to_string(),
                name: "Heritage".to_string(),
            },
        ),
        (
            "Hardy Kiwi",
            CandleTarget {
                type_: "Hardy Kiwi".to_string(),
                name: "Ken's Red".to_string(),
            },
        ),
        (
            "Fuzzy Kiwi",
            CandleTarget {
                type_: "Fuzzy Kiwi".to_string(),
                name: "Hayward".to_string(),
            },
        ),
        (
            "Pomegranate",
            CandleTarget {
                type_: "Pomegranate".to_string(),
                name: "Wonderful".to_string(),
            },
        ),
        // todo: Fuyu Persimmon? Pineapple Quince?
        // Olive, Chestnut, Hazelnut (I think Chestnut and Hazelnut probably have standard candles in the literature already)
    ])
    };
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct Candle {
    pub type_: String,
    pub name: String,
}

// returns a set of type+name based on the static table above
// will remove overlaps like peach and nectarine both pointing to redhaven
pub fn get_standard_candles() -> HashSet<Candle> {
    let mut output = HashSet::new();
    for value in (*TYPE_TO_CANDLE).values() {
        output.insert(Candle {
            type_: value.type_.clone(),
            name: value.name.clone(),
        });
    }

    output
}

pub fn type_to_standard_candle(type_input: &str) -> Option<&CandleTarget> {
    TYPE_TO_CANDLE.get(type_input)
}

// for varieties with no release year listed but a patent number given, guess at it based on their US patent number
pub fn uspp_number_to_release_year(uspp_number_input: i32) -> i32 {
    // https://www.uspto.gov/web/offices/ac/ido/oeip/taf/issuyear.htm

    struct YearAndPatentNumber {
        year: i32,
        patent_number: i32,
    }

    const PATENT_NUMBERS_TO_YEAR: [YearAndPatentNumber; 91] = [
        YearAndPatentNumber {
            year: 1931,
            patent_number: 1,
        },
        YearAndPatentNumber {
            year: 1932,
            patent_number: 6,
        },
        YearAndPatentNumber {
            year: 1933,
            patent_number: 52,
        },
        YearAndPatentNumber {
            year: 1934,
            patent_number: 85,
        },
        YearAndPatentNumber {
            year: 1935,
            patent_number: 117,
        },
        YearAndPatentNumber {
            year: 1936,
            patent_number: 162,
        },
        YearAndPatentNumber {
            year: 1937,
            patent_number: 211,
        },
        YearAndPatentNumber {
            year: 1938,
            patent_number: 266,
        },
        YearAndPatentNumber {
            year: 1939,
            patent_number: 307,
        },
        YearAndPatentNumber {
            year: 1940,
            patent_number: 352,
        },
        YearAndPatentNumber {
            year: 1941,
            patent_number: 437,
        },
        YearAndPatentNumber {
            year: 1942,
            patent_number: 499,
        },
        YearAndPatentNumber {
            year: 1943,
            patent_number: 564,
        },
        YearAndPatentNumber {
            year: 1944,
            patent_number: 611,
        },
        YearAndPatentNumber {
            year: 1945,
            patent_number: 649,
        },
        YearAndPatentNumber {
            year: 1946,
            patent_number: 666,
        },
        YearAndPatentNumber {
            year: 1947,
            patent_number: 722,
        },
        YearAndPatentNumber {
            year: 1948,
            patent_number: 774,
        },
        YearAndPatentNumber {
            year: 1949,
            patent_number: 818,
        },
        YearAndPatentNumber {
            year: 1950,
            patent_number: 911,
        },
        YearAndPatentNumber {
            year: 1951,
            patent_number: 1001,
        },
        YearAndPatentNumber {
            year: 1952,
            patent_number: 1059,
        },
        YearAndPatentNumber {
            year: 1953,
            patent_number: 1160,
        },
        YearAndPatentNumber {
            year: 1954,
            patent_number: 1238,
        },
        YearAndPatentNumber {
            year: 1955,
            patent_number: 1339,
        },
        YearAndPatentNumber {
            year: 1956,
            patent_number: 1442,
        },
        YearAndPatentNumber {
            year: 1957,
            patent_number: 1543,
        },
        YearAndPatentNumber {
            year: 1958,
            patent_number: 1672,
        },
        YearAndPatentNumber {
            year: 1959,
            patent_number: 1792,
        },
        YearAndPatentNumber {
            year: 1960,
            patent_number: 1893,
        },
        YearAndPatentNumber {
            year: 1961,
            patent_number: 2009,
        },
        YearAndPatentNumber {
            year: 1962,
            patent_number: 2117,
        },
        YearAndPatentNumber {
            year: 1963,
            patent_number: 2208,
        },
        YearAndPatentNumber {
            year: 1964,
            patent_number: 2337,
        },
        YearAndPatentNumber {
            year: 1965,
            patent_number: 2465,
        },
        YearAndPatentNumber {
            year: 1966,
            patent_number: 2585,
        },
        YearAndPatentNumber {
            year: 1967,
            patent_number: 2699,
        },
        YearAndPatentNumber {
            year: 1968,
            patent_number: 2784,
        },
        YearAndPatentNumber {
            year: 1969,
            patent_number: 2856,
        },
        YearAndPatentNumber {
            year: 1970,
            patent_number: 2959,
        },
        YearAndPatentNumber {
            year: 1971,
            patent_number: 3011,
        },
        YearAndPatentNumber {
            year: 1972,
            patent_number: 3063,
        },
        YearAndPatentNumber {
            year: 1973,
            patent_number: 3281,
        },
        YearAndPatentNumber {
            year: 1974,
            patent_number: 3413,
        },
        YearAndPatentNumber {
            year: 1975,
            patent_number: 3674,
        },
        YearAndPatentNumber {
            year: 1976,
            patent_number: 3824,
        },
        YearAndPatentNumber {
            year: 1977,
            patent_number: 4001,
        },
        YearAndPatentNumber {
            year: 1978,
            patent_number: 4174,
        },
        YearAndPatentNumber {
            year: 1979,
            patent_number: 4360,
        },
        YearAndPatentNumber {
            year: 1980,
            patent_number: 4491,
        },
        YearAndPatentNumber {
            year: 1981,
            patent_number: 4612,
        },
        YearAndPatentNumber {
            year: 1982,
            patent_number: 4796,
        },
        YearAndPatentNumber {
            year: 1983,
            patent_number: 4970,
        },
        YearAndPatentNumber {
            year: 1984,
            patent_number: 5168,
        },
        YearAndPatentNumber {
            year: 1985,
            patent_number: 5380,
        },
        YearAndPatentNumber {
            year: 1986,
            patent_number: 5622,
        },
        YearAndPatentNumber {
            year: 1987,
            patent_number: 5846,
        },
        YearAndPatentNumber {
            year: 1988,
            patent_number: 6075,
        },
        YearAndPatentNumber {
            year: 1989,
            patent_number: 6501,
        },
        YearAndPatentNumber {
            year: 1990,
            patent_number: 7089,
        },
        YearAndPatentNumber {
            year: 1991,
            patent_number: 7408,
        },
        YearAndPatentNumber {
            year: 1992,
            patent_number: 7761,
        },
        YearAndPatentNumber {
            year: 1993,
            patent_number: 8082,
        },
        YearAndPatentNumber {
            year: 1994,
            patent_number: 8527,
        },
        YearAndPatentNumber {
            year: 1995,
            patent_number: 9026,
        },
        YearAndPatentNumber {
            year: 1996,
            patent_number: 9413,
        },
        YearAndPatentNumber {
            year: 1997,
            patent_number: 9776,
        },
        YearAndPatentNumber {
            year: 1998,
            patent_number: 10172,
        },
        YearAndPatentNumber {
            year: 1999,
            patent_number: 10743,
        },
        YearAndPatentNumber {
            year: 2000,
            patent_number: 11169,
        },
        YearAndPatentNumber {
            year: 2001,
            patent_number: 11728,
        },
        YearAndPatentNumber {
            year: 2002,
            patent_number: 12314,
        },
        YearAndPatentNumber {
            year: 2003,
            patent_number: 13447,
        },
        YearAndPatentNumber {
            year: 2004,
            patent_number: 14441,
        },
        YearAndPatentNumber {
            year: 2005,
            patent_number: 15460,
        },
        YearAndPatentNumber {
            year: 2006,
            patent_number: 16176,
        },
        YearAndPatentNumber {
            year: 2007,
            patent_number: 17326,
        },
        YearAndPatentNumber {
            year: 2008,
            patent_number: 18373,
        },
        YearAndPatentNumber {
            year: 2009,
            patent_number: 19613,
        },
        YearAndPatentNumber {
            year: 2010,
            patent_number: 20622,
        },
        YearAndPatentNumber {
            year: 2011,
            patent_number: 21604,
        },
        YearAndPatentNumber {
            year: 2012,
            patent_number: 22428,
        },
        YearAndPatentNumber {
            year: 2013,
            patent_number: 23288,
        },
        YearAndPatentNumber {
            year: 2014,
            patent_number: 24135,
        },
        YearAndPatentNumber {
            year: 2015,
            patent_number: 25207,
        },
        YearAndPatentNumber {
            year: 2016,
            patent_number: 26285,
        },
        YearAndPatentNumber {
            year: 2017,
            patent_number: 27520,
        },
        YearAndPatentNumber {
            year: 2018,
            patent_number: 28831,
        },
        YearAndPatentNumber {
            year: 2019,
            patent_number: 30040,
        },
        YearAndPatentNumber {
            year: 2020,
            patent_number: 31315,
        },
        YearAndPatentNumber {
            year: 2021,
            patent_number: 32717,
        },
    ];

    let mut previous_year = PATENT_NUMBERS_TO_YEAR[0].year;
    for element in PATENT_NUMBERS_TO_YEAR {
        if element.patent_number > uspp_number_input {
            break;
        }

        previous_year = element.year;
    }

    previous_year
}
