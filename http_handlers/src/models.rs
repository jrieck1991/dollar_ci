use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::result;

#[derive(Deserialize, Serialize, Debug)]
pub struct Event {
    pub action: String,
    pub check_suite: CheckSuite,
    pub repository: Repo,
    pub installation: Installation,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CheckSuite {
    pub id: u64,
    pub status: String,
    pub head_sha: String,
    pub check_runs_url: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Repo {
    pub full_name: String,
    pub clone_url: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Installation {
    pub id: u64,
}

// result type alias
pub type Result<T> = result::Result<T, HandlersErr>;

// Error wrapper for the project
#[derive(Debug)]
pub enum HandlersErr {
    Json(serde_json::Error),
    Client(reqwest::Error),
    Jwt(jsonwebtoken::errors::Error),
    Io(std::io::Error),
    NotFound,
}

// implement the Display trait to eventually fulfill the Error trait
impl fmt::Display for HandlersErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HandlersErr::Json(ref err) => write!(f, "json error: {}", err),
            HandlersErr::Client(ref err) => write!(f, "client error: {}", err),
            HandlersErr::Jwt(ref err) => write!(f, "jwt error: {}", err),
            HandlersErr::Io(ref err) => write!(f, "IO error: {}", err),
            HandlersErr::NotFound => write!(f, "{}", &"not found"),
        }
    }
}

// implement the Error trait
impl Error for HandlersErr {
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            HandlersErr::Json(ref err) => Some(err),
            HandlersErr::Client(ref err) => Some(err),
            HandlersErr::Jwt(ref err) => Some(err),
            HandlersErr::Io(ref err) => Some(err),
            HandlersErr::NotFound => Some(&HandlersErr::NotFound),
        }
    }
}

// implement From for each error to be wrapped
impl From<reqwest::Error> for HandlersErr {
    fn from(err: reqwest::Error) -> HandlersErr {
        HandlersErr::Client(err)
    }
}

impl From<serde_json::Error> for HandlersErr {
    fn from(err: serde_json::Error) -> HandlersErr {
        HandlersErr::Json(err)
    }
}

impl From<jsonwebtoken::errors::Error> for HandlersErr {
    fn from(err: jsonwebtoken::errors::Error) -> HandlersErr {
        HandlersErr::Jwt(err)
    }
}

impl From<std::io::Error> for HandlersErr {
    fn from(err: std::io::Error) -> HandlersErr {
        HandlersErr::Io(err)
    }
}
