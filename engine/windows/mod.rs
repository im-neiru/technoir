mod manager;
mod messages;
mod state;
mod wallpaper_driver;

pub async fn run(config: &crate::config::Config) {
    let state = state::State::new(config).await;

    state.enter_ui();
}
