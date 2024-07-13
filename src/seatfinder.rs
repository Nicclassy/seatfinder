use std::collections::HashMap;
use std::fs::File;
use std::error::Error;
use std::io::BufReader;

use serde_json::{self, Value};
use thirtyfour::prelude::*;

use crate::constants::{ROWS_IN_TABLE, CONFIG_FILE};
use crate::error::{AllocationError, OfferingError};
use crate::query::{FinderQuery, FinderConfig};
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
                    None => self.notify_no_allocations_found(query),
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

        Ok(self.find_matching_event(query).await?)
    }

    pub fn notify_no_allocations_found(&self, query: &FinderQuery) {
        println!("No allocations found for {} matching the given query.", query.unit_code())
    }

    async fn find_matching_event(&self, query: &FinderQuery) -> AllocationResult {
        let mut timetable_row = 1;
        let table_column = format_u64(ALLOCATION_FORMAT.as_str(), query.day as u64);

        while let Ok(ref event) = self.find_event(&table_column, timetable_row).await {
            event.click().await?;
            
            let allocation = self.allocation_from_table(&table_column, timetable_row).await?;
            if allocation.activity == query.activity_number {
                return Ok(if allocation.seats > 0 { Some(allocation) } else { None })
            }
            
            self.go_back_to_timetable().await?;
            timetable_row += 1;
        }

        Ok(None)
    }

    async fn allocation_from_table(&self, column: &str, row: u64) -> Result<Allocation, Box<dyn Error>> {
        let mut allocation_table = HashMap::new();
        let mut table_rows = self.table_rows().await?; 
        
        let mut table_row_number = 0;
        while table_row_number < ROWS_IN_TABLE {
            let table_row = &table_rows[table_row_number];
            let children = table_row
                .query(By::Css("*"))
                .all_from_selector()
                .await;

            let reload_table = match children {
                Ok(children) if children.len() == 2 => {
                    let first = children[0].text().await;
                    let second = children[1].text().await;
                    match (first, second) {
                        (Ok(table_key), Ok(table_value)) => {
                            allocation_table.insert(table_key, table_value);
                            false
                        }
                        _ => true,
                    }
                },
                Ok(_) => return Err(Box::new(AllocationError::TableSizeError)),
                Err(_) => true,
            };

            if reload_table {
                self.go_back_to_timetable().await?;
                let event = self.find_event(&column, row).await?;
                event.click().await?;
                table_rows = self.table_rows().await?;
            } else {
                table_row_number += 1
            }
        }

        if allocation_table.len() != ROWS_IN_TABLE {
            return Err(Box::new(AllocationError::TableSizeError));
        }

        let allocation = Allocation::try_new(&allocation_table).await?;
        Ok(allocation)
    }

    #[inline]
    async fn query_by_xpath(&self, xpath: impl Into<String>) -> WebDriverResult<WebElement> {
        self.driver.query(By::XPath(xpath)).first().await
    }

    async fn clear_timetable(&self) -> WebDriverResult<()> {
        let clear_button = self.query_by_xpath(CLEAR_BUTTON).await?;
        Ok(clear_button.click().await?)
    }

    async fn go_back_to_timetable(&self) -> WebDriverResult<()> {
        let go_back_button = self.query_by_xpath(GO_BACK_BUTTON).await?;
        Ok(go_back_button.click().await?)
    }

    #[inline]
    async fn table_rows(&self) -> WebDriverResult<Vec<WebElement>> {
        self.driver
            .query(By::XPath(ALLOCATION_TABLE_ROWS))
            .all_from_selector()
            .await
    }

    #[inline]
    async fn find_event(&self, column: &str, row: u64) -> WebDriverResult<WebElement> {
        let by = By::XPath(format_u64(&column, row));
        self.driver.query(by).first().await
    }
}