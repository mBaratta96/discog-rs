mod web;

fn main() {
    let scraper = web::DiscogsScraper::new("./.cookies.json");
    let links = scraper.get_random_release();
    let selected = links.get(0).unwrap();
    println!("{}", selected);
    scraper.get_sellers(selected);
}
