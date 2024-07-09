use std::error::Error;

use thirtyfour::WebElement;

use crate::constants::{UNIT_CODE_FORMAT, SEMESTER_FORMAT, SEMESTER};
use crate::query::FinderQuery;
use crate::error::{RegexNoMatchError, SemesterInvalidError, OfferingError};

pub(crate) fn maybe_single_offering(query: &FinderQuery, subcode: &String) -> Result<(), Box<dyn Error>> {
    let Some((_, [unit_code, session])) = 
        UNIT_CODE_FORMAT.captures(&subcode).map(|caps| caps.extract()) else {
            return Err(
                Box::new(
                    RegexNoMatchError(UNIT_CODE_FORMAT.as_str(), subcode.to_string())
                )
            )
    };

    if unit_code != query.unit_code {
        return Err(
            Box::new(
                OfferingError::NoValidOfferingsFoundError(query.unit_code())
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
                RegexNoMatchError(
                    SEMESTER_FORMAT.as_str(), 
                    session.to_string()
                )
            )
        )
    };

    let semester = match semester_match.as_str().parse::<u64>() {
        Ok(semester) => semester,
        Err(e) => return Err(Box::new(e)),
    };

    match semester == SEMESTER {
        true => Ok(()),
        false => Err(
            Box::new(
                SemesterInvalidError { expected: SEMESTER, actual: semester } 
            )
        )
    }
}

pub(crate) fn maybe_multiple_offerings<'a>(
    query: &FinderQuery, 
    subcodes: &'a Vec<String>, 
    elements: &'a Vec<WebElement>
) -> Option<&'a WebElement> {
    let element_position = subcodes
        .iter()
        .position(|o| maybe_single_offering(query, o).is_ok());

    match element_position {
        Some(i) => elements.get(i),
        None => None
    }
}