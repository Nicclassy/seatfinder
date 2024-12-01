use std::io::BufReader;
use std::fs::File;
use std::error::Error;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::net::TcpListener;
use std::thread;

use serde_json::{self, Value};
use chrono::Datelike;
use rodio::Source;

use crate::consts::{
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

const QUERY: &str = "query";
const QUERIES: &str = "queries";

pub fn format_u64(fmt: &str, value: u64) -> String {
    // Workaround for lack of runtime variadic .format method in C#/C/Python/Java etc.
    // Not the cleanest solution but obeys the orphan rule
    fmt.replacen("{}", &value.to_string(), 1)
}

pub fn format_usize(fmt: &str, value: usize) -> String {
    fmt.replacen("{}", &value.to_string(), 1)
}

pub fn format_str(fmt: &str, value: &str) -> String {
    fmt.replacen("{}", value, 1)
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

    if json_config[QUERY].as_object().is_some() {
        let query = FinderQuery::try_new(&json_config[QUERY])?;
        return Ok(vec![query]);
    };

    let queries: Vec<FinderQuery> = json_config[QUERIES]
        .as_array()
        .ok_or(ParseError::ParseQueriesError)?
        .iter()
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

pub fn annoy(path: &PathBuf) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open(path).unwrap());
    let source = rodio::Decoder::new_mp3(file).unwrap();
    stream_handle.play_raw(source.convert_samples()).unwrap();

    loop {
        thread::sleep(std::time::Duration::from_millis(1));
    }
}

pub fn single_offering(query: &FinderQuery, subcode: &String) -> Result<(), Box<dyn Error>> {
    let Some((_, [unit_code, session])) = 
        SUBCODE_RE.captures(subcode).map(|caps| caps.extract()) else {
            return Err(
                Box::new(
                    ParseError::RegexNoMatch(
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
                ParseError::RegexNoMatch(
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
                OfferingError::SemesterInvalid { 
                    expected: query.semester.clone(), 
                    actual: semester 
                } 
            )
        )
    }
}

pub fn multiple_offerings(
    query: &FinderQuery, 
    subcodes: &[String], 
) -> Option<usize> {
    subcodes
        .iter()
        .position(|subcode| single_offering(query, subcode).is_ok())
}