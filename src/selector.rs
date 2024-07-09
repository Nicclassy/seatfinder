#[derive(Clone, Copy)]
pub struct XPathStr(&'static str);

impl Into<String> for XPathStr {
    fn into(self) -> String {
        self.0.to_owned()
    }
}

pub static SEARCH_BAR: XPathStr = XPathStr(r#"//*[@id="search_box"]"#);
pub static SEARCH_BUTTON: XPathStr = XPathStr(r#"//*[@id="search-form"]/input"#);
pub static SHOW_TIMETABLE: XPathStr = XPathStr(r#"//*[@id="toggle-right-col-btn"]"#);

pub static UNIT_OFFERINGS: XPathStr = XPathStr(r#"//*[@id="selected-results"]/li/strong"#);
pub static OFFERING_CHECKBOX: XPathStr = XPathStr(r#"//*[@id="selected-results"]/li/input"#);