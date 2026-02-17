# Rust Pointer Authentication
This document describes how to build and run the Rust pointer authentication development work.

## Introduction
An effort will be made to version notable milestones. In that spirit, [v.0.1.1](https://github.com/jchlanda/rust/tree/v.0.1.1) marks the most recent version of the work.

## v.0.1.1
Work completed for this revision includes:

* `aarch64-unknown-linux-pauthtest` Rust target description
* Full integration with AccessSoftware's toolchain
* Constant/zero discrimination of external C function pointers
* Signing init/fini array entries using appropriate discriminator
* Skeleton for executable, IR, and assembly testing
* Full build of std library using pauthtest target

## Pre-requisites
Throughout this document it is assumed that work is being done on an AArch64 Linux system. Testing was specifically done on Ubuntu AArch64 24.04.3 LTS.

In principle building of the Rust compiler and compilation of tests targeting `aarch64-unknown-linux-pauthtest` can be done on any Linux based system, however for the execution, AArch64 is necessary. Please see the [emulation section](https://github.com/access-softek/pauth-toolchain-build-scripts?tab=readme-ov-file#cross-debugging-with-qemu-user-and-gdb) for details on how to use qemu to run AArch64 binaries.

### LLVM-based toolchain and musl
Later on in the document it is assumed that AccessSoftware [musl](https://github.com/access-softek/musl) based toolchain is present on the system. Rust compiler will make assumptions about the location of the toolchain, so if it is not installed in the standard location: `/opt/llvm-pauth` it is necessary to provide the location through `LLVM_PAUTH` environment variable.

In order to build the toolchain, alongside the patched version of musl please follow the instructions in the [build scripts repo](https://github.com/access-softek/pauth-toolchain-build-scripts). Make sure that the following variables are correctly set up in the [config file](https://github.com/access-softek/pauth-toolchain-build-scripts/blob/master/config):
* `LLVM_BRANCH=`
* `MUSL_BRANCH=v1.2.5-pauth-rev2025-11-21`
* `LLVM_SHA=4d6fb8834216ba559c7baa73c0ef7f2b6998341a` that corresponds to "\[PAC\]\[Driver\] Support ptrauth flags only on ARM64 Darwin or with pauthtest ABI (#113152)"
* `MUSL_SHA=b37ee52aff13880884a7afa8c5161a4f4f7e0236`

Keep extra flags empty:

```
EXTRA_FLAGS_PAUTHTEST=""
EXTRA_FLAGS_MUSL=""
```

At the time of writing we support ptrauth intrinsics, calls, returns, auth-traps as well as signing entries in init/fini array. It's necessary to disable other functionality by applying the following patch in `<pauth-toolchain-build-scripts-root>/src/llvm`

```diff
diff --git a/clang/lib/Driver/ToolChains/Linux.cpp b/clang/lib/Driver/ToolChains/Linux.cpp
index 94a9fe8b1a63..c87ef9f0791b 100644
--- a/clang/lib/Driver/ToolChains/Linux.cpp
+++ b/clang/lib/Driver/ToolChains/Linux.cpp
@@ -499,37 +499,37 @@ static void handlePAuthABI(const Driver &D, const ArgList &DriverArgs,
                          options::OPT_fno_ptrauth_auth_traps))
     CC1Args.push_back("-fptrauth-auth-traps");

-  if (!DriverArgs.hasArg(
-          options::OPT_fptrauth_vtable_pointer_address_discrimination,
-          options::OPT_fno_ptrauth_vtable_pointer_address_discrimination))
-    CC1Args.push_back("-fptrauth-vtable-pointer-address-discrimination");
-
-  if (!DriverArgs.hasArg(
-          options::OPT_fptrauth_vtable_pointer_type_discrimination,
-          options::OPT_fno_ptrauth_vtable_pointer_type_discrimination))
-    CC1Args.push_back("-fptrauth-vtable-pointer-type-discrimination");
-
-  if (!DriverArgs.hasArg(
-          options::OPT_fptrauth_type_info_vtable_pointer_discrimination,
-          options::OPT_fno_ptrauth_type_info_vtable_pointer_discrimination))
-    CC1Args.push_back("-fptrauth-type-info-vtable-pointer-discrimination");
-
-  if (!DriverArgs.hasArg(options::OPT_fptrauth_indirect_gotos,
-                         options::OPT_fno_ptrauth_indirect_gotos))
-    CC1Args.push_back("-fptrauth-indirect-gotos");
+  // if (!DriverArgs.hasArg(
+  //         options::OPT_fptrauth_vtable_pointer_address_discrimination,
+  //         options::OPT_fno_ptrauth_vtable_pointer_address_discrimination))
+  //   CC1Args.push_back("-fptrauth-vtable-pointer-address-discrimination");
+  //
+  // if (!DriverArgs.hasArg(
+  //         options::OPT_fptrauth_vtable_pointer_type_discrimination,
+  //         options::OPT_fno_ptrauth_vtable_pointer_type_discrimination))
+  //   CC1Args.push_back("-fptrauth-vtable-pointer-type-discrimination");
+  //
+  // if (!DriverArgs.hasArg(
+  //         options::OPT_fptrauth_type_info_vtable_pointer_discrimination,
+  //         options::OPT_fno_ptrauth_type_info_vtable_pointer_discrimination))
+  //   CC1Args.push_back("-fptrauth-type-info-vtable-pointer-discrimination");
+  //
+  // if (!DriverArgs.hasArg(options::OPT_fptrauth_indirect_gotos,
+  //                        options::OPT_fno_ptrauth_indirect_gotos))
+  //   CC1Args.push_back("-fptrauth-indirect-gotos");

   if (!DriverArgs.hasArg(options::OPT_fptrauth_init_fini,
                          options::OPT_fno_ptrauth_init_fini))
     CC1Args.push_back("-fptrauth-init-fini");

-  if (!DriverArgs.hasArg(
-          options::OPT_fptrauth_init_fini_address_discrimination,
-          options::OPT_fno_ptrauth_init_fini_address_discrimination))
-    CC1Args.push_back("-fptrauth-init-fini-address-discrimination");
-
-  if (!DriverArgs.hasArg(options::OPT_faarch64_jump_table_hardening,
-                         options::OPT_fno_aarch64_jump_table_hardening))
-    CC1Args.push_back("-faarch64-jump-table-hardening");
+  // if (!DriverArgs.hasArg(
+  //         options::OPT_fptrauth_init_fini_address_discrimination,
+  //         options::OPT_fno_ptrauth_init_fini_address_discrimination))
+  //   CC1Args.push_back("-fptrauth-init-fini-address-discrimination");
+  //
+  // if (!DriverArgs.hasArg(options::OPT_faarch64_jump_table_hardening,
+  //                        options::OPT_fno_aarch64_jump_table_hardening))
+  //   CC1Args.push_back("-faarch64-jump-table-hardening");
 }

 void Linux::addClangTargetOptions(const llvm::opt::ArgList &DriverArgs,
```

## Rust
`aarch64-unknown-linux-pauthtest` was implemented as a build-in target, which does not need a specification in JSON file. Entire target description is encoded in: https://github.com/jchlanda/rust/blob/v.0.1.1/compiler/rustc_target/src/spec/targets/aarch64_unknown_linux_pauthtest.rs

### Building
Start by checking out: https://github.com/jchlanda/rust/tree/v.0.1.1 and running setup command: `x.py setup` which creates a reasonable defaults (choosing the compiler and enabling Git hooks pays off in the future).

For the vanilla checkout, building from source is described in more detail in: https://github.com/jchlanda/rust/blob/v.0.1.1/INSTALL.md

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

If the patched crates are not picked up, manually modify corresponding `Cargo.lock` files. For example:
* library:

```diff
diff --git a/library/Cargo.lock b/library/Cargo.lock
index accbbe9d236..86dd4b887d4 100644
--- a/library/Cargo.lock
+++ b/library/Cargo.lock
@@ -146,9 +146,7 @@ dependencies = [

 [[package]]
 name = "libc"
-version = "0.2.177"
-source = "registry+https://github.com/rust-lang/crates.io-index"
-checksum = "2874a2af47a2325c2001a6e6fad9b16a53b802102b528163885171cf92b15976"
+version = "0.2.178"
 dependencies = [
  "rustc-std-workspace-core",
 ]
```

* bootstrap:

```diff
diff --git a/src/bootstrap/Cargo.lock b/src/bootstrap/Cargo.lock
index 884f67e91e6..453f1fd5321 100644
--- a/src/bootstrap/Cargo.lock
+++ b/src/bootstrap/Cargo.lock
@@ -96,8 +96,6 @@ dependencies = [
 [[package]]
 name = "cc"
 version = "1.2.28"
-source = "registry+https://github.com/rust-lang/crates.io-index"
-checksum = "4ad45f4f74e4e20eaa392913b7b33a7091c87e59628f4dd27888205ad888843c"
 dependencies = [
  "shlex",
 ]
@@ -380,9 +378,7 @@ checksum = "bbd2bcb4c963f2ddae06a2efc7e9f3591312473c50c6685e1f298068316e66fe"

 [[package]]
 name = "libc"
-version = "0.2.174"
-source = "registry+https://github.com/rust-lang/crates.io-index"
-checksum = "1171693293099992e19cddea4e8b849964e9846f4acee11b3948bcc337be8776"
+version = "0.2.178"

 [[package]]
 name = "libredox"
```

If `x.py setup` run correctly it should have generated `bootstrap.toml` file. This is the main configuration point for building rustc. By default it will contain `profile` and `chagne-id`. Extend the file with the following:

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

[install]
prefix = "<root_install_dir>"
sysconfdir = "etc"
```

Note `<root_install_dir>` which needs replacing with a desired installation directory.

Finally issue: `x.py build --target aarch64-unknown-linux-pauthtest` followed by `x.py install`. Upon completion the toolchain will be available at `<root_install_dir>`. Verify the support for pauthtest target by running:

```
<rust_install_dir>/bin/rustc --print target-list | grep pauthtest
```

Which should return the target `aarch64-unknown-linux-pauthtest`.

### Testing
* Simple ASM emission test (required by Rust's tidy policy): https://github.com/jchlanda/rust/blob/v.0.1.1/tests/assembly-llvm/targets/targets-aarch64_unknown_linux_pauthtest.rs
* IR generation https://github.com/jchlanda/rust/blob/v.0.1.1/tests/codegen-llvm/pauth-extern-c-direct-indirect-call.rs and https://github.com/jchlanda/rust/blob/v.0.1.1/tests/codegen-llvm/pauth-extern-c.rs
* Enabling of init/fini signing: https://github.com/jchlanda/rust/blob/v.0.1.1/tests/codegen-llvm/pauth-init-fini.rs
* End-to-end execution. Prior to running those tests make sure that the linker, as specified in `target.aarch64-unknown-linux-pauthtest` section is on the path; if the toolchain instructions have been followed, it should be located in the `/opt/llvm-pauth/bin` directory.

  * Rust drives the program, by providing the data and the comparison function, C implements the quicksort algorithm: https://github.com/jchlanda/rust/tree/v.0.1.1/tests/run-make/pauth-quicksort-rust-driver.

    Inspection of the binary:
    * Expected format:
    ```text
    file
    <build_root>/aarch64-unknown-linux-gnu/test/run-make/pauth-quicksort-rust-driver/rmake_out/main
    ```
    It should report as:
    ```text
    ELF 64-bit LSB pie executable, ARM aarch64, version 1 (SYSV), dynamically linked, interpreter /opt/llvm-pauth/aarch64-linux-pauthtest/usr/lib/libc.so
    ```

    * Assembly inspection:
    ```text
    /opt/llvm-pauth/bin/llvm-objdump -d <build_root>/aarch64-unknown-linux-gnu/test/run-make/pauth-quicksort-rust-driver/rmake_out/main > pauth-quicksort-rust-driver.s
    ```
    * Readelf and relocation:
    ```text
    /opt/llvm-pauth/bin/llvm-readelf --all <build_root>/aarch64-unknown-linux-gnu/test/run-make/pauth-quicksort-rust-driver/rmake_out/main > pauth-quicksort-rust-driver.txt
    ```

  * C drives the program, by providing the data and the comparison function, Rust implements the quicksort algorithm: https://github.com/jchlanda/rust/tree/v.0.1.1/tests/run-make/pauth-quicksort-c-driver. This is the mirror reflection of the above (in terms of which language is responsible for what).

In order to run all the test:
```bash
x.py test --target aarch64-unknown-linux-pauthtest --force-rerun tests/run-make/pauth-quicksort-rust-driver tests/run-make/pauth-quicksort-c-driver tests/codegen-llvm/pauth-extern-c-direct-indirect-call.rs tests/codegen-llvm/pauth-init-fini.rs tests/codegen-llvm/pauth-extern-c.rs tests/assembly-llvm/targets/targets-aarch64_unknown_linux_pauthtest.rs
```

## Limitation
Operand bundles should only be attached to indirect function calls. However, as function signing is unstable, we end up signing too eagerly (including direct function calls), hence operand bundles are added to all calls. The issue is discussed in further ticket in a [rust-lang ticket](https://github.com/rust-lang/rust/issues/152532).
