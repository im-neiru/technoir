#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use engine::{Config, Entry};

fn main() {
    let config = Config::load();
    let entry = smol::block_on(async { Entry::new(config).await });

    entry.run();
}
