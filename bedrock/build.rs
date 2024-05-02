use spirv_builder::{MetadataPrintout, SpirvBuilder, SpirvMetadata};

use std::fs::copy;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = SpirvBuilder::new("../shader", "spirv-unknown-vulkan1.2")
        .print_metadata(MetadataPrintout::Full)
        .spirv_metadata(SpirvMetadata::Full)
        .build()?;

    copy(result.module.unwrap_single(), "./spirv/shader.spv")?;
    Ok(())
}
