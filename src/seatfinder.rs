use std::fs::File;
use std::error::Error;
use std::io::BufReader;

use serde_json::{self, Value};
use thirtyfour::prelude::*;

use crate::constants::CONFIG_FILE;
use crate::error::OfferingError;
use crate::query::FinderConfig;
use crate::offering::{maybe_single_offering, maybe_multiple_offerings};
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
            return match maybe_single_offering(&self.config.query, first_offering) {
                Ok(()) => {
                    let parent = selected_results[0].parent().await?;
                    let checkbox = parent.query(By::XPath(OFFERING_CHECKBOX)).first().await?;
                    Ok(checkbox.click().await?)
                },
                Err(e) => Err(e),
            }
        }
        
        match maybe_multiple_offerings(&self.config.query, &subcodes, &selected_results) {
            Some(element) => {
                let parent = element.parent().await?;
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

        let tutorial_xpath = TUTORIAL_ALLOCATION_FORMAT.format_single_u64(self.config.query.day as u64);
        let events = self.driver
            .query(By::XPath(tutorial_xpath))
            .all_from_selector()
            .await?;

        Ok(self.parse_events(&events).await?)
    }

    pub fn notify_no_allocations_found(&self) {
        println!("No allocations found for {} matching the given query.", self.config.query.unit_code())
    }

    async fn parse_events(&self, timetable_events: &Vec<WebElement>) -> AllocationResult {
        for event in timetable_events {
            event.wait_until().enabled().await?;
            event.wait_until().displayed().await?;
            event.click().await?;

            match self.parse_event(event).await? {
                Some(allocation) => return Ok(Some(allocation)),
                None => {{}},
            };

            let go_back_button = self.query_by_xpath(GO_BACK_BUTTON).await?;
            go_back_button.click().await?;
        }

        Ok(None)
    }

    async fn parse_event(&self, event: &WebElement) -> AllocationResult {
        let tabulated_allocation = event
            .query(By::XPath(ALLOCATIONS_TABLE))
            .all_from_selector()
            .await?;

        let allocation = Allocation::try_new(&tabulated_allocation).await?;
        if allocation.seats > 0 { Ok(Some(allocation)) } else { Ok(None) }
    }

    #[inline]
    async fn query_by_xpath(&self, xpath: XPathStr) -> WebDriverResult<WebElement> {
        self.driver.query(By::XPath(xpath)).first().await
    }
}