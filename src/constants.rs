use regex::Regex;
use lazy_static::lazy_static;

lazy_static!{
    pub static ref UNIT_CODE_FORMAT: Regex = Regex::new(r"([a-zA-Z]{4}\d{4})-(\w+).+").unwrap();
    pub static ref SEMESTER_FORMAT: Regex = Regex::new(r"S([12]).+").unwrap();
}


pub const CONFIG_FILE: &'static str = "query.json";
pub const PORT: u16 = 9515;

pub const SEMESTER: u64 = 1;
pub const PUBLIC_TIMETABLE_URL: &'static str = "https://timetable.sydney.edu.au/even/timetable/#subjects";
// chromedriver --port={port}