extern crate reqwest;

use gdash::issue::issue::Issue;
use gdash::args::Arguments;
use std::env;
use serde_json::Value;
use std::collections::HashMap;

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
    let issues: Vec<Issue> = api_request(&client, issues, token).expect("Could not request issues");

    for i in issues {
        println!("{}", i)
    };
    Ok(())

/*     let issues: GraphQL = GraphQL::build_from(issues);
    let pull_requests: GraphQL = GraphQL::build_from(pull_requests);

    println!("{:?}", issues);

    let mut json_data: HashMap<String, String> = HashMap::new();
    let query = include_str!("search_issues.graphql").to_string();

    let mut variables: String = r#"{ "searchQuery": "is:open archived:false assignee:"#.to_string();
    variables.push_str(&args.user.clone());
    for org in args.orgs.iter() {
        variables.push_str(&format!(" user:{}", &org));
    }
    variables.push_str(r#" sort:comments-desc" }"#);
    json_data.insert(String::from("query"), query);
    json_data.insert(String::from("variables"), String::from(variables));
    json_data.insert(String::from("operationName"), String::from("UserIssues"));

    let client: reqwest::Client = reqwest::Client::new();
    let response: String = client
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .json(&json_data)
        .send()?
        .text()?;

    let json: Value = serde_json::from_str(&response)?;
    let issues: Vec<Issue> = Issue::from_json(&json); */

    
    // for i in issues {
    //     println!("{}", i)
    // }
    // for p in pull_requests {
    //     println!("{}", p)
    // }
    // Ok(())
}

fn api_request(client: &reqwest::Client, query: QueryType, token: String) -> Result<Vec<Issue>, &'static str> {
    let gql = GraphQL::build_from(query);
    let json = gql.as_json();
    let response: Result<reqwest::Response, reqwest::Error> = client
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .json(&json)
        .send();

    println!("{:?}", json);
    let body: String = match response {
        Ok(mut data) => data.text().expect("Unable to read body"),
        Err(err) => panic!(err)
    };

    println!("{}", body);
    let json: Value = serde_json::from_str(body.as_str()).expect("Cannot parse response as JSON");
    Ok(Issue::from_json(&json))
}

#[derive(Debug)]
struct GraphQL {
    query: String,
    search_query: String,
    operation_name: String
}

impl GraphQL {
    fn build_from(query: QueryType) -> GraphQL {
        GraphQL {
            query: query.query(),
            search_query: query.search_query(),
            operation_name: query.operation_name()
        }
    }

    fn as_json(&self) -> HashMap<String, String> {
        let mut json: HashMap<String, String> = HashMap::new();
        json.insert(String::from("query"), self.query.clone());
        json.insert(String::from("searchQuery"), self.search_query.clone());
        json.insert(String::from("operationName"), self.operation_name.clone());
        json
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

    pub fn operation_name(&self) -> String {
        match self {
            QueryType::Issue { user: _, orgs: _ } => String::from("UserIssues"),
            QueryType::PullRequest { user: _, orgs: _ } => String::from("UserPullRequest"),
            QueryType::ReviewRequest { user: _, orgs: _ } => String::from("UserReviewRequest")
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
        println!("{}", query);
        query
    }
}