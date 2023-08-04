mod web;

fn main() {
    let api = web::DiscogsApi {
        user_agent: String::from("Discogs-stats/0.0.1"),
        url: String::from("https://www.discogs.com"),
        cookies: web::create_cookie_header("./.cookies.json"),
    };
    api.get_random_release()
}
