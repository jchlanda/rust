// FIXME: Jakub: Add description.

// FIXME: Jakub:
// Till pauthtest target is added run it with RUST_TARGET_PATH and json explicitly, for exammple:
// $ RUST_TARGET_PATH=$(pwd) ./x.py test --target aarch64-linux-pauthtest c-dynamic-linker-pauth

//@ only-aarch64-linux-pauthtest

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

    let dynamic_linker =
        format!("link-arg=-Wl,--dynamic-linker={}/aarch64-linux-pauthtest/usr/lib/libc.so", root,);
    let rpath = format!("link-arg=-Wl,--rpath={}/aarch64-linux-pauthtest/usr/lib", root,);
    let libpath = format!("link-arg=-L{}/aarch64-linux-pauthtest/usr/lib", root,);
    rustc()
        .input("main.rs")
        .args(&[
            "-C",
            "target-feature=-crt-static",
            "-Z",
            "pauth",
            "-C",
            "link-arg=-lunwind",
            "-C",
            &dynamic_linker,
            "-C",
            &rpath,
            "-C",
            &libpath,
        ])
        .run();
    run("main");

    rfs::remove_file(&lib_name);
    run_fail("main");
}
