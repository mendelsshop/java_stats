use std::fs;

use config::Config;
use getrepolist::request;
use proccessrepo::{Data, Repo};
use rayon::iter::ParallelIterator;

mod config;
mod getrepolist;
mod proccessrepo;
mod repolist;

fn main() {
    let config_file = fs::read_to_string("stats.toml").unwrap();
    let mut config = toml::from_str::<Config>(&config_file).unwrap();
    let resp = request(&config).unwrap();
    config.next_page = Some(resp.data.search.page_info.end_cursor);
    fs::write("stats.toml", toml::to_string(&config).unwrap()).unwrap();
    let repo = resp.data.search.repos;
    let res = proccessrepo::proccess_reops(&config, repo)
        .filter_map(|k| k.ok())
        .collect::<Vec<Repo>>();
    let repo = Data { repos: res };
    println!("{}", serde_json::to_string_pretty(&repo).unwrap())
}
