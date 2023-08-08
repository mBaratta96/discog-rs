mod cli;
mod web;

use clap::Parser;

fn check_wantlist(scraper: web::DiscogsScraper) {
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
        let selected = &links[selected_index as usize];
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

fn add_to_wantlist(scraper: web::DiscogsScraper, search: &str) {
    let (links, table) = scraper.search_release(&search);
    cli::print_table(vec!["Release", "Status", "Info", "Details"], &table);
    let len = links.len() as i32;
    let selected_index = cli::ask_id(len, "Select an ID:");
    if selected_index == -1 {
        std::process::exit(0);
    }
    let link = &links[selected_index as usize];
    scraper.add_lps_to_wantlist(link);
}

fn main() {
    let args = cli::Args::parse();
    let cookies_path = args.cookies;
    let scraper = web::DiscogsScraper::new(&cookies_path);
    if args.wantlist.len() > 0 {
        add_to_wantlist(scraper, &args.wantlist);
    } else {
        check_wantlist(scraper);
    }
}
