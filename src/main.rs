extern crate reqwest;

use gdash::issue::issue::Issue;
use gdash::args::Arguments;
use std::env;
use serde_json::Value;
use serde_json::json;

fn main() -> Result<(), Box<std::error::Error>> {
    let args = Arguments::from_args();
    let args = match args {
        Ok(arguments) => arguments,
        Err(error) => panic!(error)
    };

    let token: Option<(String, String)> = env::vars().find(|env| {env.0 == "GITHUB_TOKEN"});
    let token: String = match token {
        Some(s) => s.1,
        None => panic!("No GitHub API token found")
    };

    let issues = QueryType::Issue {
        orgs: args.orgs.clone(),
        user: args.user.clone()
    };

    let pull_requests = QueryType::PullRequest {
        orgs: args.orgs.clone(),
        user: args.user.clone()
    };

    let review_requests = QueryType::ReviewRequest {
        orgs: args.orgs.clone(),
        user: args.user.clone()
    };

    let client: reqwest::Client = reqwest::Client::new();
    let mut issues: Vec<Issue> = api_request(&client, issues, &token).expect("Could not request issues");
    let mut pull_requests: Vec<Issue> = api_request(&client, pull_requests, &token).expect("Could not request pull requests");
    let mut review_requests: Vec<Issue> = api_request(&client, review_requests, &token).expect("Could not request review requests");

    let mut all: Vec<Issue> = Vec::with_capacity(64);
    all.append(&mut issues);
    all.append(&mut pull_requests);
    all.append(&mut review_requests);
    all.sort();

    for i in all {
        println!("{}", i)
    };
    Ok(())
}

fn api_request(client: &reqwest::Client, query: QueryType, token: &String) -> Result<Vec<Issue>, &'static str> {
    let gql = GraphQL::build_from(query);
    let json = gql.as_json();
    let response: Result<reqwest::Response, reqwest::Error> = client
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .json(&json)
        .send();

    let body: String = match response {
        Ok(mut data) => data.text().expect("Unable to read body"),
        Err(err) => panic!(err)
    };

    let json: Value = serde_json::from_str(body.as_str()).expect("Cannot parse response as JSON");
    Ok(Issue::from_json(&json))
}

#[derive(Debug)]
struct GraphQL {
    query: String,
    search_query: String
}

impl GraphQL {
    fn build_from(query: QueryType) -> GraphQL {
        GraphQL {
            query: query.query(),
            search_query: query.search_query()
        }
    }

    fn as_json(&self) -> Value {
        json!({
            "query": self.query.clone(),
            "variables": {
                "searchQuery": self.search_query.clone()
            }
        })
    }
}

#[derive(Debug)]
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
        match self {
            QueryType::Issue {user, orgs: _} => user,
            QueryType::PullRequest {user, orgs: _} => user,
            QueryType::ReviewRequest {user, orgs: _} => user
        }.clone()
    }

    fn orgs(&self) -> Vec<String> {
        match self {
            QueryType::Issue {user: _, orgs} => orgs,
            QueryType::PullRequest {user: _, orgs} => orgs,
            QueryType::ReviewRequest {user: _, orgs} => orgs
        }.clone()
    }


    pub fn query(&self) -> String {
        match self {
            QueryType::Issue { user: _, orgs: _ } => QueryType::issues(),
            QueryType::PullRequest { user: _, orgs: _ } => QueryType::pull_request(),
            QueryType::ReviewRequest { user: _, orgs: _ } => QueryType::pull_request()
        }
    }

    fn search_query(&self) -> String {
        let base_query: String = String::from("is:open archived:false");
        let additional: String = match self {
            QueryType::Issue { user: _, orgs: _ } => String::from("is:issue assignee:"),
            QueryType::PullRequest { user: _, orgs: _ } => String::from("is:pr assignee:"),
            QueryType::ReviewRequest { user: _, orgs: _ } => String::from("is:pr review-requested:")
        };
        let additional: String = format_args!("{}{}", additional, self.user().clone()).to_string();
        let users: String = self.orgs().iter().map(|o| format!("user:{}", o)).collect::<Vec<String>>().join(" ");
        let query: String = vec![base_query, additional, users].join(" ");
        query
    }
}