use std::error::Error;
use std::collections::HashMap;
use std::fmt;

use strum::{Display, IntoStaticStr};

use crate::constants::{SEMESTER_KEY_RE, TWELVE_HOUR_TIME_RE};
use crate::error::{ParseError, TableError};

pub type AllocationResult = Result<Option<Allocation>, Box<dyn Error>>;

#[derive(Debug)]
pub struct TwentyFourHourTime {
    pub hours: u8,
    pub minutes: u8,
}

impl fmt::Display for TwentyFourHourTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:0>2}:{:0>2}", self.hours, self.minutes)
    }
}

impl TwentyFourHourTime {
    pub fn new(value: &str) -> Option<Self> {
        Self::from_twenty_four_hr(value).or_else(|| Self::from_twelve_hour(value))
    }

    pub fn from_twenty_four_hr(value: &str) -> Option<Self> {
        let (hrs, mins) = value.split_once(':')?;
        let hours = hrs.parse::<u8>().ok()?;
        let minutes = mins.parse::<u8>().ok()?;

        if hours > 23 || minutes > 59 {
            None
        } else {
            Some(Self { hours, minutes })
        }
    }

    pub fn from_twelve_hour(value: &str) -> Option<Self> {
        let (mut hours, minutes, period) = match TWELVE_HOUR_TIME_RE.captures(value) {
            Some(caps) => {
                let hours = caps[1].parse::<u8>().ok()?;
                let minutes = match caps.get(2) {
                    Some(value) => (value.as_str()[1..]).parse::<u8>().ok()?,
                    None => 0,
                };
                let period = caps[3].to_string();
                (hours, minutes, period)
            }
            None => return None,
        };

        if period.to_ascii_lowercase() == "pm" {
            hours += 12;
        }

        Some(TwentyFourHourTime { hours, minutes })
    }

    pub fn progress_one_hour(&self) -> Self {
        Self { hours: self.hours + 1, minutes: self.minutes }
    }
}

#[derive(Debug, Display, PartialEq, Clone)]
pub enum Semester {
    Any = 0,
    One = 1,
    Two = 2,
}

impl TryFrom<u64> for Semester {
    type Error = ParseError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Any),
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            _ => Err(ParseError::ParseSemesterError(value))
        }
    }
}

impl TryFrom<String> for Semester {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.parse::<u64>() {
            Ok(semester) => return Self::try_from(semester),
            Err(_) => {{}},
        }
        
        let Some(caps) = SEMESTER_KEY_RE.captures(&value) else {
            return Err(ParseError::ParseSemesterStrError(value));
        };

        match (&caps[1]).parse::<u64>() {
            Ok(semester) => Self::try_from(semester),
            Err(e) => Err(
                ParseError::ParseSemesterStrError(e.to_string())
            ),
        }
    }
}

#[derive(IntoStaticStr, Debug, Clone, Copy)]
pub enum ActivityType {
    Assesment,
    CompulsoryLecture,
    Fieldwork,
    Film,
    Lab,
    Lecture,
    Online,
    OnlineLive,
    Optional,
    Other,
    Practical,
    Presentation,
    Seminar,
    Studio,
    Tutorial,
    Workshop,
}

impl TryFrom<&str> for ActivityType {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Assesment" => Ok(ActivityType::Assesment),
            "Compulsory Lecture" => Ok(ActivityType::CompulsoryLecture),
            "Fieldwork" => Ok(ActivityType::Fieldwork),
            "Film" => Ok(ActivityType::Film),
            "Lab" => Ok(ActivityType::Lab),
            "Lecture" => Ok(ActivityType::Lecture),
            "Online" => Ok(ActivityType::Online),
            "Online (live)" => Ok(ActivityType::OnlineLive),
            "Optional" => Ok(ActivityType::Optional),
            "Other" => Ok(ActivityType::Other),
            "Practical" => Ok(ActivityType::Practical),
            "Presentation" => Ok(ActivityType::Presentation),
            "Seminar" => Ok(ActivityType::Seminar),
            "Studio" => Ok(ActivityType::Studio),
            "Tutorial" => Ok(ActivityType::Tutorial),
            "Workshop" => Ok(ActivityType::Workshop),
            _ => Err(ParseError::ParseActivityTypeError(value.to_string())),
        }
    }
}

impl ActivityType {
    pub fn checkbox_id_suffix(&self) -> &'static str {
        match self {
            Self::CompulsoryLecture => "Compulsory Lecture",
            Self::OnlineLive => "Online (live)",
            _ => self.into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Day {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6, 
    Sunday = 7,
}

impl TryFrom<u64> for Day {
    type Error = ParseError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Day::Monday),
            2 => Ok(Day::Tuesday),
            3 => Ok(Day::Wednesday),
            4 => Ok(Day::Thursday),
            5 => Ok(Day::Friday),
            6 => Ok(Day::Saturday),
            7 => Ok(Day::Sunday),
            _ => Err(ParseError::ParseDayIsoError(value)),
        }
    }
}

impl TryFrom<&str> for Day {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Monday"    | "Mon" => Ok(Day::Monday),
            "Tuesday"   | "Tue" => Ok(Day::Tuesday),
            "Wednesday" | "Wed" => Ok(Day::Wednesday),
            "Thursday"  | "Thu" => Ok(Day::Thursday),
            "Friday"    | "Fri" => Ok(Day::Friday),
            "Saturday"  | "Sat" => Ok(Day::Saturday),
            "Sunday"    | "Sun" => Ok(Day::Sunday),
            _ => Err(ParseError::ParseDayStrError(value.to_string())),
        }
    }
}

#[derive(Debug)]
pub struct Allocation {
    pub activity_type: ActivityType,
    pub group: String,
    pub activity: u64,
    pub description: String,

    pub day: Day,
    pub time: TwentyFourHourTime,

    pub semester: Semester,
    pub campus: String,
    pub location: String,

    pub duration: String,
    pub weeks: String,
    pub seats: u16,
}

fn allocation_table_get(map: &HashMap<String, String>, key: &str) -> Result<String, TableError> {
    map.get(key).ok_or(TableError::RowMissingError(key.to_owned())).cloned()
}

impl Allocation {
    pub fn try_new(table: &HashMap<String, String>) -> Result<Allocation, Box<dyn Error>> {
        let activity_type = ActivityType::try_from(
            allocation_table_get(&table, "Activity Type")?.as_str()
        )?;

        let group = allocation_table_get(&table, "Group")?;
        let activity = allocation_table_get(&table, "Activity")?.parse::<u64>()?;
        let description = allocation_table_get(&table, "Description")?;

        let day = Day::try_from(
            allocation_table_get(&table, "Day")?.as_str()
        )?;
        let time_string = allocation_table_get(&table, "Time")?;
        let time = TwentyFourHourTime::new(&time_string)
            .ok_or(ParseError::ParseTimeError(time_string))?;

        let semester = Semester::try_from(
            allocation_table_get(&table, "Semester")?
        )?;
        let campus = allocation_table_get(&table, "Campus")?;
        let location = allocation_table_get(&table, "Location")?;

        let duration = allocation_table_get(&table, "Duration")?;
        let weeks = allocation_table_get(&table, "Weeks")?;
        let seats = allocation_table_get(&table, "Seats")?.parse::<u16>()?;

        Ok(Allocation {
            activity_type,
            group,
            activity,
            description,
            day,
            time,
            semester,
            campus,
            location,
            duration,
            weeks,
            seats
        })
    }

    pub fn notify_query_resolved(&self, unit_code: String) {
        println!("Activity {} of {} has {} seats left", self.activity, unit_code, self.seats);
    }
}