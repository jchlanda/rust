// Test compilation flow using custom pauth-enabled toolchain and signing extern "C" function
// pointers used from withing rust. Note that in order for the test to work the toolchain has to be
// provided via env variable (LLVM_PAUTH), or present at `/opt/llvm-pauth`.
// In this test c is the driver - providing the data and the comparison function; while rust -
// provides the implementation of quicksort algorithm and is the user of  the data and comparator.

//FIXME: JKB: Limit it to only only-aarch64-linux-pauthtest

use run_make_support::{cc, rfs, run, run_fail, rustc};

fn main() {
    let root = std::env::var("LLVM_PAUTH").unwrap_or_else(|_| "/opt/llvm-pauth".into());

    let clang_path = format!("{}/bin/clang", root);
    unsafe {
        std::env::set_var("CC", clang_path);
    }
    let dynamic_linker =
        format!("-Wl,--dynamic-linker={}/aarch64-linux-pauthtest/usr/lib/libc.so", root);
    let rpath = format!("-Wl,--rpath={}/aarch64-linux-pauthtest/usr/lib", root);

    let rust_lib_name = "rust_quicksort";
    rustc()
        .target("aarch64-unknown-linux-pauthtest")
        .crate_type("cdylib")
        .input("quicksort.rs")
        .crate_name(rust_lib_name)
        .args(&[&dynamic_linker, &rpath])
        .run();

    let exe_name = "main";
    cc().out_exe(exe_name)
        .input("main.c")
        .args(&[
            "-march=armv8.3-a",
            "-target",
            "aarch64-unknown-linux-pauthtest",
            "-L.",
            &format!("-l{}", rust_lib_name),
            &dynamic_linker,
            &rpath,
        ])
        .run();

    run(exe_name);

    rfs::remove_file(format!("{}{rust_lib_name}.{}", "lib", "so"));
    run_fail(exe_name);
}
