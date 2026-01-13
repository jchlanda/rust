// Test compilation flow using custom pauth-enabled toolchain and signing extern "C" function
// pointers used from withing rust. Note that in order for the test to work the toolchain has to be
// provided via env variable (LLVM_PAUTH), or present at `/opt/llvm-pauth`.

//FIXME: JKB: Limit it to only only-aarch64-linux-pauthtest

use run_make_support::{cc, rfs, run, run_fail, rustc};

fn main() {
    let root = std::env::var("LLVM_PAUTH").unwrap_or_else(|_| "/opt/llvm-pauth".into());

    let clang_path = format!("{}/bin/clang", root);
    unsafe {
        std::env::set_var("CC", clang_path);
    }

    let input = "cquicksort";
    let input_name = format!("{input}.c");
    let lib_name = format!("{}{input}.{}", "lib", "so");
    cc().out_exe(&lib_name)
        .input(&input_name)
        .args(&["-target", "aarch64-linux-pauthtest", "-fPIC", "-shared"])
        .run();

    rustc().target("aarch64-unknown-linux-pauthtest").input("main.rs").args(&["-Z", "pauth"]).run();
    run("main");

    rfs::remove_file(&lib_name);
    run_fail("main");
}
