use jiff::civil::Date;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PullRequests {
    pub user: String,
    pub start: Date,
    pub end: Date,
    pub authored: Vec<String>,
    pub reviewed: Vec<String>,
}
