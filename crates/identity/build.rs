fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 用 crate 自带的 protoc，不依赖本机装的版本（§12）
    // Edition 2024 把 set_var 标为 unsafe：多线程同时读写环境变量是 UB。
    // build script 在这里是单线程、且在调用 protoc 之前设置，不存在竞争。
    unsafe {
        std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);
    }

    println!("cargo:rerun-if-changed=../../proto/identity.proto");

    // 给 tonic-reflection 用：grpcurl list 靠它发现服务。
    // 注意 build.rs 里的 OUT_DIR 是【运行时】环境变量（cargo 执行本脚本时才传入），
    // 不是编译期常量 —— 所以用 std::env::var，不能用 env!（main.rs 里才用 env!）
    let descriptor_path = format!(
        "{}/identity_descriptor.bin",
        std::env::var("OUT_DIR")?
    );

    tonic_prost_build::configure()
        .file_descriptor_set_path(descriptor_path)
        .build_server(true)
        .build_client(false)
        .compile_protos(&["../../proto/identity.proto"], &["../../proto"])?;

    Ok(())
}
