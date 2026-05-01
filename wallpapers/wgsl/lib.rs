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

            pub fn descriptor<'a>(&self) -> wgpu::ShaderModuleDescriptor<'a> {
                let path = format!("shaders/{}.naga", self.name());
                let source = std::fs::File::open(&path).expect("Failed to open shader file");

                let module: wgpu::naga::Module = ciborium::from_reader(source).expect("Failed to deserialize shader module");

                wgpu::ShaderModuleDescriptor {
                    label: Some(self.name()),
                    source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(module)),
                }
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
