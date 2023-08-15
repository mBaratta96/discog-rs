mod cli;
mod web;

use clap::Parser;
use cli::Commands::*;
use cli::{MenuOptions, TableType};
use core::iter::zip;

const WANTIST_HEADER: &[&str] = &["Sellers", "Title", "Format", "Year"];
const SELLERS_HEADER: &[&str] = &["Seller", "Amount", "Shipping From", "Condition", "Price"];
const ITEMS_HEADER: &[&str] = &["Realease", "Condition", "Price"];
const RELEASE_HEADER: &[&str] = &["Release", "Status", "Info", "Details"];
const CART_HEADER: &[&str] = &["Description", "Price"];

#[derive(Debug)]
enum WantlistOperations {
    Add,
    Remove,
}

fn check_wantlist(scraper: web::DiscogsScraper, query: Option<String>) {
    let (links, table) = scraper.get_release(query);
    let mut print_table = true;
    if links.len() == 0 {
        println!("No items in your wantlist");
        std::process::exit(0);
    }
    loop {
        if print_table {
            cli::print_table(WANTIST_HEADER, &table, "Releases", TableType::Default);
        }
        let mut selected_index: usize;
        match cli::select_operation() {
            MenuOptions::SelectId => selected_index = cli::ask_id(links.len(), "Select an Id:"),
            MenuOptions::Exit => std::process::exit(0),
            MenuOptions::GoBack => break,
        };
        if links[selected_index as usize].len() == 0 {
            println!("No sellers for the selected item. Retry:");
            print_table = false;
            continue;
        }
        print_table = true;
        let selected = &links[selected_index];
        let table = scraper.get_sellers(selected);
        loop {
            cli::print_table(SELLERS_HEADER, &table, "Sellers", TableType::Default);
            match cli::select_operation() {
                MenuOptions::SelectId => selected_index = cli::ask_id(table.len(), "Select an Id:"),
                MenuOptions::Exit => std::process::exit(0),
                MenuOptions::GoBack => break,
            };
            let selected = &table[selected_index][0];
            let (links, table) = scraper.get_seller_items(selected);
            cli::print_table(
                ITEMS_HEADER,
                &table,
                &format!("{} Items", selected),
                TableType::Default,
            );
            loop {
                match cli::select_operation() {
                    MenuOptions::SelectId => {
                        selected_index = cli::ask_id(links.len(), "Select an Id:")
                    }
                    MenuOptions::Exit => std::process::exit(0),
                    MenuOptions::GoBack => break,
                };
                scraper.add_to_cart(&links[selected_index as usize]);
            }
        }
    }
}

fn master_release_to_wantlist(
    scraper: web::DiscogsScraper,
    search: &str,
    operation: WantlistOperations,
) {
    let (links, table) = scraper.search_release(&search);
    cli::print_table(
        RELEASE_HEADER,
        &table,
        "Master Releases",
        TableType::Default,
    );
    let selected_index: usize;
    match cli::select_operation() {
        MenuOptions::SelectId => selected_index = cli::ask_id(links.len(), "Select an Id:"),
        _ => std::process::exit(0),
    };
    let link = &links[selected_index];
    match operation {
        WantlistOperations::Add => scraper.add_lps_to_wantlist(&link),
        WantlistOperations::Remove => scraper.remove_all_wantlist(&link),
    }
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
        Wantlist { query } => check_wantlist(scraper, query),
        Add { release } => master_release_to_wantlist(scraper, &release, WantlistOperations::Add),
        Remove { release } => {
            master_release_to_wantlist(scraper, &release, WantlistOperations::Remove)
        }
        Cart => get_cart(scraper),
    }
}
