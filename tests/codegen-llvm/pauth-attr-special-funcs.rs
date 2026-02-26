//@ compile-flags: --target=aarch64-unknown-linux-pauthtest -C opt-level=0
// Make sure that compiler generated functions (main wrapper and __rust_try) also have ptrauth
// attributes set correctly. Rustc only generates __rust_try at O0, so use that opt level for the
// test.

// FIXME: JKB: This should really be aarch64 pauthtest
//@ needs-llvm-components: aarch64

use std::panic;

// CHECK: define {{.*}} @__rust_try{{.*}} [[ATTR_TRY:#[0-9]+]]
// CHECK: define {{.*}} @main{{.*}} [[ATTR_MAIN:#[0-9]+]]

// CHECK: attributes [[ATTR_TRY]] = { {{.*}} "ptrauth-auth-traps" "ptrauth-calls" "ptrauth-returns"
// CHECK: attributes [[ATTR_MAIN]] = { {{.*}} "ptrauth-auth-traps" "ptrauth-calls" "ptrauth-returns"
fn main() {
    let _ = panic::catch_unwind(|| {
        panic!("BOOM");
    });
}
