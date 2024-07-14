use std::time::Instant;

use tokio;
use thirtyfour::prelude::*;

use seatfinder::seatfinder::SeatFinder;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let start = Instant::now();

    let seatfinder = match SeatFinder::try_new().await {
        Ok(seatfinder) => seatfinder,
        Err(e) => panic!("Error constructing seatfinder: {}", e),
    };
    seatfinder.seatfind().await;

    let elapsed = start.elapsed();
    println!("Program took {:.2?} seconds to execute", elapsed);
    seatfinder.quit().await?;
    Ok(())
}