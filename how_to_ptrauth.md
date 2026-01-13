# Rust Pointer Authentication
This document describes how to build and run the Rust pointer authentication development work.

## Introduction
The work-in-progress branch can be found at:
<https://github.com/jchlanda/rust/tree/jakub/pauth_experiments>

An effort will be made to version notable milestones. In that spirit, [v.0.1](https://github.com/jchlanda/rust/tree/v.0.1) marks the completion of the first stage.

## v.0.1
Work completed for this revision includes:

* `aarch64-unknown-linux-pauthtest` Rust target description
* Full integration with AccessSoftware's toolchain
* Constant/zero discrimination of external C function pointers
* Skeleton for executable, IR, and assembly testing

## Pre-requisites
Throughout this document it is assumed that work is being done on an AArch64 Linux system. Testing was specifically done on Ubuntu AArch64 24.04.3 LTS.

In principle building of the Rust compiler and compilation of tests targeting `aarch64-unknown-linux-pauthtest` can be done on any Linux based system, however for the execution AArch64 is necessary. Please see the [emulation section](https://github.com/access-softek/pauth-toolchain-build-scripts?tab=readme-ov-file#cross-debugging-with-qemu-user-and-gdb) for details on how to use qemu to run AArch64 binaries.

### LLVM-based toolchain and musl
Later on in the document it is assumed that AccessSoftware [musl](https://github.com/access-softek/musl) based toolchain is present on the system. Rust compiler will make assumptions about the location of the toolchain, so if it is not installed in the standard location: `/opt/llvm-pauth` it is necessary to provide the location through `LLVM_PAUTH` environment variable.

In order to build the toolchain, alongside the patched version of musl please follow the instructions in the [build scripts repo](https://github.com/access-softek/pauth-toolchain-build-scripts). Make sure that the following variables are correctly set up in the [config file](https://github.com/access-softek/pauth-toolchain-build-scripts/blob/master/config):
* `LLVM_BRANCH=main`
* `MUSL_BRANCH=v1.2.5-pauth-rev2025-11-21`
* `LLVM_SHA=292dc2b86f66e39f4b85ec8b185fd8b60f5213ce`, the hash corresponds to `llvmorg-21.1.7`
* `MUSL_SHA=b37ee52aff13880884a7afa8c5161a4f4f7e0236`

Furthermore it is necessary to disable init/fini signing and their address discrimination, by applying the patch in `<pauth-toolchain-build-scripts-root>/src/llvm`

```diff
diff --git a/clang/lib/Driver/ToolChains/Clang.cpp b/clang/lib/Driver/ToolChains/Clang.cpp
index 626133223..80885be82 100644
--- a/clang/lib/Driver/ToolChains/Clang.cpp
+++ b/clang/lib/Driver/ToolChains/Clang.cpp
@@ -1390,9 +1390,9 @@ static void handlePAuthABI(const ArgList &DriverArgs, ArgStringList &CC1Args) {
                          options::OPT_fno_ptrauth_indirect_gotos))
     CC1Args.push_back("-fptrauth-indirect-gotos");

-  if (!DriverArgs.hasArg(options::OPT_fptrauth_init_fini,
-                         options::OPT_fno_ptrauth_init_fini))
-    CC1Args.push_back("-fptrauth-init-fini");
+//  if (!DriverArgs.hasArg(options::OPT_fptrauth_init_fini,
+//                         options::OPT_fno_ptrauth_init_fini))
+//    CC1Args.push_back("-fptrauth-init-fini");
 }

 static void CollectARMPACBTIOptions(const ToolChain &TC, const ArgList &Args,
@@ -1738,11 +1738,11 @@ void Clang::AddAArch64TargetArgs(const ArgList &Args,

   Args.addOptInFlag(CmdArgs, options::OPT_fptrauth_indirect_gotos,
                     options::OPT_fno_ptrauth_indirect_gotos);
-  Args.addOptInFlag(CmdArgs, options::OPT_fptrauth_init_fini,
-                    options::OPT_fno_ptrauth_init_fini);
-  Args.addOptInFlag(CmdArgs,
-                    options::OPT_fptrauth_init_fini_address_discrimination,
-                    options::OPT_fno_ptrauth_init_fini_address_discrimination);
+//  Args.addOptInFlag(CmdArgs, options::OPT_fptrauth_init_fini,
+//                    options::OPT_fno_ptrauth_init_fini);
+//  Args.addOptInFlag(CmdArgs,
+//                    options::OPT_fptrauth_init_fini_address_discrimination,
+//                    options::OPT_fno_ptrauth_init_fini_address_discrimination);
   Args.addOptInFlag(CmdArgs, options::OPT_faarch64_jump_table_hardening,
                     options::OPT_fno_aarch64_jump_table_hardening);
```

## Rust
`aarch64-unknown-linux-pauthtest` was implemented as a build-in target, which does not need a description in JSON file. Entire target description is encoded in: https://github.com/jchlanda/rust/blob/v.0.1/compiler/rustc_target/src/spec/targets/aarch64_unknown_linux_pauthtest.rs

### Building
Start by checking out: https://github.com/jchlanda/rust/tree/v.0.1 and running setup command: `./x.py setup` which creates a reasonable defaults (choosing the compiler and enabling Git hooks pays off in the future).

For the vanilla checkout, building from source is described in more detail in: https://github.com/jchlanda/rust/blob/v.0.1/INSTALL.md

Introduction of `aarch64-unknown-linux-pauthtest` target needs to be propagated to crates, so they can also correctly recognise it. Create a `patches` directory in the root Rust directory and checkout the following repos (notice the branches):
* `cc-rs`: https://github.com/jchlanda/cc-rs/tree/jakub/cc-v1.2.28-pauthtest
* `libc`: https://github.com/jchlanda/libc/tree/jakub/0.2.178-pauthtest

The patched versions of `cc-rs` and `libc` will have to be registered through `[patch.crates-io]` section of `Cargo.toml` files both in: `<rust_root>/src/bootstrap/` and `<rust_root>/library/`.

See attached diff (notice that library's `Cargo.toml` file already had the `patch` section):
```diff
diff --git a/src/bootstrap/Cargo.toml b/src/bootstrap/Cargo.toml
index e1725db60cf..5a54eb43119 100644
--- a/src/bootstrap/Cargo.toml
+++ b/src/bootstrap/Cargo.toml
@@ -94,3 +94,7 @@ debug = 0
 [profile.dev.package]
 # Only use debuginfo=1 to further reduce compile times.
 bootstrap.debug = 1
+
+[patch.crates-io]
+cc = { path = '<rust_root>/patches/cc-rs' }
+libc = { path = '<rust_root>/patches/libc' }
```
and:
```diff
diff --git a/library/Cargo.toml b/library/Cargo.toml
index e30e6240942..fb5a12f0065 100644
--- a/library/Cargo.toml
+++ b/library/Cargo.toml
@@ -59,3 +59,4 @@ rustflags = ["-Cpanic=abort"]
 rustc-std-workspace-core = { path = 'rustc-std-workspace-core' }
 rustc-std-workspace-alloc = { path = 'rustc-std-workspace-alloc' }
 rustc-std-workspace-std = { path = 'rustc-std-workspace-std' }
+libc = { path = '<rust_root>/patches/libc' }
```

Next create a `config.toml` file. This is the main place where build can be customised. A minimal working version:
```toml
[rust]
debug = false

[build]
host = ["aarch64-unknown-linux-gnu"]

target = [
    "aarch64-unknown-linux-gnu",
    "aarch64-unknown-linux-pauthtest",
]
extended = true

[target.aarch64-unknown-linux-pauthtest]
linker = "/opt/llvm-pauth/bin/aarch64-linux-pauthtest-clang"

```

Finally issue: `./x.py build` followed by `./x.py build --target aarch64-unknown-linux-pauthtest`.

### Testing
* Simple ASM emission test (required by Rust's tidy policy): https://github.com/jchlanda/rust/blob/v.0.1/tests/assembly-llvm/targets/targets-aarch64_unknown_linux_pauthtest.rs
* IR generation https://github.com/jchlanda/rust/blob/v.0.1/tests/codegen-llvm/pauth-extern-c-direct-indirect-call.rs and https://github.com/jchlanda/rust/blob/v.0.1/tests/codegen-llvm/pauth-extern-c.rs
* End-to-end execution. Prior to running this test make sure that the linker, as specified in `target.aarch64-unknown-linux-pauthtest` section is on the path; if the toolchain instructions have been followed, it should be located in the `/opt/llvm-pauth/bin` directory. The test: https://github.com/jchlanda/rust/tree/v.0.1/tests/run-make/c-dynamic-linker-pauth.

    Inspection of the binary:
    * Expected format:
    ```text
    file <build_root>/aarch64-unknown-linux-gnu/test/run-make/c-dynamic-linker-pauth/rmake_out/main
    ```
    It should report as:
    ```text
    ELF 64-bit LSB pie executable, ARM aarch64, version 1 (SYSV), dynamically linked, interpreter /opt/llvm-pauth/aarch64-linux-pauthtest/usr/lib/libc.so
    ```

    * Assembly inspection:
    ```text
    /opt/llvm-pauth/bin/llvm-objdump -d <build_root>/aarch64-unknown-linux-gnu/test/run-make/c-dynamic-linker-pauth/rmake_out/main > c-dynamic-linker-pauth.s
    ```
    * Readelf and relocation:
    ```text
    /opt/llvm-pauth/bin/llvm-readelf --all <build_root>/aarch64-unknown-linux-gnu/test/run-make/c-dynamic-linker-pauth/rmake_out/main > c-dynamic-linker-pauth.txt
    ```

In order to run all the test:
```bash
./x.py test --target aarch64-unknown-linux-pauthtest --force-rerun tests/run-make/c-dynamic-linker-pauth tests/codegen-llvm/pauth-extern-c-direct-indirect-call.rs tests/codegen-llvm/pauth-extern-c.rs tests/assembly-llvm/targets/targets-aarch64_unknown_linux_pauthtest.rs
```

## Limitation
There is a known limitation that causes miscompilation of Rust standard libraries. For the time being, in order to enable pointer authenticated code, it is necessary to add an unsafe option: `-Z pauth` alongside specifying the target, see [c-dynamic-linking-pauth](https://github.com/jchlanda/rust/blob/v.0.1/tests/run-make/c-dynamic-linker-pauth/rmake.rs#L25) test as an illustration. That option is then threaded through the compiler, see [builder as an example](https://github.com/jchlanda/rust/blob/v.0.1/compiler/rustc_codegen_llvm/src/builder.rs#L1968).
