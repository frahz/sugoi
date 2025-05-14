use askama::Template;
use git_version::git_version;

use crate::models::Status;

#[derive(Template)]
#[template(path = "root.html")]
pub struct RootTemplate {
    statuses: Vec<Status>,
    rows: usize,
    current_page: usize,
    total_pages: usize,
    version: &'static str,
    git_ver: &'static str,
}

impl RootTemplate {
    pub fn new(statuses: Vec<Status>, current_page: usize, total_pages: usize) -> Self {
        let rows = statuses.len();
        let version = env!("CARGO_PKG_VERSION");
        Self {
            statuses,
            rows,
            current_page,
            total_pages,
            version,
            git_ver: git_version!(),
        }
    }
}

#[derive(Template)]
#[template(path = "status/partial.html")]
pub struct StatusPartialTemplate {
    statuses: Vec<Status>,
    rows: usize,
    current_page: usize,
    total_pages: usize,
}

impl StatusPartialTemplate {
    pub fn new(statuses: Vec<Status>, current_page: usize, total_pages: usize) -> Self {
        let rows = statuses.len();
        Self {
            statuses,
            rows,
            current_page,
            total_pages,
        }
    }
}
