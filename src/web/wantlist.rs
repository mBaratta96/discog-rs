use super::types::*;
use super::DiscogsScraper;
use futures::{stream, StreamExt};
use itertools::Itertools;
use reqwest::blocking::multipart;
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use std::collections::HashMap;

const CONCURRENT_MAX_REQUESTS: usize = 50;

impl DiscogsScraper {
    pub fn get_release(&self, query: Option<String>) -> (Vec<String>, Vec<Vec<String>>) {
        let search = match query {
            Some(search) => search,
            None => {
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
                    .unwrap();
                let url = format!("releases/{}", random_release_id);
                let res = self.api.get(&url);
                let release: Release = res.send_request_json();
                let artists = release.get_artists();
                println!("Found: {} - {}", artists, release.title);
                format!("{} {}", release.title, artists)
            }
        };
        let res = self
            .web
            .get(&format!("mywantlist?limit=250&search={}", search));
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

    pub fn get_sellers(&self, sellers_link: &str) -> Vec<Vec<String>> {
        let res = self.web.get(sellers_link);
        let sellers_page = scraper::Html::parse_document(&res.send_request());
        let script = sellers_page
            .root_element()
            .get_inner_text("script#dsdata")
            .replace("\n", "");
        let script: Script =
            serde_json::from_str(&script[41..1702]).expect("Unable to parse Json file.");
        let token = script.authorization;
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let sellers = sellers_page
            .root_element()
            .get_inner_text("td.seller_info div.seller_block a");
        let sellers_names: Vec<&str> = sellers.split(" ").collect();
        let asynch_client = reqwest::Client::new();
        let amounts: HashMap<&str, usize> = rt.block_on(
            stream::iter(&sellers_names)
                .take(CONCURRENT_MAX_REQUESTS)
                .map(|seller| {
                    let url = format!(
                        "{}/marketplace/mywants/{}/amount",
                        super::API_HOME_URL,
                        seller
                    );
                    let client = &asynch_client;
                    let req = client
                        .get(&url)
                        .header(AUTHORIZATION, &token)
                        .header(USER_AGENT, super::WEB_USER_AGENT);
                    async move {
                        let res = req.send().await.unwrap();
                        if res.headers()["X-Discogs-Ratelimit-Remaining"] == "10" {
                            println!("WARNING: less than 10 API calls available!");
                        }
                        let body = res.text().await.unwrap();
                        let amount: Amount =
                            serde_json::from_str(&body).expect("Error parsing json");
                        (*seller, amount.amount)
                    }
                })
                .buffer_unordered(CONCURRENT_MAX_REQUESTS)
                .collect(),
        );

        let selector = scraper::Selector::parse("tr.shortcut_navigable").unwrap();
        let mut table: Vec<Vec<String>> = Vec::new();
        for (i, node) in sellers_page.select(&selector).enumerate() {
            let shipping_from =
                node.get_inner_text("td.seller_info ul li:nth-child(3)")[12..].to_string();
            let price = node.get_inner_text("td.item_price span.price");
            // remove condition description which is always found in between media and sleeve nodes
            let condition = node
                .get_inner_text("p.item_condition > *:not(.condition-label-desktop)")
                .split("   ")
                .enumerate()
                .filter_map(|(i, c)| if i != 1 { Some(c) } else { None })
                .join("");
            let seller = sellers_names[i].to_string();
            let amount = match amounts.get(sellers_names[i]) {
                Some(amount) => amount.to_string(),
                None => "".to_string(),
            };
            table.push(vec![seller, amount, shipping_from, condition, price]);
        }

        table
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
                format!("{}\n{}{}", release, super::WEB_HOME_URL, link),
                condition,
                price,
            ]);
        }
        (links, table)
    }

    pub fn add_to_cart(&self, link: &str) {
        self.web.get(link).send_request();
    }
}
