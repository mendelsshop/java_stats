use serde::Deserialize;

#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLResponce {
    pub data: Data,
}
#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Search {
    pub repos: Vec<Repo>,
    page_info: PageInfo,
}
#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
    pub repo: RepoData,
}
#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepoData {
    pub url: String,
    pub name_with_owner: String,
    pub default_branch_ref: Branch,
}
#[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Branch {
    pub name: String,
}
#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub search: Search,
}
#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    end_cursor: String,
    has_next_page: bool,
}

#[test]
fn test() {
    let json = serde_json::from_str::<GraphQLResponce>(
        r#"
{
  "data": {
    "search": {
      "repos": [
               {
          "repo": {
            "url": "https://github.com/Snailclimb/JavaGuide",
            "nameWithOwner": "Snailclimb/JavaGuide",
            "defaultBranchRef": {
              "name": "main"
            }
          }
        },
        {
          "repo": {
            "url": "https://github.com/krahets/hello-algo",
            "nameWithOwner": "krahets/hello-algo",
            "defaultBranchRef": {
              "name": "main"
            }
          }
        }
      ],
      "pageInfo": {
        "endCursor": "Y3Vyc29yOjUw",
        "hasNextPage": true
      }
    }
  }
}
"#,
    )
    .unwrap();
    assert_eq!(
        json,
        GraphQLResponce {
            data: Data {
                search: Search {
                    repos: vec![
                        Repo {
                            repo: RepoData {
                                url: "https://github.com/Snailclimb/JavaGuide".to_string(),
                                name_with_owner: "Snailclimb/JavaGuide".to_string(),
                                default_branch_ref: Branch {
                                    name: "main".to_string()
                                }
                            }
                        },
                        Repo {
                            repo: RepoData {
                                url: "https://github.com/krahets/hello-algo".to_string(),
                                name_with_owner: "krahets/hello-algo".to_string(),
                                default_branch_ref: Branch {
                                    name: "main".to_string()
                                }
                            }
                        }
                    ],
                    page_info: PageInfo {
                        end_cursor: "Y3Vyc29yOjUw".to_string(),
                        has_next_page: true
                    }
                }
            }
        }
    )
}
