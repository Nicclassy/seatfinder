use std::error::Error;
use std::collections::HashMap;

use strum::{Display, IntoStaticStr};
use thirtyfour::prelude::*;

use crate::constants::SEMESTER_TABLE_FORMAT;
use crate::error::{AllocationError, ParseError};

const ALLOCATION_TABLE_ROWS: u8 = 12;

pub type AllocationResult = Result<Option<Allocation>, Box<dyn Error>>;

#[derive(Debug)]
pub struct TwentyFourHourTime {
    pub hours: u8,
    pub minutes: u8,
}

impl TwentyFourHourTime {
    pub fn new(value: String) -> Option<Self> {
        let (hrs, mins) = value.split_once(':')?;
        let hours = hrs.parse::<u8>().ok()?;
        let minutes = mins.parse::<u8>().ok()?;

        Some(Self { hours, minutes })
    }
}

#[derive(Debug, Display, PartialEq, Clone)]
pub enum Semester {
    One = 1,
    Two = 2,
}

impl TryFrom<u64> for Semester {
    type Error = ParseError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
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
        
        let Some(caps) = SEMESTER_TABLE_FORMAT.captures(&value) else {
            return Err(ParseError::ParseSemesterStrError(value));
        };

        let semester_number = match (&caps[1]).parse::<u64>() {
            Ok(semester_number) => semester_number,
            Err(e) => return Err(
                ParseError::ParseSemesterStrError(e.to_string())
            ),
        };

        match semester_number {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            _ => Err(ParseError::ParseSemesterError(semester_number))
        }
    }
}

#[derive(Debug, IntoStaticStr)]
pub enum ActivityType {
    Lab,
    Tutorial,
    Workshop,
    Practical,
}

impl TryFrom<&str> for ActivityType {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Lab" => Ok(ActivityType::Lab),
            "Tutorial" => Ok(ActivityType::Tutorial),
            "Workshop" => Ok(ActivityType::Workshop),
            "Practical" => Ok(ActivityType::Practical),
            _ => Err(ParseError::ParseActivityTypeError(value.to_string())),
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
            "Mon" => Ok(Day::Monday),
            "Tue" => Ok(Day::Tuesday),
            "Wed" => Ok(Day::Wednesday),
            "Thu" => Ok(Day::Thursday),
            "Fri" => Ok(Day::Friday),
            "Sat" => Ok(Day::Saturday),
            "Sun" => Ok(Day::Sunday),
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

fn allocation_table_get(map: &HashMap<String, String>, key: &str) -> Result<String, AllocationError> {
    map.get(key).ok_or(AllocationError::TableRowNotFoundError(key.to_string())).cloned()
}

impl Allocation {
    pub async fn try_new(table_rows: &Vec<WebElement>) -> Result<Allocation, Box<dyn Error>> {
        if table_rows.len() != ALLOCATION_TABLE_ROWS as usize {
            panic!("expected {} elements got {}", ALLOCATION_TABLE_ROWS as usize, table_rows.len());
        }

        let mut mapped_rows: HashMap<String, String> = HashMap::new();
        for row in table_rows {
            let children = row
                .query(By::Css("*"))
                .all_from_selector()
                .await?;

            if children.len() < 2 {
                return Err(Box::new(AllocationError::TableSizeError));
            }
            let table_key = children[0].text().await?;
            let allocation_value = children[1].text().await?;
            mapped_rows.insert(table_key, allocation_value);
        }

        let activity_type = ActivityType::try_from(
            allocation_table_get(&mapped_rows, "Activity Type")?.as_str()
        )?;

        let group = allocation_table_get(&mapped_rows, "Group")?;
        let activity = allocation_table_get(&mapped_rows, "Activity")?.parse::<u64>()?;
        let description = allocation_table_get(&mapped_rows, "Description")?;

        let day = Day::try_from(
            allocation_table_get(&mapped_rows, "Day")?.as_str()
        )?;
        let time_string = allocation_table_get(&mapped_rows, "Time")?.to_string();
        let time = TwentyFourHourTime::new(time_string.clone())
            .ok_or(ParseError::ParseTimeError(time_string))?;

        let semester = Semester::try_from(
            allocation_table_get(&mapped_rows, "Semester")?
        )?;
        let campus = allocation_table_get(&mapped_rows, "Campus")?;
        let location = allocation_table_get(&mapped_rows, "Location")?;

        let duration = allocation_table_get(&mapped_rows, "Duration")?;
        let weeks = allocation_table_get(&mapped_rows, "Weeks")?;
        let seats = allocation_table_get(&mapped_rows, "Seats")?.parse::<u16>()?;

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