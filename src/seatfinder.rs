use std::fs::File;
use std::error::Error;
use std::io::BufReader;
use std::process::{Child, Command, Stdio};

use serde_json::{self, Value};
use thirtyfour::prelude::*;

use crate::constants::CONFIG_FILE;
use crate::error::OfferingError;
use crate::query::{FinderQuery, FinderConfig};
use crate::offering::{single_offering, multiple_offerings};
use crate::allocation::AllocationResult;
use crate::searcher::TimetableSearcher;
use crate::selector::*;

#[derive(Debug)]
pub struct Interactees {
    pub search_bar: WebElement,
    pub search_button: WebElement,
    pub show_timetable_button: WebElement,
}

#[derive(Debug)]
pub struct SeatFinder {
    driver: WebDriver,
    config: FinderConfig,
    chromedriver: Child,
}

impl SeatFinder {
    pub async fn try_new() -> Result<Self, Box<dyn Error>> {
        let file = File::open(CONFIG_FILE)?;
        let reader = BufReader::new(file);
        
        let json_config: Value = serde_json::from_reader(reader)?;
        let config = FinderConfig::try_new(&json_config)?;

        let chromedriver = Command::new("chromedriver")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg("-p")
            .arg(config.port.to_string())
            .spawn()?;

        let capabilities = DesiredCapabilities::chrome();
        let server_url = format!("http://localhost:{}", config.port);
        let driver = WebDriver::new(server_url, capabilities).await?;
        
        Ok(Self { driver, config, chromedriver })
    }

    pub async fn seatfind(&self) {    
        let queries = &self.config.queries;
        for query in queries.iter() {
            let interactees = match self.locate_interactees().await {
                Ok(interactees) => interactees,
                Err(e) => panic!("Error searching for units: {}", e),
            };

            if let Err(e) = self.search_timetable(&interactees, query).await {
                panic!("Error searching the timetable: {}", e);
            }
        
            if let Err(e) = self.select_unit(query).await {
                panic!("Error selecting the unit offering: {}", e);
            }
        
            match self.search_query(&interactees, query).await {
                Ok(o) => match o {
                    Some(allocation) => allocation.notify_query_resolved(query.unit_code()),
                    None => println!("No allocations found for {} matching the given query.", query.unit_code()),
                }
                Err(e) => panic!("Error searching for the query: {}", e)
            }

            if let Err(e) = self.clear_timetable().await {
                panic!("Error clearing the timetable: {}", e);
            }
        }
    }

    pub async fn locate_interactees(&self) -> WebDriverResult<Interactees> {
        self.driver.goto(&self.config.public_timetable_url).await?;

        let search_bar = self.query_by_xpath(SEARCH_BAR).await?;
        let search_button = self.query_by_xpath(SEARCH_BUTTON).await?;
        let show_timetable_button = self.query_by_xpath(SHOW_TIMETABLE).await?;

        Ok(Interactees { 
            search_bar, 
            search_button, 
            show_timetable_button, 
        })
    }

    pub async fn search_timetable(&self, interactees: &Interactees, query: &FinderQuery) -> WebDriverResult<()> {
        interactees.search_bar.clear().await?;
        interactees.search_bar.send_keys(&query.unit_code).await?;
        interactees.search_button.wait_until().clickable().await?;
        interactees.search_button.click().await?;

        Ok(())
    }

    pub async fn select_unit(&self, query: &FinderQuery) -> Result<(), Box<dyn Error>> {
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
                    OfferingError::NoOfferingsFoundError(query.unit_code())
                )
            ),
        };

        if subcodes.len() == 1 {
            return match single_offering(query, first_offering) {
                Ok(()) => {
                    let parent = selected_results[0].parent().await?;
                    let checkbox = parent.query(By::XPath(OFFERING_CHECKBOX)).first().await?;
                    Ok(checkbox.click().await?)
                },
                Err(e) => Err(e),
            }
        }
        
        match multiple_offerings(query, &subcodes, &selected_results) {
            Some(event) => {
                let parent = event.parent().await?;
                let checkbox = parent.query(By::XPath(OFFERING_CHECKBOX)).first().await?;
                Ok(checkbox.click().await?)
            }
            None => Err(
                Box::new(
                    OfferingError::NoValidOfferingsFoundError(query.unit_code())
                )
            )
        }
    }

    pub async fn search_query(&self, interactees: &Interactees, query: &FinderQuery) -> AllocationResult {
        interactees.show_timetable_button.click().await?;

        let searcher = TimetableSearcher::new(&self.driver, query);
        Ok(searcher.search().await?)
    }

    pub async fn quit(mut self) -> WebDriverResult<()> {
        self.chromedriver.kill().expect("chromedriver could not be killed");
        self.driver.quit().await?;
        Ok(())
    }

    async fn clear_timetable(&self) -> WebDriverResult<()> {
        let clear_button = self.query_by_xpath(CLEAR_BUTTON).await?;
        Ok(clear_button.click().await?)
    }

    #[inline]
    async fn query_by_xpath(&self, xpath: impl Into<String>) -> WebDriverResult<WebElement> {
        self.driver.query(By::XPath(xpath)).first().await
    }
}