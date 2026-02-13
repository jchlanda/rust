//@ assembly-output: emit-asm
// ignore-tidy-linelength
//@ revisions: aarch64_unknown_linux_pauthtest
//@ [aarch64_unknown_linux_pauthtest] compile-flags: --target=aarch64-unknown-linux-pauthtest
// FIXME: JKB: This should really be aarch64-pauthtest
//@ [aarch64_unknown_linux_pauthtest] needs-llvm-components: aarch64

#![no_std]
#![crate_type = "lib"]

#[no_mangle]
#[inline(never)]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
#[inline(never)]
pub fn call_add(x: i32) -> i32 {
    add(x, 1)
}

// CHECK: adrp    x16, :got:add
// CHECK: ldr x16, [x16, :got_lo12:add]
// CHECK: paciza  x16
