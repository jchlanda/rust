//@ check-fail
//@ only-pauthtest
//@ needs-llvm-components: aarch64
//@ compile-flags: --target=aarch64-unknown-linux-pauthtest --crate-type=lib --emit=llvm-ir -Zpointer-authentication=+function-pointer-type-discrimination -C opt-level=0 -Cunsafe-allow-abi-mismatch=pointer-authentication

use std::mem::transmute;

#[repr(C)]
pub enum E {
    A(extern "C" fn()),
    B(i64),
}

#[repr(C)]
pub enum E2 {
    C(extern "C" fn() -> i32),
    D(i64),
}

pub unsafe fn transmute_e(e: E) -> E2 {
    //~^ ERROR type discrimination for function pointer authentication does not yet support transmutes of multi-variant enums containing function pointers (E -> E2)
    transmute(e)
}
