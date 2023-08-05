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
    amount: i32,
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

fn send_request(req: RequestBuilder) -> String {
    let res = req.send().expect("Failed random item.");
    res.text().unwrap()
}

fn create_selector(query: &str, el: &scraper::ElementRef) -> String {
    let selector = scraper::Selector::parse(query).unwrap();
    el.select(&selector)
        .flat_map(|e| e.text().map(|t| t.trim()))
        .collect::<Vec<&str>>()
        .join(" ")
}

fn get_link(query: &str, el: &scraper::ElementRef) -> String {
    let selector = scraper::Selector::parse(query).unwrap();
    match el.select(&selector).next() {
        Some(link) => link.value().attr("href").unwrap().to_string(),
        None => String::from(""),
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
        let document = scraper::Html::parse_document(&send_request(res));
        let content = create_selector("p a", &document.root_element());
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
        let document = send_request(release_res);
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
        let search_page = scraper::Html::parse_document(&send_request(res));
        let selector = scraper::Selector::parse("tr.shortcut_navigable").unwrap();
        let releases = search_page.select(&selector);
        let mut links: Vec<String> = Vec::new();
        for (i, node) in releases.enumerate() {
            let album_info = create_selector("span.release_title > *:not(:last-child)", &node);
            let album_sellers = create_selector("span.marketplace_for_sale_count", &node);
            let format = create_selector("td[data-header='Format']", &node);
            let year = create_selector("td[data-header='Year']", &node);
            links.push(get_link("span.marketplace_for_sale_count > a", &node));
            println!(
                "{}: {}-{}-{}-{}",
                i, album_info, album_sellers, format, year
            );
        }
        links
    }

    pub fn get_sellers(&self, sellers_link: &str) {
        let res = self.web.get(sellers_link);
        let sellers_page = scraper::Html::parse_document(&send_request(res));
        let script = &create_selector("script#dsdata", &sellers_page.root_element())
            .replace("\n", "")[41..1702];
        let script: Script = serde_json::from_str(script).expect("Unable to parse Json file.");
        let token = script.authorization;
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let amount_urls = create_selector(
            "td.seller_info div.seller_block a",
            &sellers_page.root_element(),
        );
        let amounts: Vec<i32> = rt.block_on(
            stream::iter(amount_urls.split(" "))
                .map(|seller| {
                    let url = format!("{}/marketplace/mywants/{}/amount", API_HOME_URL, seller);
                    let client = reqwest::Client::new();
                    let req = client
                        .get(&url)
                        .header(AUTHORIZATION, &token)
                        .header(USER_AGENT, API_USER_AGENT);
                    async {
                        let res = req.send().await.unwrap();
                        let body = res.text().await.unwrap();
                        let amount: Amount =
                            serde_json::from_str(&body).expect("Error parsing json");
                        amount.amount
                    }
                })
                .buffer_unordered(CONCURRENT_REQUESTS)
                .collect(),
        );
        let selector = scraper::Selector::parse("tr.shortcut_navigable").unwrap();
        for (i, node) in sellers_page.select(&selector).enumerate() {
            let shipping_from = &create_selector("td.seller_info ul li:nth-child(3)", &node)[11..];
            let price = create_selector("td.item_price span.price", &node);
            let condition =
                create_selector("p.item_condition > *:not(.condition-label-desktop)", &node);
            let condition = condition
                .split("   ")
                .enumerate()
                .filter(|&(i, _)| i != 1)
                .map(|(_, v)| v)
                .collect::<Vec<&str>>()
                .join("");
            println!(
                "{}: {}-{}-{}-{}",
                i,
                shipping_from,
                price,
                condition,
                amounts.get(i).unwrap()
            );
        }
    }
}
