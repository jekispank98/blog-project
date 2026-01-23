fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(&["proto/blog.proto"], &["proto"])?;
    println!("cargo:rerun-if-changed=proto/blog.proto");
    Ok(())
}
