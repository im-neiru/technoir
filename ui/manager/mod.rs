pub struct ManagerUi {
    do_redraw: bool,
}

impl ManagerUi {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { do_redraw: true }
    }

    pub fn render(&mut self) -> (&Scene, Color) {
        (&self.scene, self.base_color)
    }

    #[inline]
    pub fn get_base_color(&self) -> Color {
        self.base_color
    }
}
