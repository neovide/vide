use spirv_builder::{MetadataPrintout, SpirvBuilder};

use std::fs::copy;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = SpirvBuilder::new("../shader", "spirv-unknown-vulkan1.1")
        .print_metadata(MetadataPrintout::Full)
        .extra_arg("--no-spirt")
        .build()?;

    copy(result.module.unwrap_single(), "./spirv/shader.spv")?;
    Ok(())
}
