use regex::Regex;
use lazy_static::lazy_static;

lazy_static!{
    pub static ref UNIT_CODE_FORMAT: Regex = Regex::new(r"([a-zA-Z]{4}\d{4})").unwrap();
    pub static ref SUBCODE_FORMAT: Regex = Regex::new(r"([a-zA-Z]{4}\d{4})-(\w+).+").unwrap();
    pub static ref SEMESTER_FORMAT: Regex = Regex::new(r"S([12]).+").unwrap();
    pub static ref SEMESTER_KEY_FORMAT: Regex = Regex::new(r"Semester (\d+)").unwrap();
}

pub const DEFAULT_HEADLESS: bool = false;
pub const DEFAULT_PORT: u64 = 9515;

pub const MIN_PORT: u64 = 1024;
pub const MAX_PORT: u64 = 65535;

pub const CONFIG_FILE: &'static str = "config.json";
pub const ROWS_IN_TABLE: usize = 12;

pub const PUBLIC_TIMETABLE_ODD: &'static str = "https://timetable.sydney.edu.au/odd/timetable/#subjects";
pub const PUBLIC_TIMETABLE_EVEN: &'static str = "https://timetable.sydney.edu.au/even/timetable/#subjects";
