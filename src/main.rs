extern crate reqwest;

use std::env;
use reqwest::Response;
use serde_json::Value;
use serde_json::{Deserializer, Serializer};
use std::ops::Add;
use std::fs;
use std::collections::HashMap;
use std::fmt::Error;

fn main() -> Result<(), Box<std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        panic!("No arguments given")
    }
    let token: Option<(String, String)> = env::vars().find(|env| {env.0 == "GITHUB_TOKEN"});
    let token: String = match token {
        Some(s) => s.1,
        None => panic!("No GitHub API token found")
    };
    println!("{:?}", token);
    println!("{:?}", args);
    let mut json_data: HashMap<String, String> = HashMap::new();
    let query = fs::read_to_string("src/search.graphql")
        .expect("Something went wrong reading the file");
    let variables: &str = r#"
        { "searchQuery": "is:open assignee:mantono archived:false user:zensum user:klira user:open-broker sort:comments-desc" }
    "#;
    json_data.insert(String::from("query"), query);
    json_data.insert(String::from("variables"), String::from(variables));
    json_data.insert(String::from("operationName"), String::from("UserIssues"));

    let client = reqwest::Client::new();
    let response: String = client
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .json(&json_data)
        .send()?
        .text()?;

    let pretty_json = serde_json::to_string_pretty(&response)?;
    println!("{:#?}", pretty_json);
    println!("{:#?}", json_data);
    let json: Value = serde_json::from_str(&response)?;
    let issue = Issue::from_json(&json);
    println!("{:#?}", issue);
    Ok(())
}


#[derive(Debug)]
struct Issue {
    url: String,
    title: String,
    labels: Vec<String>,
    comments: u32,
    updated_at: String,
    repository: String,
}

impl Issue {
    fn from_json(data: &Value) -> Vec<Issue> {
        let nodes: Option<&Vec<Value>> = data["data"]["search"]["edges"].as_array();
        let nodes: &Vec<Value> = match nodes {
            Some(vec) => vec,
            None => &Vec::new()
        };
        nodes.iter()
            .filter(|&node| node.is_object())
            .map(|&node|Issue::from_node(&node))
            .collect()
    }

    fn from_node(node: &Value) -> Issue {
        let comments: Option<u64> = node["comments"]["totalCount"].as_u64();
        let comments: u32 = match comments {
            Some(c) => c as u32,
            None => 0u32
        };
        Issue {
            url: node["url"].to_string(),
            title: node["title"].to_string(),
            labels: Vec::new(),
            comments,
            updated_at: node["updatedAt"].to_string(),
            repository: node["repository"]["nameWithOwner"].to_string()
        }
    }
}