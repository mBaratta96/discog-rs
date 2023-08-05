use crate::cli;
use futures::{stream, StreamExt};
use reqwest::blocking::{multipart, Client as ReqwestClient, RequestBuilder};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, COOKIE, USER_AGENT};
use reqwest::redirect;
use serde::{Deserialize, Serialize};
use serde_json;

const WEB_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/116.0";
const API_USER_AGENT: &str = "Discogs-stats/0.0.1";
const WEB_HOME_URL: &str = "https://www.discogs.com";
const API_HOME_URL: &str = "https://api.discogs.com";
const CONCURRENT_REQUESTS: usize = 20;

#[derive(Serialize, Deserialize)]
struct Cookie {
    name: String,
    value: String,
}

#[derive(Serialize, Deserialize)]
struct Artist {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct Release {
    title: String,
    artists: Vec<Artist>,
}

#[derive(Serialize, Deserialize)]
struct Script {
    authorization: String,
}
#[derive(Serialize, Deserialize)]
struct Amount {
    amount: String,
    seller: String,
}

#[derive(Debug)]
struct Client {
    client: ReqwestClient,
    home: String,
}

#[derive(Debug)]
pub struct DiscogsScraper {
    web: Client,
    api: Client,
}

fn create_cookie_header(path: &str) -> String {
    let data = std::fs::read_to_string(path).expect("Unable to read file");
    let parsed_cookies: Vec<Cookie> =
        serde_json::from_str(&data).expect("Unable to parse Json file.");
    parsed_cookies
        .iter()
        .map(|cookie| format!("{}={}", cookie.name, cookie.value))
        .collect::<Vec<String>>()
        .join("; ")
}

trait Send {
    fn send_request(self) -> String;
}

impl Send for RequestBuilder {
    fn send_request(self) -> String {
        let res = self.send().expect("Failed random item.");
        res.text().unwrap()
    }
}

trait ExtendedNode {
    fn get_inner_text(&self, query: &str) -> String;
    fn get_link(&self, query: &str) -> String;
}

impl ExtendedNode for scraper::ElementRef<'_> {
    fn get_inner_text(&self, query: &str) -> String {
        let selector = scraper::Selector::parse(query).unwrap();
        self.select(&selector)
            .flat_map(|e| e.text().map(|t| t.trim()))
            .collect::<Vec<&str>>()
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

impl Client {
    fn post(&self, url: &str) -> RequestBuilder {
        let url = format!("{}/{}", &self.home, url);
        self.client.post(url)
    }

    fn get(&self, url: &str) -> RequestBuilder {
        self.client.get(format!("{}/{}", &self.home, url))
    }
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
        DiscogsScraper {
            web: Client {
                client: web_client,
                home: String::from(WEB_HOME_URL),
            },
            api: Client {
                client: api_client,
                home: String::from(API_HOME_URL),
            },
        }
    }
    pub fn get_random_release(&self) -> Vec<String> {
        let form = multipart::Form::new().text("Action.RandomItem", "Random+Item");
        let res = self.web.post("mywantlist").multipart(form);
        let document = scraper::Html::parse_document(&res.send_request());
        let content = &document.root_element().get_inner_text("p a");
        println!("{}", content);
        let random_release_id = content
            .split("/")
            .last()
            .unwrap()
            .split("-")
            .next()
            .unwrap()
            .to_string();
        let release_res = self
            .api
            .get(format!("releases/{}", random_release_id).as_str());
        let document = release_res.send_request();
        let release: Release = serde_json::from_str(&document).expect("Unable to parse Json file.");
        let artists = release
            .artists
            .iter()
            .map(|a| a.name.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        println!("{} {}", release.title, artists);
        let res = self
            .web
            .get(&format!("mywantlist?limit=250&search={}", release.title));
        let search_page = scraper::Html::parse_document(&res.send_request());
        let selector = scraper::Selector::parse("tr.shortcut_navigable").unwrap();
        let releases = search_page.select(&selector);
        let mut links: Vec<String> = Vec::new();
        let mut table: Vec<Vec<String>> = Vec::new();
        for node in releases {
            let album_info = node.get_inner_text("span.release_title > *:not(:last-child)");
            let album_sellers = node.get_inner_text("span.marketplace_for_sale_count");
            let format = node.get_inner_text("td[data-header='Format']");
            let year = node.get_inner_text("td[data-header='Year']");
            links.push(node.get_link("span.marketplace_for_sale_count > a"));
            table.push(vec![album_info, format, year, album_sellers]);
        }
        cli::print_table(vec!["Title", "Format", "Year", "Sellers"], table);
        links
    }

    pub fn get_sellers(&self, sellers_link: &str) -> Vec<String> {
        let res = self.web.get(sellers_link);
        let sellers_page = scraper::Html::parse_document(&res.send_request());
        let script = &sellers_page
            .root_element()
            .get_inner_text("script#dsdata")
            .replace("\n", "")[41..1702];
        let script: Script = serde_json::from_str(script).expect("Unable to parse Json file.");
        let token = script.authorization;
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let amount_urls = &sellers_page
            .root_element()
            .get_inner_text("td.seller_info div.seller_block a");
        let asynch_client = reqwest::Client::new();
        let amounts: Vec<Amount> = rt.block_on(
            stream::iter(amount_urls.split(" "))
                .map(|seller| {
                    let url = format!("{}/marketplace/mywants/{}/amount", API_HOME_URL, seller);
                    let client = &asynch_client;
                    let req = client
                        .get(&url)
                        .header(AUTHORIZATION, &token)
                        .header(USER_AGENT, API_USER_AGENT);
                    async move {
                        let res = req.send().await.unwrap();
                        let body = res.text().await.unwrap();
                        let amount: serde_json::Value =
                            serde_json::from_str(&body).expect("Error parsing json");
                        Amount {
                            amount: amount["amount"].to_string(),
                            seller: String::from(seller),
                        }
                    }
                })
                .buffer_unordered(CONCURRENT_REQUESTS)
                .collect(),
        );
        let selector = scraper::Selector::parse("tr.shortcut_navigable").unwrap();
        let mut sellers: Vec<String> = Vec::new();
        let mut table: Vec<Vec<String>> = Vec::new();
        for (i, node) in sellers_page.select(&selector).enumerate() {
            let shipping_from = &node.get_inner_text("td.seller_info ul li:nth-child(3)")[12..];
            let price = node.get_inner_text("td.item_price span.price");
            let condition =
                node.get_inner_text("p.item_condition > *:not(.condition-label-desktop)");
            // remove condition description which is always found in between media and sleeve nodes
            let condition = condition
                .split("   ")
                .enumerate()
                .filter_map(|(i, c)| if i != 1 { Some(c) } else { None })
                .collect::<Vec<&str>>()
                .join("");
            let amount = amounts.get(i).unwrap();
            sellers.push(amount.seller.clone());
            table.push(vec![
                condition,
                amount.seller.clone(),
                amount.amount.clone(),
                shipping_from.to_string(),
                price,
            ]);
        }
        cli::print_table(
            vec!["Condition", "Seller", "Amount", "Shipping From", "Price"],
            table,
        );
        sellers
    }

    pub fn get_seller_items(&self, seller: &str) {
        let url = format!("/seller/{}/mywants?limit=250&sort=price%2Casc", seller);
        let res = self.web.get(&url);
        let items_page = scraper::Html::parse_document(&res.send_request());
        let selector = scraper::Selector::parse("tr.shortcut_navigable").unwrap();
        let mut table: Vec<Vec<String>> = Vec::new();
        for node in items_page.select(&selector) {
            let release = node.get_inner_text("a.item_description_title");
            let link = node.get_link("a.item_description_title");
            let condition =
                node.get_inner_text("p.item_condition > *:not(.condition-label-desktop)");
            let price = node.get_inner_text("td.item_price span.price");
            table.push(vec![
                format!("{}\n{}{}", release, WEB_HOME_URL, link),
                condition,
                price,
            ]);
        }
        cli::print_table(vec!["Realease", "Condition", "Price"], table);
    }
}
