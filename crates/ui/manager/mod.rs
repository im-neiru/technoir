pub struct ManagerUi {
    do_redraw: bool,
}

impl ManagerUi {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { do_redraw: true }
    }

    pub fn render(&mut self) {}
}
