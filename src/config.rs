use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
pub struct Config {
    pub next_page: Option<String>,
    pub token: String,
    pub batch: usize,
}
