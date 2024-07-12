pub mod constants;
pub mod error;

pub mod query;
pub mod selector;

pub mod offering;
pub mod allocation;

pub mod seatfinder;

use std::time::Instant;

use tokio;
use thirtyfour::prelude::*;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let start = Instant::now();

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
        Err(e) => panic!("Error selecting the unit offering: {}", e),
    }

    match seatfinder.search_query(&interactees).await {
        Ok(o) => match o {
            Some(allocation) => allocation.notify_query_resolved(seatfinder.config.query.unit_code()),
            None => seatfinder.notify_no_allocations_found(),
        }
        Err(e) => panic!("Error searching for the query: {}", e)
    }

    let elapsed = start.elapsed();
    println!("Program took {:.2?} seconds to execute", elapsed);
    Ok(seatfinder.driver.quit().await?)
}