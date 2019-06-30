extern crate reqwest;

use gdash::issue::Issue;
use gdash::args::Arguments;
use std::env;
use serde_json::Value;
use std::fs;
use std::collections::HashMap;

fn main() -> Result<(), Box<std::error::Error>> {
    let args = Arguments::from_args();
    let token: Option<(String, String)> = env::vars().find(|env| {env.0 == "GITHUB_TOKEN"});
    let token: String = match token {
        Some(s) => s.1,
        None => panic!("No GitHub API token found")
    };
    let mut json_data: HashMap<String, String> = HashMap::new();
    let query = fs::read_to_string("src/search.graphql")
        .expect("Something went wrong reading the file");

    let mut variables: String = r#"{ "searchQuery": "is:open archived:false assignee:"#.to_string();
    let args = match args {
        Ok(arguments) => arguments,
        Err(error) => panic!(error)
    };
    variables.push_str(&args.user);
    for org in args.organizations.iter() {
        variables.push_str(&format!(" user:{}", &org));
    }
    variables.push_str(r#" sort:comments-desc" }"#);
    println!("{}", variables);
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

    let json: Value = serde_json::from_str(&response)?;
    let issues: Vec<Issue> = Issue::from_json(&json);
    for i in issues {
        println!("{}", i)
    }
    Ok(())
}