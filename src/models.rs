use std::fmt::Display;
use std::str::FromStr;

use jiff::Zoned;
use rusqlite::types::{FromSql, FromSqlError, ToSqlOutput};
use rusqlite::ToSql;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct WakeForm {
    pub mac_address: String,
}

#[derive(Deserialize)]
pub struct SleepForm {
    pub address: String,
}

#[derive(Serialize)]
pub struct ApiResponse {
    message: String,
}

impl ApiResponse {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub enum CommandState {
    Wake,
    Sleep,
}

impl Display for CommandState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandState::Wake => write!(f, "Wake"),
            CommandState::Sleep => write!(f, "Sleep"),
        }
    }
}

impl ToSql for CommandState {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        Ok(ToSqlOutput::from(match self {
            CommandState::Wake => "Wake",
            CommandState::Sleep => "Sleep",
        }))
    }
}

impl FromSql for CommandState {
    fn column_result(value: rusqlite::types::ValueRef) -> rusqlite::types::FromSqlResult<Self> {
        value.as_str().and_then(|s| match s {
            "Wake" => Ok(CommandState::Wake),
            "Sleep" => Ok(CommandState::Sleep),
            _ => Err(FromSqlError::InvalidType),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(Zoned);

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ts = self.0.strftime("%Y-%m-%d %H:%M:%S %Z");
        write!(f, "{}", ts)
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct(
            "timestamp",
            &self.0.strftime("%Y-%m-%d %H:%M:%S %Z").to_string(),
        )
    }
}

impl ToSql for Timestamp {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        let iso_string = self.0.to_string();
        Ok(iso_string.into())
    }
}

impl FromSql for Timestamp {
    fn column_result(value: rusqlite::types::ValueRef) -> rusqlite::types::FromSqlResult<Self> {
        value.as_str().and_then(|s| match Zoned::from_str(s) {
            Ok(zoned) => Ok(Self(zoned)),
            Err(err) => Err(FromSqlError::Other(Box::new(err))),
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Status {
    pub timestamp: Timestamp,
    pub cmd: CommandState,
    pub msg: String,
    pub status: bool,
}

impl Status {
    pub fn new(cmd: CommandState, msg: String, status: bool) -> Self {
        Self {
            timestamp: Timestamp(Zoned::now()),
            cmd,
            msg,
            status,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct StatusRecord {
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
    pub total_items: usize,
    pub statuses: Vec<Status>,
}

impl StatusRecord {
    pub fn new(
        page: usize,
        per_page: usize,
        total_pages: usize,
        total_items: usize,
        statuses: Vec<Status>,
    ) -> Self {
        Self {
            page,
            per_page,
            total_pages,
            total_items,
            statuses,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct StatusPagination {
    pub sort: String,
    pub page: usize,
    pub per_page: usize,
}

impl Default for StatusPagination {
    fn default() -> Self {
        Self {
            sort: "desc".to_string(),
            page: 1,
            per_page: 10,
        }
    }
}

pub fn get_record(statuses: Vec<Status>, pagination: StatusPagination) -> StatusRecord {
    let total_items = statuses.len();
    let pages = (total_items as f64 / pagination.per_page as f64).ceil() as usize;
    let start = (pagination.page - 1) * pagination.per_page;
    let end = (pagination.page * pagination.per_page).min(total_items);
    let statuses = if !statuses.is_empty() {
        let mut s = statuses;
        if pagination.sort == "desc" {
            s.reverse();
        }
        if start >= end {
            Vec::new()
        } else {
            s[start..end].to_vec()
        }
    } else {
        statuses
    };

    StatusRecord::new(
        pagination.page,
        pagination.per_page,
        pages,
        total_items,
        statuses,
    )
}
