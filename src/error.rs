use thiserror::Error;

use crate::allocation::Semester;

#[derive(Error, Debug)]
#[error("expected semester {expected}, found semester {actual}")]
pub struct SemesterInvalidError {
    pub expected: Semester,
    pub actual: Semester,
}

#[derive(Error, Debug)]
pub enum AllocationError {
    #[error("value for key {:?} not found in allocation table", .0)]
    TableRowNotFoundError(String),
    #[error("table must have at least 2 elements per row")]
    TableSizeError,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("an invalid JSON value was encountered")]
    ParseJsonError,
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
    #[error("regex {:?} did not match {:?}", .0, .1)]
    RegexNoMatchError(&'static str, String),
}

#[derive(Error, Debug)]
pub enum OfferingError {
    #[error("offering {:?} in semester {:?} is improperly formatted", .0, .1)]
    SemesterFormatError(String, String),
    #[error("no sessions found for {:?}", .0)]
    NoOfferingsFoundError(String),
    #[error("no valid sessions found for {:?}", .0)]
    NoValidOfferingsFoundError(String),
}