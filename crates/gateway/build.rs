use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);
    }

    println!("cargo:rerun-if-changed=../../proto/identity.proto");

    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(&["../../proto/identity.proto"], &["../../proto"])?;

    Ok(())
}
