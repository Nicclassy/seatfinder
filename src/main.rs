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
    seatfinder.seatfind().await;

    let elapsed = start.elapsed();
    println!("Program took {:.2?} seconds to execute", elapsed);
    Ok(seatfinder.driver.quit().await?)
}