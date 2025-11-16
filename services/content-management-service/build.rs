fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the proto files from the workspace root
    let proto_path = "../../proto/content.proto";
    let proto_dir = "../../proto";
    
    // Only compile if proto file exists (for development)
    if std::path::Path::new(proto_path).exists() {
        tonic_build::configure()
            .build_server(true)
            .build_client(false)
            .compile_protos(
                &[proto_path],
                &[proto_dir],
            )?;
    }
    Ok(())
}
