// ignore-tidy-linelength
//@ revisions: O0_PAUTH O3_PAUTH O0_PAUTH_SIGN_INIT_FINI O3_PAUTH_SIGN_INIT_FINI

// FIXME: JKB: This should really be aarch64 pauthtest
//@ [O0_PAUTH] needs-llvm-components: aarch64
//@ [O0_PAUTH] compile-flags: --target=aarch64-unknown-linux-pauthtest -C opt-level=0
//@ [O3_PAUTH] needs-llvm-components: aarch64
//@ [O3_PAUTH] compile-flags: --target=aarch64-unknown-linux-pauthtest -C opt-level=3
//@ [O0_PAUTH_SIGN_INIT_FINI] needs-llvm-components: aarch64
//@ [O0_PAUTH_SIGN_INIT_FINI] compile-flags: --target=aarch64-unknown-linux-pauthtest -Z pauth_sign_init_fini -C opt-level=0
//@ [O3_PAUTH_SIGN_INIT_FINI] needs-llvm-components: aarch64
//@ [O3_PAUTH_SIGN_INIT_FINI] compile-flags: --target=aarch64-unknown-linux-pauthtest -Z pauth_sign_init_fini -C opt-level=3

#![crate_type = "lib"]
#![feature(linkage)]

#[used]
#[link_section = ".init_array.90"]
// O0_PAUTH: @{{[0-9A-Za-z_]+}}GLOBAL_INIT = constant ptr @{{[0-9A-Za-z_]+}}init_fn, section ".init_array.90"
// O3_PAUTH: @{{[0-9A-Za-z_]+}}GLOBAL_INIT = constant ptr @{{[0-9A-Za-z_]+}}init_fn, section ".init_array.90"
// O3_PAUTH_SIGN_INIT_FINI: @{{[0-9A-Za-z_]+}}GLOBAL_INIT = constant ptr ptrauth (ptr @{{[0-9A-Za-z_]+}}init_fn, i32 0), section ".init_array.90"
// O0_PAUTH_SIGN_INIT_FINI: @{{[0-9A-Za-z_]+}}GLOBAL_INIT = constant ptr ptrauth (ptr @{{[0-9A-Za-z_]+}}init_fn, i32 0), section ".init_array.90"
static GLOBAL_INIT: extern "C" fn() = init_fn;

#[used]
#[link_section = ".fini_array.90"]
// O0_PAUTH: @{{[0-9A-Za-z_]+}}GLOBAL_FINI = constant ptr @{{[0-9A-Za-z_]+}}fini_fn, section ".fini_array.90"
// O3_PAUTH: @{{[0-9A-Za-z_]+}}GLOBAL_FINI = constant ptr @{{[0-9A-Za-z_]+}}fini_fn, section ".fini_array.90"
// O3_PAUTH_SIGN_INIT_FINI: @{{[0-9A-Za-z_]+}}GLOBAL_FINI = constant ptr ptrauth (ptr @{{[0-9A-Za-z_]+}}fini_fn, i32 0), section ".fini_array.90"
// O0_PAUTH_SIGN_INIT_FINI: @{{[0-9A-Za-z_]+}}GLOBAL_FINI = constant ptr ptrauth (ptr @{{[0-9A-Za-z_]+}}fini_fn, i32 0), section ".fini_array.90"
static GLOBAL_FINI: extern "C" fn() = fini_fn;

extern "C" fn init_fn() {}
extern "C" fn fini_fn() {}
