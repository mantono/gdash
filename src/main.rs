extern crate reqwest;

use std::env;
use serde_json::Value;
use std::fs;
use std::collections::HashMap;
use std::cmp::Ordering;
use chrono::DateTime;

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
    state: Option<State>,
    updated_at: DateTime<chrono::FixedOffset>,
    repository: String,
}

impl Issue {
    fn from_json(data: &Value) -> Vec<Issue> {
        let nodes: Option<&Vec<Value>> = data["data"]["search"]["edges"].as_array();
        let empty = Vec::new();
        let nodes: &Vec<Value> = match nodes {
            Some(vec) => vec,
            None => &empty
        };
        let mut issues: Vec<Issue> = nodes.iter()
            .filter(|node| node.is_object())
            .map(|node| node.get("node"))
            .filter(|node| node.is_some())
            .map(|node| match node {
                Some(n) => n,
                None => panic!("Impossible")
            })
            .map(|node| Issue::from_node(&node))
            .filter(|node| node.is_some())
            .map(|node| node.expect("This is ok"))
            .collect::<Vec<Issue>>();

        issues.sort_by(|issue1, issue2| if issue2.comments < issue1.comments { Ordering::Less } else { Ordering::Greater});
        issues
    }

    fn from_node(node: &Value) -> Option<Issue> {
        println!("{:#?}", node);
        let comments: Option<u64> = node["comments"]["totalCount"].as_u64();
        let comments: u32 = match comments {
            Some(c) => c as u32,
            None => 0u32
        };
        let updated_at: &str = match node["updatedAt"].as_str() {
            Some(s) => s,
            None => return None
        };
        let issue = Issue {
            url: node["url"].as_str()?.to_string(),
            title: node["title"].as_str()?.to_string(),
            labels: Vec::new(),
            state: State::from_string(&node["state"].as_str()?.to_string()),
            comments,
            updated_at: DateTime::parse_from_rfc3339(updated_at).expect("Unable to parse date"),
            repository: node["repository"]["nameWithOwner"].as_str()?.to_string()
        };
        Some(issue)
    }
}

#[derive(Debug)]
enum State {
    Open,
    Closed
}

impl State {
    fn from_string(str: &String) -> Option<State> {
        let str: &str = str.as_str();
        match str {
            "OPEN" => Some(State::Open),
            "CLOSED" => Some(State::Closed),
            "null" => None,
            _ => panic!("Invalid argument {}", str)
        }
    }
}