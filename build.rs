use spirv_builder::SpirvBuilder;
use std::fs::copy;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>>  {
    let result = SpirvBuilder::new("./shader", "spirv-unknown-vulkan1.1")
            .build()?;

    copy(result.module.unwrap_single(), "./spirv/shader.spv")?;

    Ok(())
}
