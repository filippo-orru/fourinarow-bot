mod player_bot;
mod watcher;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("watch") => watcher::main(),
        Some("bot") => player_bot::main(),
        _ => {
            println!("No command specified! Either 'watch' or 'bot'");
        }
    }
}
