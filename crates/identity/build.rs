fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 用 crate 自带的 protoc，不依赖本机装的版本（§12）
    // Edition 2024 把 set_var 标为 unsafe：多线程同时读写环境变量是 UB。
    // build script 在这里是单线程、且在调用 protoc 之前设置，不存在竞争。
    unsafe {
        std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);
    }

    println!("cargo:rerun-if-changed=../../proto/identity.proto");

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(&["../../proto/identity.proto"], &["../../proto"])?;

    Ok(())
}
