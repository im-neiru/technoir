use engine::{Config, run};

fn main() {
    let config = Config::load();

    smol::block_on(async {
        run(&config).await;
    });
}
