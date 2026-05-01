#[cfg(feature = "compile")]
pub mod compile;

#[cfg(feature = "compile")]
pub use compile::compile_shaders;

macro_rules! define_shaders {
    ($($variant:ident => $file_name:expr),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum ShaderSource {
            $($variant),*
        }

        impl ShaderSource {
            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant => $file_name),*
                }
            }

            pub fn path(&self, backend: wgpu::Backend) -> String {
                let ext = match backend {
                    wgpu::Backend::Metal => "metal",
                    wgpu::Backend::Vulkan => "spv",
                    wgpu::Backend::Dx12 => "hlsl",
                    _ => unreachable!("Unsupported backend: {:?}", backend),
                };
                format!("shaders/{}.{}", self.name(), ext)
            }
        }

        #[cfg(feature = "compile")]
        impl ShaderSource {
            pub fn source(&self) -> &'static str {
                match self {
                    $(Self::$variant => include_str!(concat!($file_name, ".wgsl"))),*
                }
            }

            pub const ALL: &[ShaderSource] = &[
                $(ShaderSource::$variant),*
            ];
        }
    };
}

define_shaders! {
    CosmicBubble => "cosmic_bubble",
}
