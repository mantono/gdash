pub mod issue;

pub mod state {

    pub trait Closeable {
        fn is_open(&self) -> bool;
    }

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

    pub fn is_open(state: &Option<State>) -> bool {
        match state {
            Some(s) => match s {
                State::Open => true,
                State::Closed => false
            }
            _ => false
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
            let args: Vec<String> = env::args().skip(1).collect();
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