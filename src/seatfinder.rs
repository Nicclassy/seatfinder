use std::fs::File;
use std::error::Error;
use std::io::{self, BufReader};
use std::thread;
use std::time::Duration;

use serde_json::{self, Value};
use thirtyfour::prelude::*;

use crate::constants::{CONFIG_FILE, PUBLIC_TIMETABLE_URL, PORT};
use crate::error::OfferingError;
use crate::query::FinderQuery;
use crate::offering::{maybe_single_offering, maybe_multiple_offerings};
use crate::selector::*;

#[derive(Debug)]
pub struct Interactees {
    pub search_bar: WebElement,
    pub search_button: WebElement,
    pub show_timetable_button: WebElement,
}

#[derive(Debug)]
pub struct SeatFinder {
    pub driver: WebDriver,
    pub query: FinderQuery,
}

impl SeatFinder {
    pub async fn try_new() -> Result<SeatFinder, Box<dyn Error>> {
        let file = File::open(CONFIG_FILE)?;
        let reader = BufReader::new(file);
        
        let config: Value = serde_json::from_reader(reader)?;
        let query = match FinderQuery::try_new(&config) {
            Some(query) => query,
            None => return Err(
                Box::new(
                    io::Error::new(
                        io::ErrorKind::InvalidInput, 
                        "could not construct query from the provided JSON file."
                    )
                )
            )
        };
        let capabilities = DesiredCapabilities::chrome();
        let server_url = format!("http://localhost:{}", PORT);
        let driver = WebDriver::new(server_url, capabilities).await?;
        
        Ok(Self { driver, query })
    }

    pub async fn locate_interactees(&self) -> WebDriverResult<Interactees> {
        self.driver.goto(PUBLIC_TIMETABLE_URL).await?;

        let search_bar = self.query_by_xpath(SEARCH_BAR).await?;
        let search_button = self.query_by_xpath(SEARCH_BUTTON).await?;
        let show_timetable_button = self.query_by_xpath(SHOW_TIMETABLE).await?;

        Ok(Interactees { search_bar, search_button, show_timetable_button })
    }

    pub async fn search_timetable(&self, interactees: &Interactees) -> WebDriverResult<()> {
        interactees.search_bar.send_keys(&self.query.unit_code).await?;
        interactees.search_button.wait_until().clickable().await?;
        interactees.search_button.click().await?;

        Ok(())
    }

    pub async fn select_unit(&self) -> Result<(), Box<dyn Error>> {
        let selected_results = self.driver
            .query(By::XPath(UNIT_OFFERINGS))
            .all_from_selector()
            .await?;
        let mut subcodes = Vec::with_capacity(selected_results.len());

        for offering in selected_results.iter() {
            let offering_subcode = offering.text().await?;
            subcodes.push(offering_subcode);
        }

        let first_offering = match subcodes.first() {
            Some(offering) => offering,
            None => return Err(
                Box::new(
                    OfferingError::NoOfferingsFoundError(self.query.unit_code())
                )
            ),
        };

        if subcodes.len() == 1 {
            return match maybe_single_offering(&self.query, first_offering) {
                Ok(()) => {
                    let parent = selected_results[0].parent().await?;
                    let checkbox = parent.query(By::XPath(OFFERING_CHECKBOX)).first().await?;
                    Ok(checkbox.click().await?)
                },
                Err(e) => Err(e),
            }
        }
        
        match maybe_multiple_offerings(&self.query, &subcodes, &selected_results) {
            Some(element) => {
                let parent = element.parent().await?;
                let checkbox = parent.query(By::XPath(OFFERING_CHECKBOX)).first().await?;
                Ok(checkbox.click().await?)
            }
            None => Err(
                Box::new(
                    OfferingError::NoValidOfferingsFoundError(self.query.unit_code())
                )
            )
        }
    }

    #[inline]
    async fn query_by_xpath(&self, xpath: XPathStr) -> WebDriverResult<WebElement> {
        self.driver.query(By::XPath(xpath)).first().await
    }
}