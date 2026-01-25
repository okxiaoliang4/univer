fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("CARGO_FEATURE_SERVER").is_err() {
        return Ok(());
    }

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("ot_rpc_descriptor.bin"))
        .compile(&["proto/ot_rpc.proto"], &["proto"])?;

    println!("cargo:rerun-if-changed=proto/ot_rpc.proto");
    Ok(())
}
