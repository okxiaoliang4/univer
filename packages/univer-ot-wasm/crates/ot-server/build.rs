fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);

    // Proto files are in the workspace root
    let proto_path = std::path::PathBuf::from("../../proto");

    // Compile OT RPC proto
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("ot_rpc_descriptor.bin"))
        .compile(&[proto_path.join("ot_rpc.proto")], &[proto_path.clone()])?;

    // Compile user service proto (for token verification)
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(&[proto_path.join("user.proto")], &[proto_path.clone()])?;

    // Compile document service proto (for permission checks)
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(&[proto_path.join("document.proto")], &[proto_path.clone()])?;

    println!("cargo:rerun-if-changed=../../proto/ot_rpc.proto");
    println!("cargo:rerun-if-changed=../../proto/user.proto");
    println!("cargo:rerun-if-changed=../../proto/document.proto");
    Ok(())
}
