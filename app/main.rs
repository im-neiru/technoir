mod config;
mod platforms;

fn main() {
    let config = config::Config::load();

    smol::block_on(async {
        platforms::run(&config).await;
    });
}
