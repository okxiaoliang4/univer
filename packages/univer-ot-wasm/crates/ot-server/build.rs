fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);

    // Proto files are in the workspace root
    let proto_path = std::path::PathBuf::from("../../proto");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("ot_rpc_descriptor.bin"))
        .compile(
            &[proto_path.join("ot_rpc.proto")],
            &[proto_path],
        )?;

    println!("cargo:rerun-if-changed=../../proto/ot_rpc.proto");
    Ok(())
}
