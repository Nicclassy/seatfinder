use serde_json::{self, Value};

use crate::allocation::{ActivityType, Day, Semester};
use crate::constants::{
    PUBLIC_TIMETABLE_EVEN, 
    PUBLIC_TIMETABLE_ODD, 
    SUBCODE_FORMAT, 
    UNIT_CODE_FORMAT
};
use crate::error::ParseError;

const PORT: &'static str = "port";
const PARITY: &'static str = "parity";

const UNIT_CODE: &'static str = "unit_code";
const SEMESTER: &'static str = "semester";
const DAY: &'static str = "day";
const ACTIVITIY_TYPE: &'static str = "activity_type";
const ACTIVITY_NUMBER: &'static str = "activity_number";
const QUERY: &'static str = "query";

#[derive(Debug)]
pub struct FinderQuery {
    pub unit_code: String,
    pub semester: Semester,
    pub day: Day,
    pub activity_type: ActivityType,
    pub activity_number: u64,
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

        let day = Day::try_from(
            config[DAY]
            .as_u64()
            .ok_or(ParseError::ParseJsonError)?
        )?;

        let semester = Semester::try_from(
            config[SEMESTER].as_u64().ok_or(ParseError::ParseJsonError)?
        )?;

        let activity_type = ActivityType::try_from(
            config[ACTIVITIY_TYPE].as_str().ok_or(ParseError::ParseJsonError)?
        )?;
        let activity_number = config[ACTIVITY_NUMBER].as_u64().ok_or(ParseError::ParseJsonError)?;
        Ok(FinderQuery { unit_code, day, semester, activity_type, activity_number })
    }

    pub fn unit_code(&self) -> String {
        self.unit_code.clone()
    }
}

#[derive(Debug)]
pub struct FinderConfig {
    pub port: u64,
    pub public_timetable_url: String,
    pub query: FinderQuery,
}

impl FinderConfig {
    pub fn try_new(config: &Value) -> Result<Self, ParseError> {
        let port = config[PORT].as_u64().ok_or(ParseError::ParseJsonError)?;
        if port < 1024 || port > 65535 {
            return Err(ParseError::ParseJsonError);
        }

        let parity = config[PARITY].as_str().ok_or(ParseError::ParseJsonError)?;
        let public_timetable_url = match parity {
            "odd" => PUBLIC_TIMETABLE_ODD.to_owned(),
            "even" => PUBLIC_TIMETABLE_EVEN.to_owned(),
            _ => return Err(ParseError::ParseJsonError),
        };

        let query = FinderQuery::try_new(&config[QUERY])?;
        Ok(Self { port, public_timetable_url, query })
    }
}