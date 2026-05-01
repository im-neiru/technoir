use wgpu::ShaderModuleDescriptor;

pub use wgsl_shaders::ShaderSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ShaderKey {
    BuiltIn(ShaderSource),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderIndex(usize);

impl super::WallpaperRenderer {
    pub fn load_built_in_shader(&mut self, shader_source: ShaderSource) -> ShaderIndex {
        if let Some((index, _key, _value)) =
            self.shaders.get_full(&ShaderKey::BuiltIn(shader_source))
        {
            return ShaderIndex(index);
        }

        let descriptor = shader_source.descriptor();
        let module = self.device.create_shader_module(descriptor);

        let (index, _) = self
            .shaders
            .insert_full(ShaderKey::BuiltIn(shader_source), module);

        ShaderIndex(index)
    }

    pub fn get_shader_module(&self, index: ShaderIndex) -> Option<&wgpu::ShaderModule> {
        let (_key, module) = self.shaders.get_index(index.0)?;

        Some(module)
    }
}
