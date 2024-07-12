use std::fs::File;
use std::error::Error;
use std::io::BufReader;

use serde_json::{self, Value};
use thirtyfour::prelude::*;

use crate::constants::CONFIG_FILE;
use crate::error::OfferingError;
use crate::query::FinderConfig;
use crate::offering::{single_offering, multiple_offerings};
use crate::allocation::{Allocation, AllocationResult};
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
    pub config: FinderConfig,
}

impl SeatFinder {
    pub async fn try_new() -> Result<Self, Box<dyn Error>> {
        let file = File::open(CONFIG_FILE)?;
        let reader = BufReader::new(file);
        
        let json_config: Value = serde_json::from_reader(reader)?;
        let config = FinderConfig::try_new(&json_config)?;

        let capabilities = DesiredCapabilities::chrome();
        let server_url = format!("http://localhost:{}", config.port);
        let driver = WebDriver::new(server_url, capabilities).await?;
        
        Ok(Self { driver, config })
    }

    pub async fn locate_interactees(&self) -> WebDriverResult<Interactees> {
        self.driver.goto(self.config.public_timetable_url.clone()).await?;

        let search_bar = self.query_by_xpath(SEARCH_BAR).await?;
        let search_button = self.query_by_xpath(SEARCH_BUTTON).await?;
        let show_timetable_button = self.query_by_xpath(SHOW_TIMETABLE).await?;

        Ok(Interactees { search_bar, search_button, show_timetable_button })
    }

    pub async fn search_timetable(&self, interactees: &Interactees) -> WebDriverResult<()> {
        interactees.search_bar.send_keys(&self.config.query.unit_code).await?;
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
                    OfferingError::NoOfferingsFoundError(self.config.query.unit_code())
                )
            ),
        };

        if subcodes.len() == 1 {
            return match single_offering(&self.config.query, first_offering) {
                Ok(()) => {
                    let parent = selected_results[0].parent().await?;
                    let checkbox = parent.query(By::XPath(OFFERING_CHECKBOX)).first().await?;
                    Ok(checkbox.click().await?)
                },
                Err(e) => Err(e),
            }
        }
        
        match multiple_offerings(&self.config.query, &subcodes, &selected_results) {
            Some(event) => {
                let parent = event.parent().await?;
                let checkbox = parent.query(By::XPath(OFFERING_CHECKBOX)).first().await?;
                Ok(checkbox.click().await?)
            }
            None => Err(
                Box::new(
                    OfferingError::NoValidOfferingsFoundError(self.config.query.unit_code())
                )
            )
        }
    }

    pub async fn search_query(&self, interactees: &Interactees) -> AllocationResult {
        interactees.show_timetable_button.click().await?;

        Ok(self.find_matching_event().await?)
    }

    pub fn notify_no_allocations_found(&self) {
        println!("No allocations found for {} matching the given query.", self.config.query.unit_code())
    }

    async fn find_matching_event(&self) -> AllocationResult {
        let mut n_parsed_events = 1;
        let query = &self.config.query;
        let column = format_u64(ALLOCATION_FORMAT.into(), query.day as u64);

        while let Some(event) = self.find_event(&column, n_parsed_events).await {
            event.click().await?;
            
            let allocation = self.try_parse_allocation(&event).await?;
            if allocation.activity == query.activity_number {
                return Ok(if allocation.seats > 0 { Some(allocation) } else { None })
            }
            
            let go_back_button = self.query_by_xpath(GO_BACK_BUTTON).await?;
            go_back_button.click().await?;
            n_parsed_events += 1;
        }

        Ok(None)
    }

    async fn try_parse_allocation(&self, event: &WebElement) -> Result<Allocation, Box<dyn Error>> {
        let tabulated_allocation = event
            .query(By::XPath(ALLOCATIONS_TABLE))
            .all_from_selector()
            .await?;

        let allocation = Allocation::try_new(&tabulated_allocation).await?;
        Ok(allocation)
    }

    #[inline]
    async fn query_by_xpath(&self, xpath: impl Into<String>) -> WebDriverResult<WebElement> {
        self.driver.query(By::XPath(xpath)).first().await
    }

    #[inline]
    async fn find_event(&self, column: &str, row: u64) -> Option<WebElement> {
        let by = By::XPath(format_u64(&column, row));
        let event = self.driver.query(by).first().await.ok();
        event
    }
}