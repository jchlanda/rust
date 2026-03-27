//@ assembly-output: emit-asm
// ignore-tidy-linelength
//@ only-aarch64-unknown-linux-pauthtest
//@ revisions: aarch64_unknown_linux_pauthtest
//@ [aarch64_unknown_linux_pauthtest] compile-flags: --target=aarch64-unknown-linux-pauthtest
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
fn call_through(f: extern "C" fn(i32, i32) -> i32, x: i32) -> i32 {
    f(x, 1)
}

#[no_mangle]
#[inline(never)]
pub fn call_add(x: i32) -> i32 {
    call_through(add, x)
}

// CHECK-LABEL: call_through:
// CHECK:       mov     [[PTR:x[0-9]+]], x0
// CHECK:       mov     w0, w1
// CHECK:       mov     w1, #1
// CHECK:       braaz   [[PTR]]

// CHECK-LABEL: call_add:
// CHECK:       adrp    [[GOT_REG:x[0-9]+]], :got_auth:add
// CHECK:       add     [[GOT_REG]], [[GOT_REG]], :got_auth_lo12:add
// CHECK:       ldr     [[FN_REG:x[0-9]+]], [[[GOT_REG]]]
// CHECK:       autia   [[FN_REG]], [[GOT_REG]]
// CHECK:       mov     [[TMP_REG:x[0-9]+]], [[FN_REG]]
// CHECK:       xpaci   [[TMP_REG]]
// CHECK:       cmp     [[FN_REG]], [[TMP_REG]]
// CHECK:       b.eq    [[SUCCESS:\.Lauth_success_0]]
// CHECK:       brk     #0xc470
// CHECK:       [[SUCCESS]]:
// CHECK:       paciza  [[FN_REG]]
// CHECK:       mov     w1, w0
// CHECK:       mov     x0, [[FN_REG]]
// CHECK:       b       call_through
