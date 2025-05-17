use askama::Template;
use git_version::git_version;

use crate::models::StatusRecord;

#[derive(Template)]
#[template(path = "root.html")]
pub struct RootTemplate {
    record: StatusRecord,
    rows: usize,
    version: &'static str,
    git_ver: &'static str,
}

impl RootTemplate {
    pub fn new(record: StatusRecord) -> Self {
        let rows = record.statuses.len();
        let version = env!("CARGO_PKG_VERSION");
        Self {
            record,
            rows,
            version,
            git_ver: git_version!(fallback = "unknown"),
        }
    }
}

#[derive(Template)]
#[template(path = "status/partial.html")]
pub struct StatusPartialTemplate {
    record: StatusRecord,
    rows: usize,
}

impl StatusPartialTemplate {
    pub fn new(record: StatusRecord) -> Self {
        let rows = record.statuses.len();
        Self { record, rows }
    }
}

#[derive(Template)]
#[template(path = "fragments/toast.html")]
pub struct ToastFragment {
    msg: String,
    status: bool,
}

impl ToastFragment {
    pub fn new(msg: String, status: bool) -> Self {
        Self { msg, status }
    }
}
