mod cli;
mod web;

fn main() {
    let scraper = web::DiscogsScraper::new("./.cookies.json");
    let (links, table) = scraper.get_random_release();
    loop {
        cli::print_table(vec!["Title", "Format", "Year", "Sellers"], table.clone());
        let len = links.len() as i32;
        let selected_index = cli::ask_id(len);
        if selected_index == -1 {
            std::process::exit(0);
        }
        if selected_index == len {
            break;
        }
        let selected = links.get(selected_index as usize).unwrap();
        let (sellers, table) = scraper.get_sellers(selected);
        loop {
            cli::print_table(
                vec!["Condition", "Seller", "Amount", "Shipping From", "Price"],
                table.clone(),
            );
            let len = sellers.len() as i32;
            let selected_index = cli::ask_id(len);
            if selected_index == -1 {
                std::process::exit(0);
            }
            if selected_index == len {
                break;
            }

            let selected = sellers.get(selected_index as usize).unwrap();
            scraper.get_seller_items(selected);
            break;
        }
    }
}
