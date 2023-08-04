use reqwest::blocking::{multipart, Client};
use reqwest::{header, redirect};
use serde::{Deserialize, Serialize};
use serde_json;

fn parse_document(doc: &str) -> scraper::Html {
    scraper::Html::parse_document(doc)
}

fn create_selector(query: &str) -> scraper::Selector {
    scraper::Selector::parse(query).unwrap()
}

#[derive(Serialize, Deserialize)]
struct Cookie {
    name: String,
    value: String,
}

pub fn create_cookie_header(path: &str) -> String {
    let data = std::fs::read_to_string(path).expect("Unable to read file");
    let parsed_cookies: Vec<Cookie> =
        serde_json::from_str(&data).expect("Unable to parse Json file.");
    parsed_cookies
        .iter()
        .map(|cookie| format!("{}={}", cookie.name, cookie.value))
        .collect::<Vec<String>>()
        .join("; ")
}

#[derive(Debug)]
pub struct DiscogsApi {
    pub user_agent: String,
    pub url: String,
    pub cookies: String,
}
impl DiscogsApi {
    pub fn get_random_release(&self) {
        let url = format!("{}/mywantlist", &self.url);
        println!("{}", url);
        let form = multipart::Form::new().text("Action.RandomItem", "Random+Item");
        let client = Client::builder()
            .user_agent(
                "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/116.0",
            )
            .redirect(redirect::Policy::none())
            .build()
            .unwrap();
        let res = client
            .post(url)
            .multipart(form)
            .header(header::COOKIE, &self.cookies)
            .send()
            .expect("Failed random item.");
        let body = res.text().unwrap();
        let document = parse_document(&body);
        let links = document
            .select(&create_selector("p a"))
            .next()
            .unwrap()
            .text()
            .collect::<Vec<_>>();
        println!("{}", links.join(" "))
    }
}
