use tokio;
use thirtyfour::error::WebDriverResult;

use seatfinder::seatfinder;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    seatfinder::run().await?;
    Ok(())
}