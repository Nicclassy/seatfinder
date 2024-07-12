pub static SEARCH_BAR: XPathStr = XPathStr(r#"//*[@id="search_box"]"#);
pub static SEARCH_BUTTON: XPathStr = XPathStr(r#"//*[@id="search-form"]/input"#);
pub static SHOW_TIMETABLE: XPathStr = XPathStr(r#"//*[@id="toggle-right-col-btn"]"#);

pub static UNIT_OFFERINGS: XPathStr = XPathStr(r#"//*[@id="selected-results"]/li/strong"#);
pub static OFFERING_CHECKBOX: XPathStr = XPathStr(r#"//*[@id="selected-results"]/li/input"#);

pub static ALLOCATION_FORMAT: XPathStr = XPathStr(r#"//*[@id="timetable-grid"]/div[4]/div[{}]/div[{}]"#);
pub static ALLOCATIONS_TABLE: XPathStr = XPathStr(r#"//*[@id="activity-details-tpl"]/div[2]/div[4]/table/tbody/*"#);
pub static GO_BACK_BUTTON: XPathStr = XPathStr(r#"//*[@id="activity-details-tpl"]/div[2]/div[6]/button[1]"#);

#[derive(Clone, Copy)]
pub struct XPathStr(&'static str);

impl Into<String> for XPathStr {
    fn into(self) -> String {
        self.0.to_owned()
    }
}

impl Into<&str> for XPathStr {
    fn into(self) -> &'static str {
        self.0
    }
}

pub fn format_u64(src: &str, value: u64) -> String {
    // Workaround for lack of variadic .format method in C#/C/Python/Java etc.
    // Not the cleanest solution but obeys the orphan rule
    src.replacen("{}", &value.to_string(), 1)
}
