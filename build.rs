fn main() {
    // 告诉 Rust 去哪个目录下找 .lib 文件
    // 注意：这里要换成你实际存放 opencc.lib 的绝对或相对路径
    println!("cargo:rustc-link-search=native=deps/opencc/lib");
    
    // 告诉 Rust 链接 opencc 这个库
    println!("cargo:rustc-link-lib=opencc");
}
