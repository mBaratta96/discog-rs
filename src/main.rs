mod web;

fn main() {
    let scraper = web::DiscogsScraper::new("./.cookies.json");
    let links = scraper.get_random_release();
}
