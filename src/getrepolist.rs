#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(
    clippy::use_self,
    rust_2018_idioms,
    missing_debug_implementations,
    clippy::missing_panics_doc
)]

use std::error::Error;

use reqwest::blocking::Client;
use reqwest::{Method, header};
use serde_json::json;

use crate::{config::Config, repolist::GraphQLResponce};
pub fn request(config: &Config) -> Result<GraphQLResponce, Box<dyn Error>> {
    let after = config
        .next_page
        .as_ref()
        .map_or(String::new(), |s| format!(r#", after: "{s}""#));
    let query = format!(
        r#"query {{
  search(type: REPOSITORY, query: """
  is:public 
  language:java
  """, last: {}{after}
  ) {{
    repos: edges {{
      repo: node {{
        ... on Repository {{
          url
          nameWithOwner
          defaultBranchRef {{
            name
          }}
        }}
      }}
    }}
    pageInfo {{
      endCursor
      hasNextPage
    }}
  }}
}}"#,
        config.batch
    );
    let client = Client::new();
    let url = "https://api.github.com/graphql";
    let header = client
        .request(Method::POST, url)
        .header(header::AUTHORIZATION, format!("bearer {}", config.token))
        .header(header::USER_AGENT, "me");
    Ok(header
        .json(&json!({"query": query}))
        .send()?
        .json::<GraphQLResponce>()?)
}
