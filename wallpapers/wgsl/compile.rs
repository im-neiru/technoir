use naga::{
    front::wgsl,
    valid::{Capabilities, ValidationFlags, Validator},
};

use std::path::Path;

pub fn compile_shaders(output_dir: impl AsRef<Path>) {
    let output_dir = output_dir.as_ref();
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    let mut errors = vec![];

    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).expect("Failed to create shaders directory");
    }

    for shader in crate::ShaderSource::ALL {
        let module = wgsl::parse_str(shader.source()).expect("Failed to parse shader module");

        if let Err(e) = validator.validate(&module) {
            errors.push(e);
            continue;
        };

        let out_file = std::fs::File::create(output_dir.join(format!("{}.naga", shader.name())))
            .expect("Failed to create shaders directory");
        ciborium::into_writer(&module, out_file).expect("Failed to write shader module");

        // TODO when available

        // // SPIR-V
        // {
        //     let spv_path = output_dir.join(format!("{}.spv", shader.name()));
        //     let spv_options = naga::back::spv::Options::default();
        //     let spv = naga::back::spv::write_vec(&module, &info, &spv_options, None)
        //         .expect("Failed to compile to SPIR-V");

        //     let bytes = spv
        //         .iter()
        //         .flat_map(|&u| u.to_le_bytes())
        //         .collect::<Vec<u8>>();

        //     std::fs::write(spv_path, bytes).expect("Failed to write SPIR-V file");
        // }

        // Metal
        // {
        //     let msl_path = output_dir.join(format!("{}.metal", shader.name()));
        //     let msl_options = naga::back::msl::Options::default();
        //     let mut output = String::new();
        //     let mut msl_writer = naga::back::msl::Writer::new(&mut output);
        //     let pipeline_options = naga::back::msl::PipelineOptions::default();

        //     msl_writer
        //         .write(&module, &info, &msl_options, &pipeline_options)
        //         .expect("Failed to compile to Metal");

        //     std::fs::write(msl_path, output).expect("Failed to write Metal file");
        // }

        // // HLSL
        // {
        //     let hlsl_path = output_dir.join(format!("{}.hlsl", shader.name()));
        //     let hlsl_options = naga::back::hlsl::Options::default();
        //     let mut output = String::new();
        //     let pipeline_options = naga::back::hlsl::PipelineOptions::default();
        //     let mut hlsl_writer =
        //         naga::back::hlsl::Writer::new(&mut output, &hlsl_options, &pipeline_options);
        //     hlsl_writer
        //         .write(&module, &info, None)
        //         .expect("Failed to compile to HLSL");

        //     std::fs::write(hlsl_path, output).expect("Failed to write HLSL file");
        // }
    }

    let mut errors_str = String::new();

    for e in errors {
        errors_str += &format!("{e}\n");
    }

    if !errors_str.is_empty() {
        panic!("{errors_str}");
    }
}
