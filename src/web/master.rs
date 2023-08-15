use super::types::*;
use super::DiscogsScraper;

const VERSION: usize = 1;
const OPERATION_NAME: &str = "AddReleasesToWantlist";
const SHA256HASH: &str = "d07fa55f88404b5d0e5253faf962ed104ad1efd3af871c9281b76e874d4a2bf4";
const GETLP: &str = "/as_json?filter=1&is_mobile=0&return_field=id&format=LP";
const ADDLPWANTLIST: &str = "service/catalog/api/graphql";

impl DiscogsScraper {
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
        let master_release_id = url.split("-").next().unwrap().to_string() + GETLP;
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
            .post(ADDLPWANTLIST)
            .body(serde_json::to_string(&add_wantlist).unwrap());
        let body = res.send_request();
        match serde_json::from_str::<ErrorMessage>(&body) {
            Ok(e) => println!("{:#?}", e.get_messages()),
            Err(_) => {
                let success_body: serde_json::Value =
                    serde_json::from_str(&body).expect("Can't parse json");
                let objects =
                    &success_body["data"]["addReleasesToWantlist"]["wantlistItems"].to_string();
                let items_added: Vec<AddedItems> = serde_json::from_str(&objects).unwrap();
                println!("Added {} items to wantlist.", items_added.len());
            }
        }
    }
}
