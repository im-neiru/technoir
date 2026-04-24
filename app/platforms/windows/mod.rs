mod event_loop;
mod manager;
mod state;

pub(crate) fn run(config: &crate::config::Config) {
    let state = state::State::new(config);

    state.enter_loop();
}
