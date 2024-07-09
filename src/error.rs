use thiserror::Error;

#[derive(Error, Debug)]
#[error("regex {:?} did not match {:?}", .0, .1)]
pub struct RegexNoMatchError<'a>(pub &'a str, pub String);

#[derive(Error, Debug)]
#[error("expected semester {expected}, found semester {actual}")]
pub struct SemesterInvalidError {
    pub expected: u64,
    pub actual: u64,
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