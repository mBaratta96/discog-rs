mod types;

use futures::{stream, StreamExt};
use itertools::Itertools;
use reqwest::blocking::{multipart, Client as ReqwestClient};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, COOKIE, USER_AGENT};
use reqwest::redirect;
use serde_json;
use types::*;

const WEB_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/116.0";
const API_USER_AGENT: &str = "Discogs-stats/0.0.1";
const WEB_HOME_URL: &str = "https://www.discogs.com";
const API_HOME_URL: &str = "https://api.discogs.com";
const CONCURRENT_MAX_REQUESTS: usize = 50;
const VERSION: usize = 1;
const OPERATION_NAME: &str = "AddReleasesToWantlist";
const SHA256HASH: &str = "d07fa55f88404b5d0e5253faf962ed104ad1efd3af871c9281b76e874d4a2bf4";

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
        DiscogsScraper {
            web: Client::new(web_client, WEB_HOME_URL),
            api: Client::new(api_client, API_HOME_URL),
        }
    }
    pub fn get_random_release(&self) -> (Vec<String>, Vec<Vec<String>>) {
        let form = multipart::Form::new().text("Action.RandomItem", "Random+Item");
        let res = self.web.post("mywantlist").multipart(form);
        let document = scraper::Html::parse_document(&res.send_request());
        let content = &document.root_element().get_inner_text("p a");
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
        let release: Release = release_res.send_request_json();
        let artists = release.get_artists();
        println!("Found {} - {}", release.title, artists);
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
            table.push(vec![album_sellers, album_info, format, year]);
        }
        (links, table)
    }

    pub fn get_sellers(&self, sellers_link: &str) -> (Vec<String>, Vec<Vec<String>>) {
        let res = self.web.get(sellers_link);
        let sellers_page = scraper::Html::parse_document(&res.send_request());
        let script = &sellers_page
            .root_element()
            .get_inner_text("script#dsdata")
            .replace("\n", "")[41..1702];
        let script: serde_json::Value =
            serde_json::from_str(script).expect("Unable to parse Json file.");
        let token = script["authorization"].to_string();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let sellers_names: Vec<String> = sellers_page
            .root_element()
            .get_inner_text("td.seller_info div.seller_block a")
            .split(" ")
            .map(&str::to_string)
            .collect();
        let asynch_client = reqwest::Client::new();
        let amounts: Vec<String> = rt.block_on(
            stream::iter(sellers_names.iter())
                .take(CONCURRENT_MAX_REQUESTS)
                .map(|seller| {
                    let url = format!("{}/marketplace/mywants/{}/amount", API_HOME_URL, seller);
                    let client = &asynch_client;
                    let req = client
                        .get(&url)
                        .header(AUTHORIZATION, &token)
                        .header(USER_AGENT, API_USER_AGENT);
                    async move {
                        let body = req.send().await.unwrap().text().await.unwrap();
                        let amount: serde_json::Value =
                            serde_json::from_str(&body).expect("Error parsing json");
                        amount["amount"].to_string()
                    }
                })
                .buffer_unordered(CONCURRENT_MAX_REQUESTS)
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
                .join("");
            let seller = &sellers_names[i];
            let amount = if i < CONCURRENT_MAX_REQUESTS {
                &amounts[i]
            } else {
                ""
            };
            sellers.push(seller.to_string());
            table.push(vec![
                seller.to_string(),
                amount.to_string(),
                shipping_from.to_string(),
                condition,
                price,
            ]);
        }

        (sellers, table)
    }

    pub fn get_seller_items(&self, seller: &str) -> (Vec<String>, Vec<Vec<String>>) {
        let url = format!("/seller/{}/mywants?limit=250&sort=price%2Casc", seller);
        let res = self.web.get(&url);
        let items_page = scraper::Html::parse_document(&res.send_request());
        let selector = scraper::Selector::parse("tr.shortcut_navigable").unwrap();
        let mut table: Vec<Vec<String>> = Vec::new();
        let mut links: Vec<String> = Vec::new();
        for node in items_page.select(&selector) {
            let release = node.get_inner_text("a.item_description_title");
            let link = node.get_link("a.item_description_title");
            let condition =
                node.get_inner_text("p.item_condition > *:not(.condition-label-desktop)");
            let price = node.get_inner_text("td.item_price span.price");
            links.push(node.get_link("td.item_add_to_cart > a.button"));
            table.push(vec![
                format!("{}\n{}{}", release, WEB_HOME_URL, link),
                condition,
                price,
            ]);
        }
        (links, table)
    }

    pub fn add_to_cart(&self, link: &str) {
        self.web.get(link).send_request();
    }

    pub fn search_release(&self, search: &str) -> (Vec<String>, Vec<Vec<String>>) {
        let url = format!(
            "search/?q={}&type=master&layout=sm",
            search.replace(" ", "+")
        );
        let res = self.web.get(&url);
        let search_page = scraper::Html::parse_document(&res.send_request());
        let selector = scraper::Selector::parse("li.card div.card_body").unwrap();
        let mut table: Vec<Vec<String>> = Vec::new();
        let mut links: Vec<String> = Vec::new();
        for node in search_page.select(&selector) {
            let release = node
                .get_inner_text("h4[role='none']")
                .trim()
                .replace("  ", " ");
            links.push(node.get_link("h4[role='none'] > a"));
            let status = node.get_inner_text("p.card_status");
            let info = node.get_inner_text("p.card_info").trim().replace("  ", " ");
            let details = node.get_inner_text("div.search_result_details");
            table.push(vec![release, status, info, details]);
        }
        (links, table)
    }

    pub fn add_lps_to_wantlist(&self, url: &str) {
        let master_release_id = url.split("-").next().unwrap().to_owned()
            + "/as_json?filter=1&is_mobile=0&return_field=id&format=LP";
        let res = self.web.get(&master_release_id);
        let results: LPRelease = res.send_request_json();
        let extensions = Extensions::new(OPERATION_NAME, SHA256HASH, VERSION);
        let variables = Variables::new(results.get_ids());
        let add_wantlist = AddWantlist {
            extensions,
            variables,
        };
        let res = self
            .web
            .post("service/catalog/api/graphql")
            .body(serde_json::to_string(&add_wantlist).unwrap());
        let body = res.send_request();
        println!("{}", body);
    }
}
