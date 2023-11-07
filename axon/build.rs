use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "src/proto/connection.proto",
            "src/proto/client.proto",
            "src/proto/connection.proto",
            "src/proto/channel.proto",
        ],
        &["src/"],
    )?;
    Ok(())
}
