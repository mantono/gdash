pub mod issue {

    use serde_json::Value;
    use std::fmt;
    use std::cmp::Ordering;
    use chrono::{DateTime, FixedOffset, Utc};
    use crate::state::State;

    #[derive(Debug)]
    pub struct Issue {
        id: String,
        url: String,
        title: String,
        labels: Vec<String>,
        comments: u32,
        state: Option<State>,
        updated_at: DateTime<chrono::FixedOffset>,
        repository: String,
    }

    impl Issue {
        pub fn from_json(data: &Value) -> Vec<Issue> {
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
                .filter(|node| is_open(&node.state))
                .collect::<Vec<Issue>>();

            issues.sort();
            issues
        }

        pub fn from_node(node: &Value) -> Option<Issue> {
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
                id: node["id"].as_str()?.to_string(),
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

        fn score(&self) -> u64 {
            let then: DateTime<FixedOffset> = self.updated_at.clone();
            let now: &DateTime<Utc> = &Utc::now();
            let dur = now.signed_duration_since(then);
            let days: u64 = (dur.num_seconds() as u64) / 86_400;
            let comments: u64 = (self.comments * self.comments).min(1) as u64;
            days * comments
        }
    }

    fn is_open(state: &Option<State>) -> bool {
        match state {
            Some(s) => match s {
                State::Open => true,
                State::Closed => false
            }
            _ => false
        }
    }

    impl fmt::Display for Issue {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.write_fmt(format_args!("{} -> {}\t[{}]", &self.repository, &self.title, &self.url))
        }
    }

    impl Ord for Issue {
        fn cmp(&self, other: &Self) -> Ordering {
            other.score().cmp(&self.score())
        }
    }

    impl PartialOrd for Issue {
        fn partial_cmp(&self, other: &Issue) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Eq for Issue {}

    impl PartialEq for Issue {
        fn eq(&self, other: &Issue) -> bool {
            self.id == other.id
        }
    }
}

pub mod state {

    #[derive(Debug, PartialEq)]
    pub enum State {
        Open,
        Closed
    }

    impl State {
        pub fn from_string(str: &String) -> Option<State> {
            let str: &str = str.as_str();
            match str {
                "OPEN" => Some(State::Open),
                "CLOSED" => Some(State::Closed),
                "null" => None,
                _ => panic!("Invalid argument {}", str)
            }
        }
    }
}

pub mod args {

    use std::env;

    pub struct Arguments {
        pub user: String,
        pub organizations: Vec<String>
    }

    impl Arguments {
        pub fn from_args() -> Result<Arguments, &'static str> {
            let args: Vec<String> = env::args().filter(|x| !x.ends_with("gdash")).collect();
            match args.len() {
                0 => Err("No arguments given, needs [USER] [ORGANIZATION ...]"),
                1 => Err("No argument given for organization"),
                _ => Ok(Arguments {
                    user: args.first().unwrap().clone(),
                    organizations: env::args().skip(2).collect()
                })
            }
        }
    }
}