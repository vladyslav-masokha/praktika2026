fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path()
        .expect("Failed to find bundled protoc");

    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    prost_build::compile_protos(
        &["../proto/order_events.proto"],
        &["../proto"],
    )
    .expect("Failed to compile proto files");
}
