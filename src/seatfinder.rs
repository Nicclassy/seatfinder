use std::fs::File;
use std::error::Error;
use std::io::BufReader;
use std::time::Instant;
use std::process::Child;

use serde_json::{self, Value};
use thirtyfour::prelude::*;

use crate::constants::{CONFIG_FILE, TIMED};
use crate::error::OfferingError;
use crate::query::{FinderQuery, FinderConfig};
use crate::methods::{
    format_str, 
    chromedriver_process, 
    parse_queries,
    single_offering,
    multiple_offerings, 
};
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
    chromedriver: Option<Child>,
    queries: Vec<FinderQuery>,
}

impl SeatFinder {
    pub async fn new() -> Self {
        match SeatFinder::try_new().await {
            Ok(seatfinder) => seatfinder,
            Err(e) => panic!("Error constructing seatfinder: {}", e),
        }
    }

    pub async fn try_new() -> Result<Self, Box<dyn Error>> {
        let file = File::open(CONFIG_FILE)?;
        let reader = BufReader::new(file);
        
        let json_config: Value = serde_json::from_reader(reader)?;
        let config = FinderConfig::try_new(json_config)?;

        let chromedriver = if config.run_chromedriver {
            Some(chromedriver_process(config.port)?)
        } else {
            None
        };

        let mut capabilities = DesiredCapabilities::chrome();
        if config.headless {
            capabilities.add_arg("--headless")?;
        }
        
        let server_url = format!("http://localhost:{}", config.port);
        let driver = match WebDriver::new(server_url, capabilities).await {
            Ok(driver) => driver,
            Err(e) => {
                if let Some(mut child) = chromedriver {
                    child.kill()?;
                }
                return Err(Box::new(e));
            }
        };
        
        let queries = match parse_queries() {
            Ok(queries) => queries,
            Err(e) => {
                if let Some(mut child) = chromedriver {
                    child.kill()?;
                }
                return Err(e);
            }
        };

        Ok(Self { driver, config, chromedriver, queries })
    }

    pub fn add_query(&mut self, query: FinderQuery) -> &mut Self {
        self.queries.push(query);
        self
    }

    pub async fn seatfind(&self) {   
        if self.queries.is_empty() {
            println!("No queries to find.");
            return;
        } 

        for query in self.queries.iter() {
            let interactees = match self.locate_interactees().await {
                Ok(interactees) => interactees,
                Err(e) => panic!("Error searching for units: {}", e),
            };

            if let Err(e) = self.toggle_advanced_filter(query).await {
                panic!("Error toggling advanced filter: {}", e);
            }

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

            if let Err(e) = self.reset_timetable().await {
                panic!("Error clearing the timetable: {}", e);
            }
        }
    }

    pub async fn quit(self) {
        self.driver.quit().await.expect("webdriver did not succesfully quit");
        if let Some(mut child) = self.chromedriver {
            child.kill().expect("chromedriver did not succesfully quit");
        }
    }
}

impl SeatFinder {
    async fn locate_interactees(&self) -> WebDriverResult<Interactees> {
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

    async fn toggle_advanced_filter(&self, query: &FinderQuery) -> WebDriverResult<()> {
        let checkbox_id = format_str(
            ACTIVITY_CHECKBOX_FORMAT.as_str(), 
            query.activity_type.checkbox_id_suffix()
        );
        let by = By::Id(checkbox_id);
        
        let activity_checkbox = self.driver
            .query(by)
            .first()
            .await?;
        Ok(activity_checkbox.click().await?)
    }

    async fn search_timetable(&self, interactees: &Interactees, query: &FinderQuery) -> WebDriverResult<()> {
        interactees.search_bar.clear().await?;
        interactees.search_bar.send_keys(&query.unit_code).await?;
        interactees.search_button.wait_until().clickable().await?;
        Ok(interactees.search_button.click().await?)
    }

    async fn select_unit(&self, query: &FinderQuery) -> Result<(), Box<dyn Error>> {
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
                    OfferingError::NoOfferingsError(query.unit_code())
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
                    OfferingError::NoValidOfferingsError(query.unit_code())
                )
            )
        }
    }

    async fn search_query(&self, interactees: &Interactees, query: &FinderQuery) -> AllocationResult {
        interactees.show_timetable_button.click().await?;

        let searcher = TimetableSearcher::new(&self.driver, query);
        Ok(searcher.search().await?)
    }

    async fn reset_timetable(&self) -> WebDriverResult<()> {
        self.clear_timetable().await?;
        self.reselect_all().await?;
        Ok(())
    }

    async fn clear_timetable(&self) -> WebDriverResult<()> {
        let clear_button = self.query_by_xpath(CLEAR_BUTTON).await?;
        Ok(clear_button.click().await?)
    }

    async fn reselect_all(&self) -> WebDriverResult<()> {
        let checkbox_id = format_str(
            ACTIVITY_CHECKBOX_FORMAT.as_str(), 
            "ALL"
        );
        let by = By::Id(checkbox_id);
        
        let activity_checkbox = self.driver
            .query(by)
            .first()
            .await?;
        Ok(activity_checkbox.click().await?)
    }

    #[inline]
    async fn query_by_xpath(&self, xpath: impl Into<String>) -> WebDriverResult<WebElement> {
        self.driver.query(By::XPath(xpath)).first().await
    }
}

pub async fn run() -> WebDriverResult<()> {
    let start = if TIMED { Some(Instant::now()) } else { None };
    
    let seatfinder = SeatFinder::new().await;
    seatfinder.seatfind().await;
    seatfinder.quit().await;
    
    if let Some(instant) = start {
        println!("Program took {:.2?} seconds to execute", instant.elapsed());
    }
    Ok(())
}