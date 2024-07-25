use std::error::Error;
use std::fs::File;
use std::process::{Child, Command, Stdio};
use std::net::TcpListener;

use serde_json::{self, Value};
use chrono::Datelike;

use crate::constants::{
    CONFIG_FILE, 
    LOCALHOST, 
    MAX_PORT, 
    PUBLIC_TIMETABLE_EVEN, 
    PUBLIC_TIMETABLE_ODD, 
    SEMESTER_RE, 
    SUBCODE_RE
};
use crate::allocation::Semester;
use crate::query::FinderQuery;
use crate::error::{ParseError, OfferingError};

const QUERY: &'static str = "query";
const QUERIES: &'static str = "queries";

pub fn format_u64(src: &str, value: u64) -> String {
    // Workaround for lack of runtime variadic .format method in C#/C/Python/Java etc.
    // Not the cleanest solution but obeys the orphan rule
    src.replacen("{}", &value.to_string(), 1)
}

pub fn format_usize(src: &str, value: usize) -> String {
    src.replacen("{}", &value.to_string(), 1)
}

pub fn format_str(src: &str, value: &str) -> String {
    src.replacen("{}", value, 1)
}

pub fn chromedriver_process(port: u16) -> Result<Child, std::io::Error> {
    Command::new("chromedriver")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg(format!("--port={}", port))
        .spawn()
}

pub fn parse_queries() -> Result<Vec<FinderQuery>, Box<dyn Error>> {
    let file = File::open(CONFIG_FILE)?;
    let json_config: Value = serde_json::from_reader(file)?;

    match json_config[QUERY].as_object() {
        Some(_) => {
            let query = FinderQuery::try_new(&json_config[QUERY])?;
            return Ok(vec![query]);
        }
        None => {},
    };

    let queries: Vec<FinderQuery> = json_config[QUERIES]
        .as_array()
        .ok_or(ParseError::ParseQueriesError)?
        .into_iter()
        .map(FinderQuery::try_new)
        .collect::<Result<_, _>>()?;
    Ok(queries)
}

pub fn public_timetable_url_default() -> &'static str {
    let now = chrono::Local::now();
    let year = now.year();
    if year % 2 == 0 { PUBLIC_TIMETABLE_EVEN } else { PUBLIC_TIMETABLE_ODD }
}

pub fn port_is_occupied(port: u16) -> bool {
   TcpListener::bind((LOCALHOST, port)).is_err()
}

pub fn unoccupied_port(start: u16) -> u16 {
    let mut port = start;

    loop {
        if port == MAX_PORT {
            panic!("could not find an available port")
        }

        match TcpListener::bind((LOCALHOST, port)) {
            Ok(_) => return port,
            Err(_) => port += 1,
        }
    }
}

pub fn single_offering(query: &FinderQuery, subcode: &String) -> Result<(), Box<dyn Error>> {
    let Some((_, [unit_code, session])) = 
        SUBCODE_RE.captures(&subcode).map(|caps| caps.extract()) else {
            return Err(
                Box::new(
                    ParseError::RegexNoMatchError(
                        SUBCODE_RE.as_str(), 
                        subcode.to_string()
                    )
                )
            )
    };

    if unit_code != query.unit_code {
        return Err(
            Box::new(
                OfferingError::NoValidOfferingsError(query.unit_code())
            )
        )
    }

    let semester_match = match SEMESTER_RE.captures(session) {
        Some(caps) => match caps.get(1) {
            Some(semester_match) => semester_match,
            None => return Err(
                Box::new(
                    OfferingError::SemesterFormatError(
                        session.to_string(), 
                        subcode.to_string()
                    )
                )
            )
        },
        None => return Err(
            Box::new(
                ParseError::RegexNoMatchError(
                    SEMESTER_RE.as_str(), 
                    session.to_string()
                )
            )
        )
    };

    let semester = Semester::try_from(semester_match.as_str().to_string())?;

    match semester == query.semester || semester == Semester::Any {
        true => Ok(()),
        false => Err(
            Box::new(
                OfferingError::SemesterInvalidError { 
                    expected: query.semester.clone(), 
                    actual: semester 
                } 
            )
        )
    }
}

pub fn multiple_offerings(
    query: &FinderQuery, 
    subcodes: &Vec<String>, 
) -> Option<usize> {
    subcodes
        .iter()
        .position(|subcode| single_offering(query, subcode).is_ok())
}