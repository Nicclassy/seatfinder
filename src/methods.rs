use std::error::Error;

use chrono::Datelike;
use thirtyfour::WebElement;

use crate::constants::{
    PUBLIC_TIMETABLE_EVEN, 
    PUBLIC_TIMETABLE_ODD, 
    SEMESTER_FORMAT, 
    SUBCODE_FORMAT
};
use crate::allocation::Semester;
use crate::query::FinderQuery;
use crate::error::{ParseError, OfferingError};

pub fn public_timetable_url_default() -> &'static str {
    let now = chrono::Local::now();
    let year = now.year();
    if year % 2 == 0 { PUBLIC_TIMETABLE_EVEN } else { PUBLIC_TIMETABLE_ODD }
}

pub fn single_offering(query: &FinderQuery, subcode: &String) -> Result<(), Box<dyn Error>> {
    let Some((_, [unit_code, session])) = 
        SUBCODE_FORMAT.captures(&subcode).map(|caps| caps.extract()) else {
            return Err(
                Box::new(
                    ParseError::RegexNoMatchError(
                        SUBCODE_FORMAT.as_str(), 
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

    let semester_match = match SEMESTER_FORMAT.captures(session) {
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
                ParseError::RegexNoMatchError(
                    SEMESTER_FORMAT.as_str(), 
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
                OfferingError::SemesterInvalidError { 
                    expected: query.semester.clone(), 
                    actual: semester 
                } 
            )
        )
    }
}

pub fn multiple_offerings<'a>(
    query: &FinderQuery, 
    subcodes: &'a Vec<String>, 
    elements: &'a Vec<WebElement>
) -> Option<&'a WebElement> {
    let element_position = subcodes
        .iter()
        .position(|subcode| single_offering(query, subcode).is_ok());

    match element_position {
        Some(index) => elements.get(index),
        None => None
    }
}