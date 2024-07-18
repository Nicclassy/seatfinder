use serde_json::{self, Value};

use crate::allocation::{ActivityType, Day, Semester};
use crate::constants::{
    DEFAULT_HEADLESS, 
    DEFAULT_PORT, 
    DEFAULT_RUN_CHROMEDRIVER, 
    MIN_PORT, 
    MAX_PORT, 
    PUBLIC_TIMETABLE_EVEN, 
    PUBLIC_TIMETABLE_ODD, 
    SUBCODE_FORMAT, 
    UNIT_CODE_FORMAT
};
use crate::error::ParseError;
use crate::methods::{port_is_occupied, public_timetable_url_default, unoccupied_port};

const HEADLESS: &'static str = "headless";
const PORT: &'static str = "port";
const PARITY: &'static str = "parity";
const RUN_CHROMEDRIVER: &'static str = "run_chromedriver";

const UNIT_CODE: &'static str = "unit_code";
const SEMESTER: &'static str = "semester";
const DAY: &'static str = "day";
const ACTIVITIY_TYPE: &'static str = "activity_type";
const ACTIVITY: &'static str = "activity";

#[derive(Debug)]
pub struct FinderQuery {
    pub unit_code: String,
    pub semester: Semester,
    pub day: Day,
    pub activity_type: ActivityType,
    pub activity: u64,
}

impl FinderQuery {
    pub fn try_new(config: &Value) -> Result<Self, ParseError> {
        let unit_code = config[UNIT_CODE]
            .as_str()
            .ok_or(ParseError::ParseJsonError)?
            .to_string();
        if !UNIT_CODE_FORMAT.is_match(unit_code.as_str()) {
            return Err(ParseError::RegexNoMatchError(SUBCODE_FORMAT.as_str(), unit_code));
        }
        let day = match config[DAY].as_u64() {
            Some(value) => Day::try_from(value)?,
            None => match config[DAY].as_str() {
                Some(value) => Day::try_from(value)?,
                None => return Err(ParseError::ParseJsonError),
            }
        };

        let semester = match config[SEMESTER].as_u64() {
            Some(semester) => Semester::try_from(semester)?,
            None => Semester::Any,
        };

        let activity_type = ActivityType::try_from(
            config[ACTIVITIY_TYPE].as_str().ok_or(ParseError::ParseJsonError)?
        )?;
        let activity = config[ACTIVITY].as_u64().ok_or(ParseError::ParseJsonError)?;

        Ok(FinderQuery { 
            unit_code, 
            day, 
            semester, 
            activity_type, 
            activity 
        })
    }

    pub fn unit_code(&self) -> String {
        self.unit_code.clone()
    }
}

#[derive(Debug)]
pub struct FinderConfig {
    pub port: u16,
    pub public_timetable_url: String,
    pub headless: bool,
    pub run_chromedriver: bool,
}

impl FinderConfig {
    pub fn try_new(json_config: Value) -> Result<Self, ParseError> {
        let headless = match json_config.get(HEADLESS) {
            Some(value) => value.as_bool().ok_or(ParseError::ParseJsonError)?,
            None => DEFAULT_HEADLESS,
        };

        let run_chromedriver = match json_config.get(RUN_CHROMEDRIVER) {
            Some(value) => value.as_bool().ok_or(ParseError::ParseJsonError)?,
            None => DEFAULT_RUN_CHROMEDRIVER,
        };

        let mut port = match json_config[PORT].as_u64() {
            Some(port) => port as u16,
            None => DEFAULT_PORT,
        };

        if port < MIN_PORT || port > MAX_PORT {
            return Err(ParseError::ParseJsonError);
        }

        if run_chromedriver && port_is_occupied(port) {
            port = unoccupied_port(DEFAULT_PORT);
        }

        let parity = json_config[PARITY].as_str().unwrap_or("default");
        let public_timetable_url = match parity {
            "odd" => PUBLIC_TIMETABLE_ODD.to_owned(),
            "even" => PUBLIC_TIMETABLE_EVEN.to_owned(),
            "default" => public_timetable_url_default().to_owned(),
            _ => return Err(ParseError::ParseParityError),
        };

        Ok(Self { port, public_timetable_url, headless, run_chromedriver })
    }
}