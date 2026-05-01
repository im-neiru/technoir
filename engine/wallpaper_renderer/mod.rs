pub struct WallpaperRenderer {
    instance: wgpu::Instance,
}

impl WallpaperRenderer {
    pub fn new(instance: &wgpu::Instance) -> Self {
        let instance = instance.clone();

        Self { instance }
    }
}
