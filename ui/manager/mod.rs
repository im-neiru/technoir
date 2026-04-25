use vello::{Scene, peniko::Color};

pub struct ManagerUi {
    scene: Scene,
    base_color: Color,
    do_redraw: bool,
}

impl ManagerUi {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            scene: Scene::new(),
            base_color: Color::from_rgb8(8, 12, 12),
            do_redraw: true,
        }
    }

    pub fn render(&mut self) -> (&Scene, Color) {
        if self.do_redraw {
            self.scene.reset();
            self.scene.fill(
                vello::peniko::Fill::NonZero,
                vello::kurbo::Affine::IDENTITY,
                Color::WHITE,
                None,
                &vello::kurbo::Circle::new((80.0, 80.0), 40.0),
            );

            self.do_redraw = false;
        }

        (&self.scene, self.base_color)
    }

    #[inline]
    pub fn get_base_color(&self) -> Color {
        self.base_color
    }
}
