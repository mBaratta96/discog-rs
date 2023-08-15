use super::types::*;
use super::DiscogsScraper;
use itertools::Itertools;

const CART: &str = "sell/cart";

impl DiscogsScraper {
    pub fn get_cart(&self) -> (Vec<String>, Vec<Vec<Vec<String>>>) {
        let res = self.web.get(CART);
        let cart_page = scraper::Html::parse_document(&res.send_request());
        let selector = scraper::Selector::parse("div.orders form").unwrap();
        let mut tabels: Vec<Vec<Vec<String>>> = Vec::new();
        let mut sellers: Vec<String> = Vec::new();
        for node in cart_page.select(&selector) {
            let mut table: Vec<Vec<String>> = Vec::new();
            let selector = scraper::Selector::parse("table.order_list_table tr.order_row").unwrap();
            for item in node.select(&selector) {
                let name = item.get_inner_text("td.order-item-info a.item_link");
                let condition = item
                    .get_inner_text("td.order-item-info span.item_condition")
                    .split("   ")
                    .enumerate()
                    .filter_map(|(i, c)| if i != 1 { Some(c) } else { None })
                    .join("")
                    .split_whitespace()
                    .join(" ");
                let link = item.get_link("td.order-item-info a.item_link");
                let price = item.get_inner_text("td.price");
                table.push(vec![
                    format!("{}\n{}\n{}{}", name, condition, super::WEB_HOME_URL, link),
                    price,
                ]);
            }
            let subtotal =
                node.get_inner_text("div.order_summary tr.order_subtotal td.order_summary_value");
            let subtotal_price: f32 = subtotal.split(" ").next().unwrap()[3..].parse().unwrap();
            table.push(vec![String::from("Subtotal"), subtotal]);
            let selector =
                scraper::Selector::parse("div.order_summary select.shipping_method option")
                    .unwrap();
            for option in node.select(&selector) {
                let description = option.get_text();
                let parsed_price: f32 =
                    option.value().attr("data-amount").unwrap().parse().unwrap();
                let total_price = subtotal_price + parsed_price;
                table.push(vec![description, format!("€{:.2} EUR", total_price)]);
            }
            tabels.push(table);
            let seller_name = node.get_inner_text("div.box-header-row span.linked_username");
            let seller_rating = node
                .get_inner_text("div.box-header-row span.inline_rating small")
                .split_whitespace()
                .join(" ");
            sellers.push(format!("{} {}", seller_name, seller_rating));
        }
        (sellers, tabels)
    }
}
