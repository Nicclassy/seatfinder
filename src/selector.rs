#[derive(Clone, Copy)]
pub struct XPathStr(&'static str);

impl Into<String> for XPathStr {
    fn into(self) -> String {
        self.0.to_owned()
    }
}

impl XPathStr {
    pub fn format_single_u64(&self, value: u64) -> String {
        // Workaround for lack of variadic .format method in C#/C/Python/Java etc.
        self.0.replace("{}", value.to_string().as_str())
    }
}

pub static SEARCH_BAR: XPathStr = XPathStr(r#"//*[@id="search_box"]"#);
pub static SEARCH_BUTTON: XPathStr = XPathStr(r#"//*[@id="search-form"]/input"#);
pub static SHOW_TIMETABLE: XPathStr = XPathStr(r#"//*[@id="toggle-right-col-btn"]"#);

pub static UNIT_OFFERINGS: XPathStr = XPathStr(r#"//*[@id="selected-results"]/li/strong"#);
pub static OFFERING_CHECKBOX: XPathStr = XPathStr(r#"//*[@id="selected-results"]/li/input"#);

pub static TUTORIAL_ALLOCATION_FORMAT: XPathStr = XPathStr(r#"//*[@id="timetable-grid"]/div[4]/div[{}]/*"#);
pub static ALLOCATIONS_TABLE: XPathStr = XPathStr(r#"//*[@id="activity-details-tpl"]/div[2]/div[4]/table/tbody/*"#);
pub static GO_BACK_BUTTON: XPathStr = XPathStr(r#"//*[@id="activity-details-tpl"]/div[2]/div[6]/button[1]"#);