mod desktop_handles;
mod event_loop;
mod manager;
mod screen;
mod state;

pub(crate) async fn run(config: &crate::config::Config) {
    let state = state::State::new(config).await;

    state.enter_loop();
}
