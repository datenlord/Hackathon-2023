use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "src/network/proto_src/metric.proto",
            "src/network/proto_src/cache.proto",
        ],
        &["src/"],
    )?;
    Ok(())
}
