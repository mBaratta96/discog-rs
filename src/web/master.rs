use super::types::*;
use super::DiscogsScraper;

const VERSION: usize = 1;
const ADD_OPERATION_NAME: &str = "AddReleasesToWantlist";
const ADD_SHA256HASH: &str = "d07fa55f88404b5d0e5253faf962ed104ad1efd3af871c9281b76e874d4a2bf4";
const GETLP: &str = "/as_json?filter=1&is_mobile=0&return_field=id&format=LP";
const GRAPHQL_URL: &str = "service/catalog/api/graphql";
const REMOVE_OPERATION_NAME: &str = "RemoveReleasesFromWantlist";
const REMOVE_SHA256HASH: &str = "ab4a277f4c5d9da56ba17d4b88643c51a1935f500813133c55fe5a340625d06f";

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

    fn graphql_post_request(&self, url: &str, operation: &str, sha256hash: &str) -> String {
        let master_release_id = url.split("-").next().unwrap().to_string() + GETLP;
        let res = self.web.get(&master_release_id);
        let results: LPRelease = res.send_request_json();
        let extensions = Extensions::new(operation, sha256hash, VERSION);
        let variables = Variables::new(results.get_ids());
        let add_wantlist = AddWantlist {
            extensions,
            variables,
        };
        let res = self
            .web
            .post(GRAPHQL_URL)
            .body(serde_json::to_string(&add_wantlist).unwrap());
        res.send_request()
    }

    pub fn add_lps_to_wantlist(&self, url: &str) {
        let response = self.graphql_post_request(url, ADD_OPERATION_NAME, ADD_SHA256HASH);
        match serde_json::from_str::<ErrorMessage>(&response) {
            Ok(e) => println!("{:#?}", e.get_messages()),
            Err(_) => {
                let success_body: serde_json::Value =
                    serde_json::from_str(&response).expect("Can't parse json");
                let objects =
                    &success_body["data"]["addReleasesToWantlist"]["wantlistItems"].to_string();
                let items_added: Vec<AddedItems> = serde_json::from_str(&objects).unwrap();
                println!("Added {} items to wantlist.", items_added.len());
            }
        }
    }

    pub fn remove_all_wantlist(&self, url: &str) {
        let response = self.graphql_post_request(url, REMOVE_OPERATION_NAME, REMOVE_SHA256HASH);
        match serde_json::from_str::<ErrorMessage>(&response) {
            Ok(e) => println!("{:#?}", e.get_messages()),
            Err(_) => {
                let success_body: serde_json::Value =
                    serde_json::from_str(&response).expect("Can't parse json");
                let objects = &success_body["data"]["removeReleasesFromWantlist"].to_string();
                let items_added: RemovedItems = serde_json::from_str(&objects).unwrap();
                if items_added.success {
                    println!("Items removed");
                } else {
                    println!("Error in removing items");
                }
            }
        }
    }
}
