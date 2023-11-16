use std::io::Result;
fn main() -> Result<()> {
    std::env::set_var("PROTOC", protobuf_src::protoc());
    prost_build::Config::new()
        .type_attribute(
            "client.Height",
            "#[derive(Copy, rlp_derive::RlpEncodable, rlp_derive::RlpDecodable)]",
        )
        .compile_protos(
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
