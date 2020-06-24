fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/manager.proto")?;
    tonic_build::compile_protos("proto/watcher.proto")?;
    Ok(())
}
