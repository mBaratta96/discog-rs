mod cart;
mod master;
mod types;
mod wantlist;
use itertools::Itertools;
use reqwest::blocking::Client as ReqwestClient;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use reqwest::redirect;
use serde_json;
use types::*;

const WEB_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/116.0";
const API_USER_AGENT: &str = "Discogs-stats/0.0.1";
const WEB_HOME_URL: &str = "https://www.discogs.com";
const API_HOME_URL: &str = "https://api.discogs.com";

fn create_cookie_header(path: &str) -> String {
    let data = std::fs::read_to_string(path).expect("Unable to read file");
    let parsed_cookies: Vec<Cookie> =
        serde_json::from_str(&data).expect("Unable to parse Json file.");
    parsed_cookies.iter().map(Cookie::to_string).join("; ")
}

#[derive(Debug)]
pub struct DiscogsScraper {
    web: Client,
    api: Client,
}

impl DiscogsScraper {
    pub fn new(path: &str) -> DiscogsScraper {
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&create_cookie_header(path)).unwrap(),
        );
        let web_client = ReqwestClient::builder()
            .user_agent(WEB_USER_AGENT)
            .redirect(redirect::Policy::none())
            .default_headers(headers)
            .build()
            .unwrap();
        let api_client = ReqwestClient::builder()
            .user_agent(API_USER_AGENT)
            .build()
            .unwrap();
        let scraper = DiscogsScraper {
            web: Client::new(web_client, WEB_HOME_URL),
            api: Client::new(api_client, API_HOME_URL),
        };
        scraper
    }
}
