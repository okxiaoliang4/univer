fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("CARGO_FEATURE_SERVER").is_err() {
        return Ok(());
    }

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(&["proto/ot_rpc.proto"], &["proto"])?;

    println!("cargo:rerun-if-changed=proto/ot_rpc.proto");
    Ok(())
}
