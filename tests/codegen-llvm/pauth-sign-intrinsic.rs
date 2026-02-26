//@ compile-flags: --target=aarch64-unknown-linux-pauthtest -C opt-level=3
// Check that we correctly generate @llvm.ptrauth.sign(i64 %ptr, i32 imm_key, i64 %data) intrinsic.

// FIXME: JKB: This should really be aarch64 pauthtest
//@ needs-llvm-components: aarch64

#![crate_type = "lib"]
#![feature(core_intrinsics)]

// CHECK-LABEL: @test_ptrauth_sign
// CHECK: [[PTRINT:%[0-9]+]] = ptrtoint ptr %ptr to i64
// CHECK: [[SIGNED:%[0-9]+]] = tail call i64 @llvm.ptrauth.sign(i64 [[PTRINT]], i32 1, i64 42)
// CHECK: [[RES:%[0-9]+]] = inttoptr i64 [[SIGNED]] to ptr
// CHECK: ret ptr [[RES]]
#[no_mangle]
pub unsafe fn test_ptrauth_sign(ptr: *const u8) -> *const u8 {
    core::intrinsics::ptrauth_sign(ptr, 1, 42)
}
