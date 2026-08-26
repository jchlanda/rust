//@ check-fail
//@ only-pauthtest
//@ needs-llvm-components: aarch64
//@ compile-flags: --target=aarch64-unknown-linux-pauthtest --crate-type=lib --emit=llvm-ir -Zpointer-authentication=+function-pointer-type-discrimination -C opt-level=0 -Cunsafe-allow-abi-mismatch=pointer-authentication

use std::mem::transmute;

pub union U {
    f: extern "C" fn(),
    f2: extern "C" fn() -> i32,
    f3: extern "C" fn() -> i128,
}

pub struct S {
    f: extern "C" fn(),
}

pub unsafe fn transmute_union_struct(u: U) -> S {
    //~^ ERROR type discrimination for function pointer authentication does not yet support transmutes of unions that contain a function pointer (U -> S)
    transmute(u)
}
