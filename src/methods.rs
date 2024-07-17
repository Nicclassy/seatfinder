use std::error::Error;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::net::TcpListener;

use once_cell::sync::Lazy;
use serde_json::{self, Value};
use chrono::Datelike;
use thirtyfour::WebElement;

use crate::constants::{
    CONFIG_FILE, 
    LOCALHOST, 
    MAX_PORT, 
    PUBLIC_TIMETABLE_EVEN, 
    PUBLIC_TIMETABLE_ODD, 
    SEMESTER_FORMAT, 
    SUBCODE_FORMAT
};
use crate::allocation::Semester;
use crate::query::FinderQuery;
use crate::error::{ParseError, OfferingError};

static USED_PORTS: Lazy<Arc<Mutex<Vec<u16>>>> = Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

const QUERIES: &'static str = "queries";

pub fn parse_queries() -> Result<Vec<FinderQuery>, Box<dyn Error>> {
    let file = File::open(CONFIG_FILE)?;
    let json_config: Value = serde_json::from_reader(file)?;

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
    TcpListener::bind((LOCALHOST, port)).is_ok()
}

pub fn unoccupied_port(start: u16) -> u16 {
    let mut port = start;
    let used_ports_cloned = Arc::clone(&USED_PORTS);

    loop {
        if port == MAX_PORT {
            panic!("could not find available port")
        }

        let used_ports = used_ports_cloned.lock().unwrap();
        if used_ports.contains(&port) {
            port += 1;
            continue;
        }

        drop(used_ports);

        match TcpListener::bind((LOCALHOST, port)) {
            Ok(_) => {
                let mut used_ports = used_ports_cloned.lock().unwrap();
                used_ports.push(port);
                return port;
            },
            Err(_) => port += 1,
        }
    }
}

pub fn single_offering(query: &FinderQuery, subcode: &String) -> Result<(), Box<dyn Error>> {
    let Some((_, [unit_code, session])) = 
        SUBCODE_FORMAT.captures(&subcode).map(|caps| caps.extract()) else {
            return Err(
                Box::new(
                    ParseError::RegexNoMatchError(
                        SUBCODE_FORMAT.as_str(), 
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

    let semester_match = match SEMESTER_FORMAT.captures(session) {
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
                    SEMESTER_FORMAT.as_str(), 
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

pub fn multiple_offerings<'a>(
    query: &FinderQuery, 
    subcodes: &'a Vec<String>, 
    elements: &'a Vec<WebElement>
) -> Option<&'a WebElement> {
    let element_position = subcodes
        .iter()
        .position(|subcode| single_offering(query, subcode).is_ok());

    match element_position {
        Some(index) => elements.get(index),
        None => None
    }
}