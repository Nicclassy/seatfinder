use strum::IntoStaticStr;
use serde_json::{self, Value};

const UNIT: &'static str = "unit_code";
const DAY: &'static str = "day";
const ACTIVITIY_TYPE: &'static str = "activity_type";
const ACTIVITY_NUMBER: &'static str = "activity_number";

#[derive(Debug, IntoStaticStr)]
pub enum ActivityType {
    Lab,
    Tutorial,
    Workshop,
    Practical,
}

#[derive(Debug)]
pub struct FinderQuery {
    pub unit_code: String,
    pub day: u64,
    pub activity_type: ActivityType,
    pub activity_number: u64,
}

impl FinderQuery {
    pub fn try_new(config: &Value) -> Option<Self> {
        let unit = config[UNIT].as_str()?.to_owned();
        let day = config[DAY].as_u64()?;
        let activity_type = match config[ACTIVITIY_TYPE].as_str()? {
            "Lab" => ActivityType::Lab,
            "Tutorial" => ActivityType::Tutorial,
            "Workshop" => ActivityType::Workshop,
            "Practical" => ActivityType::Practical,
            _ => return None,
        };
        let activity_number = config[ACTIVITY_NUMBER].as_u64()?;

        Some(FinderQuery { unit_code: unit, day, activity_type, activity_number })
    }

    pub fn unit_code(&self) -> String {
        self.unit_code.clone()
    }
}