mod config;
mod platforms;

fn main() {
    let config = config::Config::load();

    platforms::run(&config);
}
