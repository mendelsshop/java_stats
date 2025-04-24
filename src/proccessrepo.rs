use std::error::Error;
use std::fs::{self};
use std::io::Read;

use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use reqwest::{Method, header};
use tar::Archive;
use tempfile::TempDir;
use tree_sitter::{Parser, Query, QueryCursor, Range, StreamingIterator};

use crate::config::Config;

use crate::repolist::RepoData;

#[derive(Debug)]
pub struct Repo {
    data: RepoData,
    generics_used: Vec<GenericUsage>,
}
#[derive(Debug)]
pub struct GenericUsage {
    name: String,
    range: Range,
    kind: DefintionKind,
}
#[derive(Debug)]
pub enum DefintionKind {
    Class,
    Interface,
    Method,
    Constructor,
}
pub struct QueryInfo {
    query: Query,
    class: u32,
}
pub fn proccess_repo(config: &Config, repo: RepoData) -> Result<Repo, Box<dyn Error>> {
    let path = get_repo(config, &repo)?;
    let mut generics_used = vec![];
    let query = Query::new(
        &tree_sitter_java::LANGUAGE.into(),
        "(class_declaration (identifier) @name (type_parameters)) @generic_class",
    )?;
    let class = query
        .capture_index_for_name("generic_class")
        .ok_or("could not find class query")?;
    let query = QueryInfo { query, class };
    traveserse_and_find(path.path(), &query, &mut generics_used);
    Ok(Repo {
        data: repo,
        generics_used,
    })
}

fn traveserse_and_find(
    path: &std::path::Path,
    query: &QueryInfo,
    generics_used: &mut Vec<GenericUsage>,
) {
    if path.is_file() {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_java::LANGUAGE.into())
            .unwrap();
        let Ok(contents) = fs::read_to_string(path) else {
            return;
        };
        let Some(code) = parser.parse(&contents, None) else {
            return;
        };
        let mut cursor = QueryCursor::new();
        cursor
            .matches(&query.query, code.root_node(), contents.as_bytes())
            .filter(|m| m.nodes_for_capture_index(query.class).any(|_| true))
            .for_each(|m| {
                let class = m.captures[0].node.range();
                let Ok(name) = m.captures[1].node.utf8_text(contents.as_bytes()) else {
                    return;
                };
                generics_used.push(GenericUsage {
                    name: name.to_string(),
                    range: class,
                    kind: DefintionKind::Class,
                });
            });
    } else if path.is_dir() {
        let Ok(read_dir) = path.read_dir() else {
            return;
        };
        for entry in read_dir {
            if let Ok(entry) = entry {
                _ = traveserse_and_find(entry.path().as_path(), query, generics_used);
            }
        }
    }
}

fn extract_data<R: Read>(data: R) -> Result<TempDir, Box<(dyn Error + 'static)>> {
    let path = tempfile::tempdir()?;
    let gz = GzDecoder::new(data);
    let mut archive = Archive::new(gz);
    archive.unpack(path.path())?;
    Ok(path)
}

fn get_repo(config: &Config, repo: &RepoData) -> Result<TempDir, Box<dyn Error>> {
    // TODO: we anyway have to traverse the whole repo maybe doesnt make sense to use tarball
    // approach
    let client = Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/tarball/{}",
        repo.name_with_owner, repo.default_branch_ref.name
    );
    let header = client
        .request(Method::GET, url)
        .header(header::AUTHORIZATION, format!("bearer {}", config.token))
        .header(header::USER_AGENT, "me")
        .header(header::ACCEPT, "application/vnd.github+json");
    let resp = header.send()?;
    extract_data(resp)
}
