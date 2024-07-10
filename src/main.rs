pub mod constants;
pub mod error;

pub mod query;
pub mod offering;
pub mod selector;

pub mod seatfinder;

use tokio;
use thirtyfour::prelude::*;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let seatfinder = match seatfinder::SeatFinder::try_new().await {
        Ok(seatfinder) => seatfinder,
        Err(e) => panic!("Error constructing seatfinder: {}", e),
    };

    let interactees = match seatfinder.locate_interactees().await {
        Ok(interactees) => interactees,
        Err(e) => panic!("Error searching for units: {}", e),
    };

    match seatfinder.search_timetable(&interactees).await {
        Ok(()) => {},
        Err(e) => panic!("Error searching the timetable: {}", e),
    }

    match seatfinder.select_unit().await {
        Ok(()) => {},
        Err(e) => panic!("Error selecting the unit session: {}", e),
    }

    seatfinder.driver.quit().await?;
    Ok(())
}