mod cli;
mod web;

fn main() {
    let cookies_path = std::env::args().nth(1).expect("no cookies file path given");
    let scraper = web::DiscogsScraper::new(&cookies_path);
    let (links, table) = scraper.get_random_release();
    let mut print_table = true;
    loop {
        if print_table {
            cli::print_table(vec!["Sellers", "Title", "Format", "Year"], &table);
        }
        let len = links.len() as i32;
        let selected_index = cli::ask_id(len, "Select an ID:");
        if selected_index == -1 {
            std::process::exit(0);
        }
        if selected_index == len {
            break;
        }
        if links[selected_index as usize].len() == 0 {
            println!("No sellers for the selected item. Retry:");
            print_table = false;
            continue;
        }
        print_table = true;
        let selected = links.get(selected_index as usize).unwrap();
        let (sellers, table) = scraper.get_sellers(selected);
        loop {
            cli::print_table(
                vec!["Seller", "Amount", "Shipping From", "Condition", "Price"],
                &table,
            );
            let len = sellers.len() as i32;
            let selected_index = cli::ask_id(len, "Select an ID:");
            if selected_index == -1 {
                std::process::exit(0);
            }
            if selected_index == len {
                break;
            }

            let selected = &sellers[selected_index as usize];
            let (links, table) = scraper.get_seller_items(selected);
            cli::print_table(vec!["Realease", "Condition", "Price"], &table);
            loop {
                let len = links.len() as i32;
                let selected_index = cli::ask_id(len, "Want to add something to the cart?");
                if selected_index == -1 {
                    std::process::exit(0);
                }
                if selected_index == len {
                    break;
                }
                scraper.add_to_cart(&links[selected_index as usize]);
            }
        }
    }
}
