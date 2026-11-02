use super::api::ApiState;

#[derive(Default)]
pub(super) struct InstanceState {
    pub(super) api: ApiState,
}
