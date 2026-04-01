# aarch64-unknown-linux-pauthtest

**Tier: 3**

The target enables Pointer Authentication Code (PAC) support for extern "C"
calls in Rust on AArch64 ELF based Linux systems using a custom pauthtest ABI
and toolchain.

Supported features include:
* pointer authentication intrinsics
* signed function calls and returns
* pauth traps
* signing of init/fini array entries (with address diversity)
* hardened indirect control flow (`aarch64-jump-table-hardening`,
  `ptrauth-indirect-gotos`)
* signed ELF GOT entries

## Target maintainers

[@jchlanda](https://github.com/jchlanda)

## Requirements

This target supports cross-compilation from any Linux host, but execution
requires AArch64 with pointer authentication support (ARMv8.3 or higher).
Development and testing were performed on Ubuntu AArch64 24.04.3 LTS.

## Standard library support

Full std support is available `core`, `alloc`, and `std` all build successfully.
All library tests (`core`, `alloc`, `std`) pass for this target as well.

## Building the target

Building the target itself requires a PAC toolchain present on the system. In
order to build the toolchain, alongside the patched version of musl please
follow the instructions in the [build scripts
repo](https://github.com/access-softek/pauth-toolchain-build-scripts). Make sure
that the following variables are correctly set up in the [config
file](https://github.com/access-softek/pauth-toolchain-build-scripts/blob/master/config):
* `LLVM_BRANCH=`
* `MUSL_BRANCH=v1.2.5-pauth-rev2025-11-21`
* `LLVM_SHA=8e2a5e37eaf638c536dd71cb685843e8cb2aed2c` that corresponds to
  "\[DA\] Consolidate the core logic of the Weak Zero SIV tests (NFCI)
  (#185577)", top of the trunk at the time of writing this document
* `MUSL_SHA=b37ee52aff13880884a7afa8c5161a4f4f7e0236`

Make sure that the config file keeps the extra flags unset:

```
EXTRA_FLAGS_PAUTHTEST=""
EXTRA_FLAGS_MUSL=""
```

It's necessary to enable ELF GOT signing by default by applying the following
patch in `<pauth-toolchain-build-scripts-root>/src/llvm`

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

Rust compiler will make assumptions about the location of the PAC toolchain, so
if it is not installed in the standard location: `/opt/llvm-pauth` it is
necessary to provide the location through `LLVM_PAUTH` environment variable.
This is a limitation that is being investigated on.

Introduction of `aarch64-unknown-linux-pauthtest` target needs to be propagated
to some crates, so that they can correctly recognise and handle it.
Specifically:
* `cc-rs`: https://github.com/jchlanda/cc-rs/tree/jakub/cc-v1.2.28-pauthtest
* `libc`: https://github.com/jchlanda/libc/tree/jakub/0.2.178-pauthtest
* `backtrace`: https://github.com/jchlanda/backtrace-rs/tree/jakub/backtrace-v0.3.76-pauthtest

The patched versions of `cc-rs` and `libc` will have to be registered through
`[patch.crates-io]` section of `Cargo.toml` files both in:
`<rust_root>/src/bootstrap/` and `<rust_root>/library/`. Check out `cc-rs` and
`libc` to `<rust_root>/patches` and update config files. See attached diff for
details:

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

`backtrace` requires an in-tree patch to avoid incorrect pointer authentication
during unwinding (returning raw instruction pointers instead of invoking
libunwind authentication paths). Navigate to: `<rust_root>/library/backtrace`
and apply the following patch:

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

The target can be built by enabling it for a `rustc` build.

```toml
[build]
target = ["aarch64-unknown-linux-pauthtest"]
```

Make sure your toolchain is included in `$PATH`, then add it to the
`bootstrap.toml`:

```toml
[target.aarch64-unknown-linux-pauthtest]
linker = "aarch64-linux-pauthtest-clang"
```

## Building Rust programs

Rust does not currently ship precompiled artifacts for this target. Programs
must be built using a locally compiled Rust toolchain. All programs must be
dynamically linked against musl from the PAC toolchain, using provided
interpreter:

```
<toolchain_install_dir>/aarch64-linux-pauthtest/usr/lib/libc.so
```

## Cross-compilation

This target can be cross-compiled from any Linux based host, but execution must
take place on PAC aware AArch64 system.

## Testing

This target can be tested as normal with `x.py`.
The following categories are supported (all present in tree):
* Assembly tests
  * targets-aarch64_unknown_linux_pauthtest.rs
* LLVM IR/codegen tests
  * pauth-extern-c.rs
  * pauth-extern-c-direct-indirect-call.rs
  * pauth-init-fini.rs
  * pauth-attr-special-funcs.rs
  * pauth-sign-intrinsic.rs
* End-to-end execution tests
  * Rust-driven quicksort (pauth-quicksort-rust-driver)
  * C-driven quicksort (pauth-quicksort-c-driver)

All tests from `ui` and `library` subsets are expected to pass.

Command to run all passing tests:

```sh
x.py test --target aarch64-unknown-linux-pauthtest --force-rerun \
  library ui \
  tests/run-make/pauth-quicksort-rust-driver \
  tests/run-make/pauth-quicksort-c-driver \
  tests/codegen-llvm/pauth-attr-special-funcs.rs \
  tests/codegen-llvm/pauth-sign-intrinsic.rs \
  tests/codegen-llvm/pauth-extern-c-direct-indirect-call.rs \
  tests/codegen-llvm/pauth-init-fini.rs \
  tests/codegen-llvm/pauth-extern-c.rs \
  tests/assembly-llvm/targets/targets-aarch64_unknown_linux_pauthtest.rs
```

## Cross-compilation toolchains and C code

This target supports interoperability with C code. Use the [PAC-enabled LLVM
toolchain](#Building-the-target). C code must be compiled with the same
PAC-enabled toolchain. Mixed Rust/C programs are supported and tested (e.g.
quicksort examples). Pointer authentication semantics must be consistent across
Rust and C components. The target only supports dynamic linking with the custom
interpreter.
