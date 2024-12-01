use thiserror::Error;

use crate::allocation::Semester;

#[derive(Error, Debug)]
pub enum TableRowError {
    #[error("error querying or processing an element")]
    WebElementError,
    #[error("row key is empty")]
    MissingKey,
    #[error("table row should have 2 elements but has {} elements instead", .0)]
    RowSizeError(usize),
}

#[derive(Error, Debug)]
pub enum TableError {
    #[error("expected {} rows in the table but found {}", .0, .1)]
    TableSizeError(usize, usize),
    #[error("value for key {:?} not found in allocation table", .0)] 
    RowMissingError(String),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("an invalid JSON value was encountered")]
    ParseJsonError,
    #[error("parity must be even or odd")]
    ParseParityError,
    #[error("day with isoweekday {:?} is invalid; isoweekday must be between 1 and 7", .0)]
    ParseDayIsoError(u64),
    #[error("{:?} is not a day of the week", .0)]
    ParseDayStrError(String),
    #[error("invalid semester {:?}: semester must be either 1 or 2", .0)]
    ParseSemesterError(u64),
    #[error("invalid semester {:?}", .0)]
    ParseSemesterStrError(String),
    #[error("{:?} cannot be converted into 24 hour time", .0)]
    ParseTimeError(String),
    #[error("invalid activity type {:?}", .0)]
    ParseActivityTypeError(String),
    #[error("an invalid query was encountered.")]
    ParseQueriesError,
    #[error("regex {:?} did not match {:?}", .0, .1)]
    RegexNoMatch(&'static str, String),
}

#[derive(Error, Debug)]
pub enum OfferingError {
    #[error("offering {:?} in semester {:?} is improperly formatted", .0, .1)]
    SemesterFormatError(String, String),
    #[error("expected semester {expected}, found semester {actual}")]
    SemesterInvalid { expected: Semester, actual: Semester },
    #[error("no offerings found for {:?}", .0)]
    NoOfferingsError(String),
    #[error("no valid sessions found for {:?}", .0)]
    NoValidOfferingsError(String),
}