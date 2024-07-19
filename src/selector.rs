pub static SEARCH_BAR: XPathSelector = XPathSelector(r#"//*[@id="search_box"]"#);
pub static SEARCH_BUTTON: XPathSelector = XPathSelector(r#"//*[@id="search-form"]/input"#);
pub static SHOW_TIMETABLE: XPathSelector = XPathSelector(r#"//*[@id="toggle-right-col-btn"]"#);
pub static CLEAR_BUTTON: XPathSelector = XPathSelector(r#"//*[@id="clear-selected-btn"]"#);

pub static UNIT_OFFERINGS: XPathSelector = XPathSelector(r#"//*[@id="selected-results"]/li/strong"#);
pub static OFFERING_CHECKBOX: XPathSelector = XPathSelector(r#"//*[@id="selected-results"]/li/input"#);

pub static ALLOCATION_FORMAT: XPathSelector = XPathSelector(r#"//*[@id="timetable-grid"]/div[4]/div[{}]/div[{}]"#);
pub static ALLOCATION_TABLE_ROWS: XPathSelector = XPathSelector(r#"//*[@id="activity-details-tpl"]/div[2]/div[4]/table/tbody/*"#);
pub static GO_BACK_BUTTON: XPathSelector = XPathSelector(r#"//*[@id="activity-details-tpl"]/div[2]/div[6]/button[1]"#);

pub static ACTIVITY_CHECKBOX_FORMAT: IdSelector = IdSelector("ats-{}");

selector!(XPathSelector);
selector!(IdSelector);
