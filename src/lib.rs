use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PullRequests {
    pub authored: Vec<String>,
    pub reviewed: Vec<String>,
}
