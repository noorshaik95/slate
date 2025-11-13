fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile auth service proto
    tonic_build::compile_protos("proto/auth.proto")?;
    
    // Compile service auth policy proto
    tonic_build::compile_protos("proto/service_auth.proto")?;
    
    Ok(())
}
