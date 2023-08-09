use itertools::Itertools;
use reqwest::blocking::{Client as ReqwestClient, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Cookie {
    name: String,
    value: String,
}

impl Cookie {
    pub fn to_string(&self) -> String {
        format!("{}={}", self.name, self.value)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Artist {
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Release {
    pub title: String,
    artists: Vec<Artist>,
}

#[derive(Serialize, Deserialize)]
pub struct Script {
    pub authorization: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Amount {
    pub amount: usize,
}

impl Release {
    pub fn get_artists(&self) -> String {
        self.artists.iter().map(|a| a.name.to_string()).join(" ")
    }
}

#[derive(Serialize, Deserialize)]
struct Lp {
    id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct LPRelease {
    results: Vec<Lp>,
}

impl LPRelease {
    pub fn get_ids(self) -> Vec<i64> {
        self.results.iter().map(|el| el.id).collect::<Vec<i64>>()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedQuery {
    sha256_hash: String,
    version: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    operation_name: String,
    persisted_query: PersistedQuery,
}

impl Extensions {
    pub fn new(operation_name: &str, sha256_hash: &str, version: usize) -> Extensions {
        Extensions {
            operation_name: operation_name.to_string(),
            persisted_query: PersistedQuery {
                sha256_hash: sha256_hash.to_string(),
                version,
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Input {
    release_discogs_ids: Vec<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct Variables {
    input: Input,
}

impl Variables {
    pub fn new(ids: Vec<i64>) -> Variables {
        Variables {
            input: Input {
                release_discogs_ids: ids,
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AddWantlist {
    pub extensions: Extensions,
    pub variables: Variables,
}

#[derive(Serialize, Deserialize)]
struct Message {
    message: String,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorMessage {
    errors: Vec<Message>,
}

impl ErrorMessage {
    pub fn get_messages(&self) -> Vec<String> {
        self.errors
            .iter()
            .map(|m| m.message.to_string())
            .collect::<Vec<String>>()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddedItems {
    discogs_id: i64,
    added_at: String,
}

#[derive(Debug)]
pub struct Client {
    client: ReqwestClient,
    home: String,
}

impl Client {
    pub fn new(client: ReqwestClient, home: &str) -> Client {
        Client {
            client,
            home: home.to_string(),
        }
    }
    pub fn post(&self, url: &str) -> RequestBuilder {
        let url = format!("{}/{}", &self.home, url);
        self.client.post(url)
    }

    pub fn get(&self, url: &str) -> RequestBuilder {
        self.client.get(format!("{}/{}", &self.home, url))
    }
}

pub trait Send {
    fn send_request(&self) -> String;
    fn send_request_json<T: DeserializeOwned>(&self) -> T;
}

impl Send for RequestBuilder {
    fn send_request(&self) -> String {
        let res = self.send().expect("Failed to process request.");
        res.text().unwrap()
    }

    fn send_request_json<T: DeserializeOwned>(&self) -> T {
        let res = self.send().expect("Failed to process request.");
        res.json().unwrap()
    }
}

pub trait ExtendedNode {
    fn get_inner_text(&self, query: &str) -> String;
    fn get_link(&self, query: &str) -> String;
}

impl ExtendedNode for scraper::ElementRef<'_> {
    fn get_inner_text(&self, query: &str) -> String {
        let selector = scraper::Selector::parse(query).unwrap();
        self.select(&selector)
            .flat_map(|e| e.text().map(&str::trim))
            .join(" ")
    }
    fn get_link(&self, query: &str) -> String {
        let selector = scraper::Selector::parse(query).unwrap();
        match self.select(&selector).next() {
            Some(link) => link.value().attr("href").unwrap().to_string(),
            None => String::from(""),
        }
    }
}
