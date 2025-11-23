fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the proto files from the workspace root
    let proto_path = "../../proto/content.proto";
    let proto_dir = "../../proto";

    // Only compile if proto file exists (for development)
    if std::path::Path::new(proto_path).exists() {
        tonic_build::configure()
            .build_server(true)
            .build_client(false)
            .file_descriptor_set_path(
                std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("descriptor.bin"),
            )
            .compile_protos(
                &[
                    proto_path,
                    "../../proto/google/protobuf/timestamp.proto",
                    "../../proto/google/protobuf/empty.proto",
                ],
                &[proto_dir],
            )?;
    }
    Ok(())
}
