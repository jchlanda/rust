use std::cell::RefCell;

use rustc_data_structures::fx::FxHashMap;
use rustc_middle::ty::{self, Instance, Ty};
use rustc_session::Session;

use super::BackendTypes;

pub enum AddressDiversity {
    None,
    Real,           // Use actual address
    Synthetic(u64), // Use a fixed, synthetic value (i.e. 1 for init/fini)
}

pub struct PacMetadata {
    pub key: u32,
    pub disc: u64,
    pub addr_diversity: AddressDiversity,
}

pub trait MiscCodegenMethods<'tcx>: BackendTypes {
    fn vtables(
        &self,
    ) -> &RefCell<FxHashMap<(Ty<'tcx>, Option<ty::ExistentialTraitRef<'tcx>>), Self::Value>>;
    fn apply_vcall_visibility_metadata(
        &self,
        _ty: Ty<'tcx>,
        _poly_trait_ref: Option<ty::ExistentialTraitRef<'tcx>>,
        _vtable: Self::Value,
    ) {
    }
    fn get_fn(&self, instance: Instance<'tcx>) -> Self::Function;
    fn get_fn_addr(&self, instance: Instance<'tcx>, pac: Option<PacMetadata>) -> Self::Value;
    fn eh_personality(&self) -> Self::Function;
    fn sess(&self) -> &Session;
    fn set_frame_pointer_type(&self, llfn: Self::Function);
    fn apply_target_cpu_attr(&self, llfn: Self::Function);
    /// Declares the extern "C" main function for the entry point. Returns None if the symbol
    /// already exists.
    fn declare_c_main(&self, fn_type: Self::FunctionSignature) -> Option<Self::Function>;
}
