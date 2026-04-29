#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use engine::{Config, run};

fn main() {
    let config = Config::load();

    smol::block_on(async {
        run(&config).await;
    });
}
