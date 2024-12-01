use regex::Regex;
use lazy_static::lazy_static;

lazy_static!{
    pub static ref TWELVE_HOUR_TIME_RE: Regex = Regex::new(r"^([0-9]|1[012])(:[0-5]\d)?(am|pm|AM|PM)$").unwrap();
    pub static ref UNIT_CODE_RE: Regex = Regex::new(r"^([a-zA-Z]{4}\d{4})$").unwrap();
    pub static ref SUBCODE_RE: Regex = Regex::new(r"^([a-zA-Z]{4}\d{4})-(\w+).+$").unwrap();
    pub static ref SEMESTER_RE: Regex = Regex::new(r"^S([12]).+$").unwrap();
    pub static ref SEMESTER_KEY_RE: Regex = Regex::new(r"^Semester (\d+)$").unwrap();
}

pub const TIMED: bool = true;

pub const DEFAULT_RUN_CHROMEDRIVER: bool = false;
pub const DEFAULT_HEADLESS: bool = false;
pub const DEFAULT_PORT: u16 = 9515;

pub const MIN_PORT: u16 = 1024;
pub const MAX_PORT: u16 = 65535;
pub const LOCALHOST: &str = "127.0.0.1";

pub const CONFIG_FILE: &str = "config.json";
pub const ROWS_IN_TABLE: usize = 12;

pub const PUBLIC_TIMETABLE_ODD: &str = "https://timetable.sydney.edu.au/odd/timetable/#subjects";
pub const PUBLIC_TIMETABLE_EVEN: &str = "https://timetable.sydney.edu.au/even/timetable/#subjects";
