//@ revisions: O0_PAUTH O3_PAUTH

//@ [O0_PAUTH] compile-flags: --target=aarch64-unknown-linux-pauthtest -Z pauth -C opt-level=0
//@ [O3_PAUTH] compile-flags: --target=aarch64-unknown-linux-pauthtest -Z pauth -C opt-level=3

// Make sure that direct extern "C" calls are not handled by pointer authentication logic.
use std::hint::black_box;

extern "C" {
    fn rand() -> i32;
    fn add(a: i32, b: i32) -> i32;
    fn sub(a: i32, b: i32) -> i32;
}

type CFnPtr = unsafe extern "C" fn(i32, i32) -> i32;

// CHECK-LABE: test_indirect_call
#[inline(never)]
fn test_indirect_call() {
    let fp_add: CFnPtr = black_box(add);
    let fp_sub: CFnPtr = black_box(sub);

    unsafe {
        // O3_PAUTH: call {{.*}}i32 %fp_add({{.*}}) #{{[0-9]+}} [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        let _id1 = fp_add(7, 4);
        // O3_PAUTH: call {{.*}}i32 %fp_sub({{.*}}) #{{[0-9]+}} [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        let _id2 = fp_sub(10, 6);
    }

    // Also test calling via conditional pointer
    unsafe {
        let use_add = rand() % 2 == 0;
        let fp: CFnPtr = if use_add { add } else { sub };
        // O3_PAUTH: call {{.*}}i32 %add.sub({{.*}}) #{{[0-9]+}} [ "ptrauth"(i32{{.*}}, i64{{.*}}) ]
        let _id3 = fp(1, 2);
    }
}

// CHECK-LABE: test_direct_call
#[inline(never)]
fn test_direct_call() {
    unsafe {
        // O0_PAUTH: call i32 @add(
        // O0_PAUTH-NOT: "ptrauth"(
        // O3_PAUTH: call {{.*}}i32 @add(
        // O3_PAUTH-NOT: "ptrauth"(
        let _d1 = add(2, 3);
        // O0_PAUTH: call i32 @sub(
        // O0_PAUTH-NOT: "ptrauth"(
        // O3_PAUTH: call {{.*}}i32 @sub(
        // O3_PAUTH-NOT: "ptrauth"(
        let _d2 = sub(5, 1);
    }
}


fn main() {
    test_indirect_call();
    test_direct_call();
}
