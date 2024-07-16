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

macro_rules! selector {
    ($selector_name:ident) => {
        #[derive(Clone, Copy)]
        pub struct $selector_name(&'static str);

        impl Into<String> for $selector_name {
            fn into(self) -> String {
                self.0.to_owned()
            }
        }

        impl $selector_name {
            pub fn as_str(&self) -> &str {
                self.0
            }
        }
    };
}

selector!(XPathSelector);
selector!(IdSelector);

pub fn format_u64(src: &str, value: u64) -> String {
    // Workaround for lack of runtime variadic .format method in C#/C/Python/Java etc.
    // Not the cleanest solution but obeys the orphan rule
    src.replacen("{}", &value.to_string(), 1)
}

pub fn format_str(src: &str, value: &str) -> String {
    src.replacen("{}", value, 1)
}
