use std::fs::File;
use std::error::Error;
use std::io::BufReader;
use std::time::Instant;
use std::process::Child;

use env_logger;
use log::info;
use colored::{self, Colorize};
use chrono;
use serde_json::{self, Value};
use thirtyfour::prelude::*;
use tokio::{time::{self, Duration}, runtime::Runtime};

use crate::consts::{CONFIG_FILE, TIMED};
use crate::error::OfferingError;
use crate::query::{FinderQuery, FinderConfig};
use crate::methods::{
    format_str, 
    format_usize,
    chromedriver_process, 
    parse_queries,
    annoy,
    single_offering,
    multiple_offerings, 
};
use crate::selector::*;
use crate::allocation::AllocationResult;
use crate::searcher::TimetableSearcher;

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
            info!("No queries to find.");
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
        
            match self.search_query(&interactees.show_timetable_button, query).await {
                Ok(opt) => match opt {
                    Some(allocation) => allocation.notify_query_resolved(query.unit_code()),
                    None => info!("No allocations found for {} matching the given query.", query.unit_code()),
                }
                Err(e) => panic!("Error searching for the query: {}", e)
            }

            if let Err(e) = self.reset_timetable().await {
                panic!("Error clearing the timetable: {}", e);
            }
        }
    }

    pub async fn seats_are_available(&self) -> Option<bool> {
        let mut availability = false;
        for query in self.queries.iter() {
            let interactees = match self.locate_interactees().await {
                Ok(interactees) => interactees,
                Err(_) => return None,
            };

            self.toggle_advanced_filter(query).await.ok()?;
            self.search_timetable(&interactees, query).await.ok()?;
            self.select_unit(query).await.ok()?;
            
            if let Ok(opt) = self.search_query(&interactees.show_timetable_button, query).await {
                match opt {
                    Some(allocation) => {
                        allocation.notify_query_resolved(query.unit_code());
                        availability = true;
                    },
                    None => info!("No allocations found for {} matching the given query.", query.unit_code()),
                }
            }

            self.reset_timetable().await.ok()?;
        }

        Some(availability)
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
        if let Some(ref start_time) = query.start_after {
            // For some reason, the start time must be entered as one hour ahead of the actual start time
            // so as to exclude prior allocations
            let script = format!("document.getElementById('{START_TIME}').value = '{}';", start_time.progress_one_hour());
            self.driver.execute(script, Vec::new()).await?;
        }

        let checkbox_id = format_str(
            ACTIVITY_CHECKBOX_FORMAT.as_str(), 
            query.activity_type.checkbox_id_suffix()
        );
        let by = By::Id(checkbox_id);
        
        let activity_checkbox = self.driver
            .query(by)
            .first()
            .await?;
        
        activity_checkbox.click().await?;
        Ok(())
    }

    async fn search_timetable(
        &self, 
        Interactees { search_bar, search_button, show_timetable_button: _ }: &Interactees, 
        query: &FinderQuery
    ) -> WebDriverResult<()> {
        search_bar.clear().await?;
        search_bar.send_keys(&query.unit_code).await?;
        search_button.wait_until().clickable().await?;
        search_button.click().await
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
                    let by = By::XPath(format_usize(OFFERING_CHECKBOX_FORMAT.as_str(), 1));
                    let checkbox = parent.query(by).first().await?;
                    Ok(checkbox.click().await?)
                },
                Err(e) => Err(e),
            }
        }
        
        match multiple_offerings(query, &subcodes) {
            Some(index) => {
                let parent = selected_results[0].parent().await?;
                let by = By::XPath(format_usize(OFFERING_CHECKBOX_FORMAT.as_str(), index + 1));
                let checkbox = parent.query(by).first().await?;
                Ok(checkbox.click().await?)
            }
            None => Err(
                Box::new(
                    OfferingError::NoValidOfferingsError(query.unit_code())
                )
            )
        }
    }

    async fn search_query(&self, show_timetable_button: &WebElement, query: &FinderQuery) -> AllocationResult {
        show_timetable_button.click().await?;

        let searcher = TimetableSearcher::new(&self.driver, query);
        searcher.search().await
    }

    async fn reset_timetable(&self) -> WebDriverResult<()> {
        self.clear_timetable().await?;
        self.reselect_all().await?;
        Ok(())
    }

    async fn clear_timetable(&self) -> WebDriverResult<()> {
        let clear_button = self.query_by_xpath(CLEAR_BUTTON).await?;
        clear_button.click().await
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
        activity_checkbox.click().await
    }

    #[inline]
    async fn query_by_xpath(&self, xpath: impl Into<String>) -> WebDriverResult<WebElement> {
        self.driver.query(By::XPath(xpath)).first().await
    }
}

pub fn run() {
    env_logger::init();

    let rt = Runtime::new().unwrap();
    let start = if TIMED { Some(Instant::now()) } else { None };
    
    rt.block_on(async {
        let seatfinder = SeatFinder::new().await;
        seatfinder.seatfind().await;
        seatfinder.quit().await;
    });
    if let Some(instant) = start {
        info!("Program took {:.2?} seconds to execute", instant.elapsed());
    }
}

pub fn run_every(seconds: u64) {
    env_logger::init();

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let duration = Duration::from_secs(seconds);
        let seatfinder = SeatFinder::new().await;

        let mut timer = time::interval(duration);

        loop {
            timer.tick().await;

            let now = chrono::Local::now();
            let formatted = format!("{}: Seatfinding", now.format("[%d/%m/%y %H:%M:%S]"));
            info!("{}", formatted.red());

            match seatfinder.seats_are_available().await {
                Some(false) => continue,
                None => {
                    let now = chrono::Local::now();
                    let formatted = format!("{}: Refreshing page...", now.format("[%d/%m/%y %H:%M:%S]"));
                    info!("{}", formatted.cyan());

                    seatfinder.driver.refresh().await.unwrap();
                    continue;
                }
                _ => {},
            }

            match seatfinder.config.music {
                Some(ref path) => {
                    let file_name = path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap();
                    info!("Playing '{}'", file_name);
                    annoy(path);
                },
                None => info!("No music to play :("),
            }
        }
    });
}