//@ revisions: O0_PAUTH O3_PAUTH

// FIXME: JKB: This should really be aarch64 pauthtest
//@ [O0_PAUTH] needs-llvm-components: aarch64
//@ [O0_PAUTH] compile-flags: --target=aarch64-unknown-linux-pauthtest -C opt-level=0
//@ [O3_PAUTH] needs-llvm-components: aarch64
//@ [O3_PAUTH] compile-flags: --target=aarch64-unknown-linux-pauthtest -C opt-level=3

// Make sure that direct extern "C" calls are not handled by pointer authentication operand bundle
// logic.
use std::ffi::c_void;
use std::hint::black_box;

extern "C" {
    fn rand() -> i32;
    fn add(a: i32, b: i32) -> i32;
    fn sub(a: i32, b: i32) -> i32;

    // Corresponds to: void *woof;
    static mut woof: *mut c_void;
    fn direct_function_taking_void_arg(data: *mut c_void);
    fn direct_no_arg();
    fn direct_function_taking_fp_arg(func: unsafe extern "C" fn());
}

type CFnPtr = unsafe extern "C" fn(i32, i32) -> i32;

// CHECK-LABE: test_indirect_call
#[inline(never)]
fn test_indirect_call() {
    let fp_add: CFnPtr = black_box(add);
    let fp_sub: CFnPtr = black_box(sub);

    unsafe {
        // O0_PAUTH: call {{.*}}i32 %fp_add({{.*}}) #{{[0-9]+}} [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        // O3_PAUTH: call {{.*}}i32 %fp_add({{.*}}) #{{[0-9]+}} [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        let _id1 = fp_add(7, 4);
        // O0_PAUTH: call {{.*}}i32 %fp_sub({{.*}}) #{{[0-9]+}} [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        // O3_PAUTH: call {{.*}}i32 %fp_sub({{.*}}) #{{[0-9]+}} [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        let _id2 = fp_sub(10, 6);
    }

    // Also test calling via conditional pointer
    unsafe {
        // O0_PAUTH: @rand
        // O0_PAUTH-NOT: [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        // O3_PAUTH: @rand
        // O3_PAUTH-NOT: [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        let use_add = rand() % 2 == 0;
        let fp: CFnPtr = if use_add { add } else { sub };
        // O0_PAUTH: call {{.*}}i32 %{{.*}}({{.*}}) #{{[0-9]+}} [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        // O3_PAUTH: call {{.*}}i32 %{{.*}}({{.*}}) #{{[0-9]+}} [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        let _id3 = fp(1, 2);
    }

    unsafe {
        direct_function_taking_fp_arg(direct_no_arg);
    }
}

// CHECK-LABE: test_direct_call
#[inline(never)]
fn test_direct_call() {
    unsafe {
        // O0_PAUTH: @add
        // O0_PAUTH-NOT: [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        // O3_PAUTH: @add
        // O3_PAUTH-NOT: [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        let _d1 = add(2, 3);
        // O0_PAUTH: @sub
        // O0_PAUTH-NOT: [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        // O3_PAUTH: @sub
        // O3_PAUTH-NOT: [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        let _d2 = sub(5, 1);

        // O0_PAUTH: @direct_function_taking_void_arg
        // O0_PAUTH-NOT: [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        // O3_PAUTH: direct_function_taking_void_arg
        // O3_PAUTH-NOT: [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        direct_function_taking_void_arg(woof);
    }
}

fn main() {
    test_indirect_call();
    test_direct_call();
}
