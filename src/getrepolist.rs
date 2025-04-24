use std::error::Error;

use reqwest::blocking::Client;
use reqwest::{Method, header};
use serde_json::json;

use crate::{config::Config, repolist::GraphQLResponce};

const QUERY: &str = r#"query {
  search(type: REPOSITORY, query: """
  is:public 
  language:java
  """, last: 50) {
    repos: edges {
      repo: node {
        ... on Repository {
          url
          nameWithOwner
          defaultBranchRef {
            name
          }
        }
      }
    }
    pageInfo {
      endCursor
      hasNextPage
    }
  }
}"#;

pub fn request(config: &Config) -> Result<GraphQLResponce, Box<dyn Error>> {
    let client = Client::new();
    let url = "https://api.github.com/graphql";
    let header = client
        .request(Method::POST, url)
        .header(header::AUTHORIZATION, format!("bearer {}", config.token))
        .header(header::USER_AGENT, "me");
    Ok(header
        .json(&json!({"query": QUERY}))
        .send()?
        .json::<GraphQLResponce>()?)
}
