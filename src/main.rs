extern crate reqwest;

use gdash::issue::issue::Issue;
use gdash::args::Arguments;
use std::env;
use serde_json::Value;
use std::collections::HashMap;

fn main() -> Result<(), Box<std::error::Error>> {
    let args = Arguments::from_args();
    let token: Option<(String, String)> = env::vars().find(|env| {env.0 == "GITHUB_TOKEN"});
    let token: String = match token {
        Some(s) => s.1,
        None => panic!("No GitHub API token found")
    };
    let mut json_data: HashMap<String, String> = HashMap::new();
    let query = include_str!("search_issues.graphql").to_string();

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

struct GraphQL {
    query: String,
    search_query: String,
    variables: String,
    operation_name: String
}

impl GraphQL {
    fn build_from(query: QueryType, ) -> GraphQL {

    }
}

enum QueryType {
    Issue { user: String, orgs: Vec<String> },
    PullRequest { user: String, orgs: Vec<String> },
    ReviewRequest { user: String, orgs: Vec<String> }
}

impl QueryType {

    fn pull_request() -> String {
        include_str!("search_pull_requests.graphql").to_string()
    }

    fn issues() -> String {
        include_str!("search_issues.graphql").to_string()
    }

    fn user(&self) -> String {
        match *self {
            QueryType::Issue {user, orgs} => user,
            QueryType::PullRequest {user, orgs} => user,
            QueryType::ReviewRequest {user, orgs} => user
        }
    }

    fn orgs(&self) -> Vec<String> {
        match *self {
            QueryType::Issue {user, orgs} => orgs,
            QueryType::PullRequest {user, orgs} => orgs,
            QueryType::ReviewRequest {user, orgs} => orgs
        }
    }


    pub fn query(&self) -> String {
        match *self {
            Issue => QueryType::issues(),
            PullRequest => QueryType::pull_request(),
            ReviewRequest => QueryType::pull_request()
        }
    }

    pub fn operation_name(&self) -> String {
        match *self {
            Issue => String::from("UserIssues"),
            PullRequest => String::from("UserPullRequest"),
            ReviewRequest => String::from("UserReviewRequest")
        }
    }

    fn search_query(&self) -> String {
        let base_query: String = String::from("is:open archived:false ");
        let additional: String = match *self {
            Issue => String::from("is:open archived:false assignee:"),
            PullRequest => String::from("UserPullRequest"),
            ReviewRequest => String::from("UserReviewRequest")
        };
        let users: String = self.orgs().join(" user:");
        let query: String = vec![base_query, users].join("-");
        println!("{}", query);
        query
    }
}