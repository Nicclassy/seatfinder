use std::error::Error;
use std::collections::HashMap;

use thirtyfour::prelude::*;

use crate::constants::ROWS_IN_TABLE;
use crate::error::{TableRowError, TableError};
use crate::query::FinderQuery;
use crate::allocation::{Allocation, AllocationResult};
use crate::selector::*;

pub struct TimetableSearcher<'a> {
    driver: &'a WebDriver,
    query: &'a FinderQuery
}

impl<'a> TimetableSearcher<'a> {
    pub fn new(driver: &'a WebDriver, query: &'a FinderQuery) -> Self {
        Self { driver, query }
    }

    pub async fn search(&self) -> AllocationResult {
        let mut timetable_row = 1;
        let timetable_column = format_u64(ALLOCATION_FORMAT.as_str(), self.query.day as u64);

        while let Ok(ref event) = self.timetabled_event(&timetable_column, timetable_row).await {
            event.click().await?;
            
            let allocation = self.allocation_from_table(&timetable_column, timetable_row).await?;
            if allocation.activity == self.query.activity {
                return Ok(if allocation.seats > 0 { Some(allocation) } else { None })
            }
            
            self.go_back_to_timetable().await?;
            timetable_row += 1;
        }

        Ok(None)
    }

    async fn allocation_from_table(&self, timetable_column: &str, timetable_row: u64) -> Result<Allocation, Box<dyn Error>> {
        let mut allocation_table = HashMap::with_capacity(ROWS_IN_TABLE);
        let mut table_rows = self.table_rows().await?; 
        let mut table_row_number = 0;
        
        while table_row_number < ROWS_IN_TABLE {
            let table_row = &table_rows[table_row_number];
            let row_query = table_row
                .query(By::Css("*"))
                .all_from_selector()
                .await;

            let insert_row = match row_query {
                Ok(row_elements) if row_elements.len() == 2 => {
                    let first = row_elements[0].text().await;
                    let second = row_elements[1].text().await;
                    match (first, second) {
                        (Ok(table_key), Ok(table_value)) => {
                            if table_key.is_empty() {
                                Err(TableRowError::MissingKeyError)
                            } else {
                                allocation_table.insert(table_key, table_value);
                                Ok(())
                            }
                        }
                        _ => Err(TableRowError::WebElementError),
                    }
                },
                Ok(row_elements) => Err(TableRowError::RowSizeError(row_elements.len())),
                Err(_) => Err(TableRowError::WebElementError),
            };

            let row_error = match insert_row {
                Ok(()) => {
                    table_row_number += 1;
                    continue;
                },
                Err(e) => e,
            };
            
            match row_error {
                TableRowError::WebElementError | TableRowError::RowSizeError(_) => {
                    self.go_back_to_timetable().await?;
                    let event = self.timetabled_event(&timetable_column, timetable_row).await?;
                    event.click().await?;
                    table_rows = self.table_rows().await?;
                }
                TableRowError::MissingKeyError => {}
            }
        }

        if allocation_table.len() != ROWS_IN_TABLE {
            return Err(
                Box::new(
                    TableError::TableSizeError(ROWS_IN_TABLE, allocation_table.len())
                )
            );
        }

        let allocation = Allocation::try_new(&allocation_table)?;
        Ok(allocation)
    }

    async fn go_back_to_timetable(&self) -> WebDriverResult<()> {
        let go_back_button = self.driver
            .query(By::XPath(GO_BACK_BUTTON))
            .first()
            .await?;

        Ok(go_back_button.click().await?)
    }

    #[inline]
    async fn timetabled_event(&self, timetable_column: &str, timetable_row: u64) -> WebDriverResult<WebElement> {
        let by = By::XPath(format_u64(&timetable_column, timetable_row));
        self.driver.query(by).first().await
    }

    #[inline]
    async fn table_rows(&self) -> WebDriverResult<Vec<WebElement>> {
        self.driver
            .query(By::XPath(ALLOCATION_TABLE_ROWS))
            .all_from_selector()
            .await
    }
}