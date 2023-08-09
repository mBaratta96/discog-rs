mod cli;
mod web;

use clap::Parser;
use cli::Commands::*;
use cli::TableType;
use core::iter::zip;

const WANTIST_HEADER: &[&str] = &["Sellers", "Title", "Format", "Year"];
const SELLERS_HEADER: &[&str] = &["Seller", "Amount", "Shipping From", "Condition", "Price"];
const ITEMS_HEADER: &[&str] = &["Realease", "Condition", "Price"];
const RELEASE_HEADER: &[&str] = &["Release", "Status", "Info", "Details"];
const CART_HEADER: &[&str] = &["Description", "Price"];

fn check_wantlist(scraper: web::DiscogsScraper) {
    let (links, table) = scraper.get_random_release();
    let mut print_table = true;
    loop {
        if print_table {
            cli::print_table(WANTIST_HEADER, &table, "Releases", TableType::Default);
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
        let table = scraper.get_sellers(selected);
        loop {
            cli::print_table(SELLERS_HEADER, &table, "Sellers", TableType::Default);
            let len = table.len() as i32;
            let selected_index = cli::ask_id(len, "Select an ID:");
            if selected_index == -1 {
                std::process::exit(0);
            }
            if selected_index == len {
                break;
            }
            let selected = &table[selected_index as usize][0];
            let (links, table) = scraper.get_seller_items(selected);
            cli::print_table(
                ITEMS_HEADER,
                &table,
                &format!("{} Items", selected),
                TableType::Default,
            );
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
    cli::print_table(
        RELEASE_HEADER,
        &table,
        "Master Releases",
        TableType::Default,
    );
    let len = links.len() as i32;
    let selected_index = cli::ask_id(len, "Select an ID:");
    if selected_index == -1 {
        std::process::exit(0);
    }
    let link = &links[selected_index as usize];
    scraper.add_lps_to_wantlist(link);
}

fn get_cart(scraper: web::DiscogsScraper) {
    let (sellers, tables) = scraper.get_cart();
    for (seller, table) in zip(sellers, tables) {
        cli::print_table(CART_HEADER, &table, &seller, TableType::Cart);
    }
}

fn main() {
    let args = cli::Args::parse();
    let cookies_path = args.cookies;
    let scraper = web::DiscogsScraper::new(&cookies_path);
    match args.command {
        Wantlist => check_wantlist(scraper),
        Add { release } => add_to_wantlist(scraper, &release),
        Cart => get_cart(scraper),
    }
}
