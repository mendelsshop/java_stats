use serde::Deserialize;

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Config {
    pub last_page: Option<String>,
    pub token: String,
    pub batch: usize,
}
