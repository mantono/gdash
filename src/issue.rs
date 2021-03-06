pub mod issue {

    use serde_json::Value;
    use std::fmt;
    use std::cmp::Ordering;
    use chrono::{DateTime, FixedOffset, Utc};
    use crate::state::{State, Closeable, is_open};

    #[derive(Debug)]
    pub struct Issue {
        id: String,
        url: String,
        title: String,
        labels: Vec<String>,
        comments: u32,
        reactions: u32,
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

            let reactions: Option<u64> = node["reactions"]["totalCount"].as_u64();
            let reactions: u32 = match reactions {
                Some(r) => r as u32,
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
                reactions,
                updated_at: DateTime::parse_from_rfc3339(updated_at).expect("Unable to parse date"),
                repository: node["repository"]["nameWithOwner"].as_str()?.to_string()
            };
            Some(issue)
        }

        fn time_since_updated(&self) -> i64 {
            let then: DateTime<FixedOffset> = self.updated_at.clone();
            let now: &DateTime<Utc> = &Utc::now();
            now.signed_duration_since(then).num_seconds()
        }

        const ONE_DAY: i64 = 86_400;

        fn score(&self) -> i64 {
            let days: i64 = self.time_since_updated() / Issue::ONE_DAY;
            let interactions: i64 = (self.reactions + self.comments).min(1) as i64;
            let interactions: i64 = interactions * interactions;
            (interactions - days) as i64
        }

        fn is_hot(&self) -> bool {
            self.time_since_updated() <= Issue::ONE_DAY
        }
    }

    impl fmt::Display for Issue {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            let hot = if self.is_hot() { "*" }  else { " " };
            fmt.write_fmt(format_args!("{}{} -> {}\t[{}]", hot, &self.repository, &self.title, &self.url))
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

    impl Closeable for Issue {
        fn is_open(&self) -> bool {
            is_open(&self.state)
        }
    }
}