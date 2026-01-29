fn main() {
    // 只有在 Windows 平台编译时才执行
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

        // 指向你存放 .lib 文件的绝对路径
        // 这里的路径要对应你项目里的真实位置，比如 deps/clib/lib
        println!(
            "cargo:rustc-link-search=native={}/deps/clib/lib",
            manifest_dir
        );

        // 告诉它链接这两个库
        println!("cargo:rustc-link-lib=opencc");
        println!("cargo:rustc-link-lib=marisa");
    }
}
