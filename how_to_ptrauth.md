# Rust Pointer Authentication
This document describes how to build and run the Rust pointer authentication development work.

## Introduction
An effort will be made to version notable milestones. In that spirit, [v.0.1.3](https://github.com/jchlanda/rust/tree/v.0.1.3) marks the most recent version of the work. Consult the [repo](https://github.com/jchlanda/rust/branches/all) for other releases.

## v.0.1.3
Work completed for this revision includes:

* `aarch64-unknown-linux-pauthtest` Rust target description
* Full integration with AccessSoftware's toolchain
* Constant/zero discrimination of external C function pointers
* Signing init/fini array entries using appropriate discriminator
* Skeleton for executable, IR, and assembly testing
* Full build of std library using pauthtest target
* All library tests (alloc, core, std) passing on pauthtest target
* All ui tests passing on pauthtest target
* Added "aarch64-jump-table-hardening", "ptrauth-indirect-gotos",
  "ptrauth-elf-got".
* Added support for address diversity in init/fini signing.

## Pre-requisites
Throughout this document it is assumed that work is being done on an AArch64 Linux system. Testing was specifically done on Ubuntu AArch64 24.04.3 LTS.

In principle building of the Rust compiler and compilation of tests targeting `aarch64-unknown-linux-pauthtest` can be done on any Linux based system, however for the execution, AArch64 is necessary. Please see the [emulation section](https://github.com/access-softek/pauth-toolchain-build-scripts?tab=readme-ov-file#cross-debugging-with-qemu-user-and-gdb) for details on how to use qemu to run AArch64 binaries on non AArch64 systems.

### PAC enabled LLVM-based toolchain and musl
Throughout this document it is assumed that AccessSoftware [musl](https://github.com/access-softek/musl) based toolchain is present on the system. Rust compiler will make assumptions about the location of the toolchain, so if it is not installed in the standard location: `/opt/llvm-pauth` it is necessary to provide the location through `LLVM_PAUTH` environment variable.

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
index a5277dcac174..2b3214f462bb 100644
--- a/clang/lib/Driver/ToolChains/Linux.cpp
+++ b/clang/lib/Driver/ToolChains/Linux.cpp
@@ -540,6 +540,10 @@ static void handlePAuthABI(const Driver &D, const ArgList &DriverArgs,
           options::OPT_fno_ptrauth_init_fini_address_discrimination))
     CC1Args.push_back("-fptrauth-init-fini-address-discrimination");

+  if (!DriverArgs.hasArg(options::OPT_fptrauth_elf_got,
+                         options::OPT_fno_ptrauth_elf_got))
+    CC1Args.push_back("-fptrauth-elf-got");
+
   if (!DriverArgs.hasArg(options::OPT_faarch64_jump_table_hardening,
                          options::OPT_fno_aarch64_jump_table_hardening))
     CC1Args.push_back("-faarch64-jump-table-hardening");

```

## Rust
`aarch64-unknown-linux-pauthtest` was implemented as a build-in target, which does not need a specification in JSON file. Entire target description is encoded in: [aarch64_unknown_linux_pauthtest.rs](https://github.com/jchlanda/rust/blob/v.0.1.3/compiler/rustc_target/src/spec/targets/aarch64_unknown_linux_pauthtest.rs).

### Building
Start by checking out the most recent [release](https://github.com/jchlanda/rust/tree/v.0.1.3) and running setup command: `x.py setup` which creates a reasonable defaults (choosing the compiler and enabling git hooks pays off in the future).

For the vanilla checkout, building from source is described in more detail in [INSTALL.md](https://github.com/jchlanda/rust/blob/v.0.1.3/INSTALL.md).

Introduction of `aarch64-unknown-linux-pauthtest` target needs to be propagated to some crates, so that they can correctly recognise and handle it. Specifically:
* `cc-rs`: https://github.com/jchlanda/cc-rs/tree/jakub/cc-v1.2.28-pauthtest
* `libc`: https://github.com/jchlanda/libc/tree/jakub/0.2.178-pauthtest
* `backtrace`: https://github.com/jchlanda/backtrace-rs/tree/jakub/backtrace-v0.3.76-pauthtest

The patched versions of `cc-rs` and `libc` will have to be registered through `[patch.crates-io]` section of `Cargo.toml` files both in: `<rust_root>/src/bootstrap/` and `<rust_root>/library/`. Check out `cc-rs` and `libc` to `<rust_root>/patches` and update config files. See attached diff for details, notice that library's `Cargo.toml` file already had the `patch` section:

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
diff --git a/src/bootstrap/Cargo.toml b/src/bootstrap/Cargo.toml
index e1725db60cf..46763cdf9a4 100644
--- a/src/bootstrap/Cargo.toml
+++ b/src/bootstrap/Cargo.toml
@@ -94,3 +94,6 @@ debug = 0
 [profile.dev.package]
 # Only use debuginfo=1 to further reduce compile times.
 bootstrap.debug = 1
+
+[patch.crates-io]
+cc = { path = '<rust_root>/patches/cc-rs' }
```

In contrast to `cc-rs` and `libc`, which are external crates resolved from crates.io and can be overridden using `[patch.crates-io]`, `backtrace` is included in the Rust repository as a git submodule under: `<rust_root>/library/backtrace`. To keep the fork clean and avoid modifying the submodule reference, we do not replace it with a custom fork at the repository level. Instead, navigate to: `<rust_root>/library/backtrace` and apply the following patch:

```diff
diff --git a/src/backtrace/libunwind.rs b/src/backtrace/libunwind.rs
index 0564f2e..a8a0d1a 100644
--- a/src/backtrace/libunwind.rs
+++ b/src/backtrace/libunwind.rs
@@ -79,6 +79,18 @@ impl Frame {
         // clause, and if this is fixed that test in theory can be run on macOS!
         if cfg!(target_vendor = "apple") {
             self.ip()
+        } else if cfg!(target_env = "pauthtest") {
+            // NOTE: As ip here is not signed (raw, non-PAC-enabled pointer) we
+            // must not use uw::_Unwind_FindEnclosingFunction. This is because,
+            // for pauthtest toolchain, libunwind will try to authenticate and
+            // resign it. Signing here (apart from risking creating a signing
+            // oracle) is not possible. According to the schema the value must
+            // be signed using SP as the discriminator - which is the problem.
+            // SP obtained here would not match the SP at the auth-resign time,
+            // as uw::_Unwind_FindEnclosingFunction creates a new context so
+            // the SP used for signing here would belong to a different frame
+            // that the one used for auth-resign. Hence return a raw value.
+            self.ip()
         } else {
             unsafe { uw::_Unwind_FindEnclosingFunction(self.ip()) }
         }
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
* Simple ASM emission test (required by Rust's tidy policy): [targets-aarch64_unknown_linux_pauthtest.rs](https://github.com/jchlanda/rust/blob/v.0.1.3/tests/assembly-llvm/targets/targets-aarch64_unknown_linux_pauthtest.rs)
* IR generation [pauth-extern-c-direct-indirect-call.rs](https://github.com/jchlanda/rust/blob/v.0.1.3/tests/codegen-llvm/pauth-extern-c-direct-indirect-call.rs) and [pauth-extern-c.rs](https://github.com/jchlanda/rust/blob/v.0.1.3/tests/codegen-llvm/pauth-extern-c.rs)
* Enabling of init/fini signing: [pauth-init-fini.rs](https://github.com/jchlanda/rust/blob/v.0.1.3/tests/codegen-llvm/pauth-init-fini.rs)
* Attributes added to compiler generated functions [pauth-attr-special-funcs.rs](https://github.com/jchlanda/rust/blob/v.0.1.3/tests/codegen-llvm/pauth-attr-special-funcs.rs)
* Correct sign intrinsic generation [pauth-sign-intrinsic.rs](https://github.com/jchlanda/rust/blob/v.0.1.3/tests/codegen-llvm/pauth-sign-intrinsic.rs)
* End-to-end execution. Prior to running those tests make sure that the linker, as specified in `target.aarch64-unknown-linux-pauthtest` section is on the path; if the toolchain instructions have been followed, it should be located in the `/opt/llvm-pauth/bin` directory.

  * Rust drives the program, by providing the data and the comparison function, C implements the quicksort algorithm: [pauth-quicksort-rust-driver.](https://github.com/jchlanda/rust/tree/v.0.1.3/tests/run-make/pauth-quicksort-rust-driver.)

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

  * C drives the program, by providing the data and the comparison function, Rust implements the quicksort algorithm: [pauth-quicksort-c-driver](https://github.com/jchlanda/rust/tree/v.0.1.3/tests/run-make/pauth-quicksort-c-driver). This is the mirror reflection of the above (in terms of which language is responsible for what).

In order to run all the test:
```bash
x.py test --target aarch64-unknown-linux-pauthtest --force-rerun tests/run-make/pauth-quicksort-rust-driver tests/run-make/pauth-quicksort-c-driver tests/codegen-llvm/pauth-attr-special-funcs.rs tests/codegen-llvm/pauth-sign-intrinsic.rs tests/codegen-llvm/pauth-extern-c-direct-indirect-call.rs tests/codegen-llvm/pauth-init-fini.rs tests/codegen-llvm/pauth-extern-c.rs tests/assembly-llvm/targets/targets-aarch64_unknown_linux_pauthtest.rs
```

The current version passes all the test from the `ui` and `library` subset (`alloc`, `core` and `std`). This can be verified by running:
```bash
x.py test ui library --target aarch64-unknown-linux-pauthtest
```

## Limitation
Operand bundles should only be attached to indirect function calls. However, as function pointer signing is unstable, we end up signing too eagerly (including functions used for direct calls), hence operand bundles are added to all calls. The issue is further discussed in a ticket at [rust-lang issue tracker](https://github.com/rust-lang/rust/issues/152532).

The test coverage is limited; except for the tests explicitly mentioned above, no guarantees are made regarding the state of other tests.
