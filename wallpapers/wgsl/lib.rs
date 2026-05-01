#[cfg(feature = "compile")]
pub mod compile;

#[cfg(feature = "compile")]
pub use compile::compile_shaders;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderSource {
    CosmicBubble,
}

impl ShaderSource {
    fn name(&self) -> &'static str {
        match self {
            ShaderSource::CosmicBubble => "cosmic_bubble",
        }
    }

    pub fn path(&self, backend: wgpu::Backend) -> &str {
        match backend {
            wgpu::Backend::Metal => "shaders/cosmic_bubble.metal",
            wgpu::Backend::Vulkan => "shaders/cosmic_bubble.spv",
            wgpu::Backend::Dx12 => "shaders/cosmic_bubble.hlsl",
            _ => unreachable!("Unsupported backend: {backend:?}"),
        }
    }
}

#[cfg(feature = "compile")]
impl ShaderSource {
    fn source(&self) -> &'static str {
        match self {
            ShaderSource::CosmicBubble => include_str!("cosmic_bubble.wgsl"),
        }
    }

    const ALL: [ShaderSource; 1] = [ShaderSource::CosmicBubble];
}
