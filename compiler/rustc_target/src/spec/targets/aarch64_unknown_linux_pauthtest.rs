use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::spec::{
    Arch, Cc, Env, FramePointer, LinkerFlavor, Lld, StackProbeType, Target, TargetMetadata,
    TargetOptions, base,
};

pub(crate) fn target() -> Target {
    let mut pre_link_args = BTreeMap::new();
    let root = std::env::var("LLVM_PAUTH").unwrap_or_else(|_| "/opt/llvm-pauth".into());
    let lib_path = format!("{}/aarch64-linux-pauthtest/usr/lib", root);
    let lib_path_static: &'static str = Box::leak(lib_path.into_boxed_str());
    pre_link_args.insert(
        LinkerFlavor::Gnu(Cc::Yes, Lld::No),
        vec![Cow::Owned(format!("-L{}", lib_path_static))],
    );

    Target {
        llvm_target: "aarch64-unknown-linux-pauthtest".into(),
        metadata: TargetMetadata {
            description: Some("ARM64 Linux with pauth enabled musl".into()),
            tier: Some(3),
            host_tools: Some(true),
            std: Some(true),
        },
        pointer_width: 64,
        data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128-Fn32".into(),
        arch: Arch::AArch64,

        options: TargetOptions {
            env: Env::Pauthtest,
            features: "+v8a,+outline-atomics".into(),
            max_atomic_width: Some(128),
            stack_probes: StackProbeType::Inline,
            crt_static_default: false,
            crt_static_respected: false,
            default_uwtable: true,
            dynamic_linking: true,
            linker: Some("aarch64-linux-pauthtest-clang".into()),
            pre_link_args,
            // the AAPCS64 expects use of non-leaf frame pointers per
            // https://github.com/ARM-software/abi-aa/blob/4492d1570eb70c8fd146623e0db65b2d241f12e7/aapcs64/aapcs64.rst#the-frame-pointer
            // and we tend to encounter interesting bugs in AArch64 unwinding code if we do not
            frame_pointer: FramePointer::NonLeaf,
            mcount: "\u{1}_mcount".into(),
            ..base::linux_musl::opts()
         },
    }
}
