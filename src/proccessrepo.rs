use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::blocking::Client;
use reqwest::{Method, header};
use serde::Serialize;
use tar::Archive;
use tempfile::TempDir;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

use crate::config::Config;

use crate::repolist::{self, RepoData};

#[derive(Debug, Serialize)]
pub struct Data {
    pub repos: Vec<Repo>,
}
#[derive(Debug, Serialize)]
pub struct Repo {
    data: RepoData,
    files: Vec<File>,
}
#[derive(Debug, Serialize)]
pub struct File {
    generics_used: Vec<GenericUsage>,
    path: Box<Path>,
    github_link: String,
}
#[derive(Debug, Serialize)]
pub struct Point {
    pub row: usize,
    pub column: usize,
}
impl From<tree_sitter::Point> for Point {
    fn from(value: tree_sitter::Point) -> Self {
        Self {
            row: value.row,
            column: value.column,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Range {
    pub start_point: Point,
    pub end_point: Point,
}

impl From<tree_sitter::Range> for Range {
    fn from(value: tree_sitter::Range) -> Self {
        Self {
            start_point: value.start_point.into(),
            end_point: value.end_point.into(),
        }
    }
}
#[derive(Debug, Serialize)]
pub struct GenericUsage {
    name: String,
    range: Range,
    kind: DefintionKind,
    generics: String,
}
#[derive(Debug, Serialize)]
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

pub fn proccess_reops(
    config: &Config,
    repos: Vec<repolist::Repo>,
) -> impl ParallelIterator<Item = Result<Repo, Box<dyn Error + Send + Sync>>> {
    repos
        .into_par_iter()
        .map(|repo| proccess_repo(config, repo.repo))
}

pub fn proccess_repo(
    config: &Config,
    repo: RepoData,
) -> Result<Repo, Box<dyn Error + Send + Sync>> {
    let path = get_repo(config, &repo)?;
    let mut files = vec![];
    let query = Query::new(
        &tree_sitter_java::LANGUAGE.into(),
        "(class_declaration (identifier) @name (type_parameters) @type_params) @generic_class",
    )?;
    let class = query
        .capture_index_for_name("generic_class")
        .ok_or("could not find class query")?;
    let query = QueryInfo { query, class };
    traveserse_and_find(path.path(), path.path(), &query, &mut files, &repo);
    if files.is_empty() {
        Err(format!("no results found for {}", repo.name_with_owner))?;
    }
    Ok(Repo { data: repo, files })
}

fn traveserse_and_find(
    path: &Path,
    root_path: &Path,
    query: &QueryInfo,
    generics_used: &mut Vec<File>,
    repo: &RepoData,
) {
    if path.is_file() && path.extension().is_some_and(|ext| ext == "java") {
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
        let mut usages = vec![];
        cursor
            .matches(&query.query, code.root_node(), contents.as_bytes())
            .filter(|m| m.nodes_for_capture_index(query.class).any(|_| true))
            .for_each(|m| {
                let class = m.captures[0].node.range();
                let Ok(generics) = m.captures[2].node.utf8_text(contents.as_bytes()) else {
                    return;
                };
                let Some(name) = m.captures[1]
                    .node
                    .utf8_text(contents.as_bytes())
                    .ok()
                    .filter(|x| {
                        !x.contains("Listen")
                            && !x.contains("List")
                            && !x.contains("Stack")
                            && !x.contains("Consumer")
                            && !x.contains("Callback")
                            && !x.contains("CallBack")
                            && !x.contains("Entitiy")
                            && !x.contains("Map")
                            && !x.contains("Multimap")
                            && !x.contains("Future")
                            && !x.contains("Cache")
                            && !x.contains("Task")
                            && !x.contains("Array")
                            && !x.contains("Hash")
                            && !x.starts_with("Abstract")
                            && !x.starts_with("Base")
                            && !x.ends_with("Delegate")
                            && !x.contains("Function")
                            && !x.contains("Predicate")
                            && !x.contains("Supplier")
                            && !x.contains("Runnable")
                            && !x.contains("Action")
                            && !x.contains("Adapter")
                            && !x.contains("Result")
                            && !x.contains("Option")
                            && !x.contains("Maybe")
                            && !x.contains("LRU")
                            && !x.contains("Trie")
                            && !x.contains("Either")
                            && !x.ends_with("Impl")
                            && !x.contains("Test")
                            && !x.contains("Pair")
                            && !x.contains("Builder")
                            && !x.contains("Serialization")
                            && !x.contains("Serializable")
                            && !x.contains("Handler")
                            && !x.contains("Tuple")
                            && !x.contains("Tree")
                            && !x.contains("Entry")
                            && !x.contains("Set")
                            && !x.contains("Queue")
                            && !x.contains("Dequeue")
                            && !x.contains("Deque")
                            && !x.contains("Factory")
                            && !x.contains("Vector")
                            && !x.contains("Comparator")
                            && !x.contains("Observable")
                            && !x.contains("Observer")
                            && !x.ends_with("Handler")
                            && !x.contains("Iterator")
                            && !x.contains("Stream")
                            && !x.contains("Iterable")
                            && !x.contains("Heap")
                            && !x.contains("Pool")
                    })
                else {
                    return;
                };

                usages.push(GenericUsage {
                    name: name.to_string(),
                    generics: generics.to_string(),
                    range: class.into(),
                    kind: DefintionKind::Class,
                });
            });
        if !usages.is_empty() {
            let Ok(path) = path.strip_prefix(root_path) else {
                return;
            };
            generics_used.push(File {
                generics_used: usages,
                path: path.to_path_buf().into_boxed_path(),
                github_link: format!(
                    "{}/tree/{}/{}",
                    repo.url,
                    repo.default_branch_ref.name,
                    path.components().skip(1).collect::<PathBuf>().display()
                ),
            });
        }
    } else if path.is_dir() {
        let Ok(read_dir) = path.read_dir() else {
            return;
        };
        for entry in read_dir.flatten() {
            traveserse_and_find(
                entry.path().as_path(),
                root_path,
                query,
                generics_used,
                repo,
            );
        }
    }
}

fn extract_data<R: Read>(data: R) -> Result<TempDir, Box<(dyn Error + Send + Sync + 'static)>> {
    let path = tempfile::tempdir()?;
    let gz = GzDecoder::new(data);
    let mut archive = Archive::new(gz);
    archive.unpack(path.path())?;
    Ok(path)
}

fn get_repo(config: &Config, repo: &RepoData) -> Result<TempDir, Box<dyn Error + Send + Sync>> {
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
