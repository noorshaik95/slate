fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Determine proto path based on environment (Docker vs local)
    let proto_path = if std::path::Path::new("./proto").exists() {
        "./proto"  // Docker build context
    } else {
        "../../proto"  // Local build context
    };
    
    // Compile auth service proto
    tonic_build::compile_protos(&format!("{}/auth.proto", proto_path))?;
    
    // Compile service auth policy proto
    tonic_build::compile_protos(&format!("{}/service_auth.proto", proto_path))?;
    
    // Compile user service proto
    tonic_build::configure()
        .build_server(false)  // Gateway is client only
        .compile_protos(
            &[&format!("{}/user.proto", proto_path)],
            &[proto_path],
        )?;
    
    Ok(())
}
