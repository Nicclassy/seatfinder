use regex::Regex;
use lazy_static::lazy_static;

lazy_static!{
    pub static ref UNIT_CODE_FORMAT: Regex = Regex::new(r"([a-zA-Z]{4}\d{4})").unwrap();
    pub static ref SUBCODE_FORMAT: Regex = Regex::new(r"([a-zA-Z]{4}\d{4})-(\w+).+").unwrap();
    pub static ref SEMESTER_FORMAT: Regex = Regex::new(r"S([12]).+").unwrap();
    pub static ref SEMESTER_TABLE_FORMAT: Regex = Regex::new(r"Semester (\d+)").unwrap();
}

pub const CONFIG_FILE: &'static str = "config.json";
// chromedriver --port={port}
pub const PUBLIC_TIMETABLE_ODD: &'static str = "https://timetable.sydney.edu.au/odd/timetable/#subjects";
pub const PUBLIC_TIMETABLE_EVEN: &'static str = "https://timetable.sydney.edu.au/even/timetable/#subjects";
