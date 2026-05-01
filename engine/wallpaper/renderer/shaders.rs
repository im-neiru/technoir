use wgpu::{ShaderModule, ShaderModuleDescriptor};

pub use wgsl_shaders::ShaderSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ShaderKey {
    BuiltIn(ShaderSource),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderIndex(usize);

impl<'r> super::PrepareContext<'r> {
    pub fn load_built_in_shader(
        &mut self,
        shader_source: ShaderSource,
    ) -> (ShaderIndex, &ShaderModule) {
        let device = &self.inner.device;

        self.inner
            .shaders
            .get_or_insert(ShaderKey::BuiltIn(shader_source), || {
                let descriptor = shader_source.descriptor();
                let module = device.create_shader_module(descriptor);

                Ok::<_, std::convert::Infallible>(module)
            })
            .expect("Shader creation is infallible")
    }

    pub fn get_shader_module(&self, index: ShaderIndex) -> Option<&wgpu::ShaderModule> {
        self.inner.shaders.get_by_index(index.0)
    }
}

impl<'r> super::CleanupContext<'r> {
    pub fn release_shader(&mut self, index: ShaderIndex) {
        self.inner.shaders.release_by_index(index.0);
    }
}

impl From<usize> for ShaderIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value)
    }
}
