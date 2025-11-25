// revisions: O0_PAUTH O3_PAUTH O0_NO_PAUTH O3_NO_PAUTH
// [O0_PAUTH] compile-flags: -Z pauth -C opt-level=0
// [O3_PAUTH] compile-flags: -Z pauth -C opt-level=3
// [O0_NO_PAUTH] compile-flags: -C opt-level=0
// [O3_NO_PAUTH] compile-flags: -C opt-level=3

type FnPtr = unsafe extern "C" fn(i32, i32) -> i32;

// O0_NO_PAUTH-NOT: "ptrauth"(i32 0, i64 0)
// O3_NO_PAUTH-NOT: "ptrauth"(i32 0, i64 0)

// CHECK-LABEL: @_ZN14pauth_extern_c4main
fn main() {
    let add_ptr: FnPtr = add_from_c;
    // O0_PAUTH: call i32 @_ZN14pauth_extern_c7call_it{{.*}}(ptr ptrauth (ptr \
    // @add_from_c, i32 0), i32 5, i32 7)
    let _sum = call_it(add_ptr, 5, 7);
    assert!(12 == _sum);
}

// CHECK-LABEL: @_ZN14pauth_extern_c7call_it
#[inline(never)]
fn call_it(fn_ptr: FnPtr, arg_1: i32, arg_2: i32) -> i32 {
    // O0_PAUTH: call i32 %fn_ptr(i32 %arg_1, i32 %arg_2) {{.*}} [ "ptrauth"(i3\
    // 2 0, i64 0) ]
    // O3_PAUTH: tail call noundef i32 ptrauth (ptr @add_from_c, i32 0)(i32 nou\
    // ndef 5, i32 noundef 7) {{.*}} [ "ptrauth"(i32 0, i64 0) ]
    unsafe { fn_ptr(arg_1, arg_2) }
}

// CHECK: declare noundef i32 @add_from_c(i32 noundef, i32 noundef)
extern "C" {
    fn add_from_c(a: i32, b: i32) -> i32;
}
