use std::any::{type_name, Any, TypeId};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::marker::{PhantomData, Unsize};
use std::mem::transmute;
use std::ptr::{metadata, null, DynMetadata, Pointee};
use std::sync::LazyLock;
use inventory::collect;





pub enum CastError {
    TraitNotImplemented{
        trait_name: &'static str,
        trait_id: TypeId,
        type_name: &'static str,
        type_id: TypeId,
    },
    CombinationNotRegistered {
        trait_name: &'static str,
        trait_id: TypeId,
        type_name: &'static str,
        type_id: TypeId,
    },


}
impl Debug for CastError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TraitNotImplemented{trait_name, type_name,.. } => {
                f.write_fmt(format_args!("trait '{trait_name}' not implemented by the underlying concrete type '{type_name}'"))
            },
            Self::CombinationNotRegistered{trait_name, type_name,.. } => {
                f.write_fmt(format_args!("trait '{trait_name}' has not been registered to check if it is implemented by the underlying concrete type '{type_name}'"))
            },
        }

    }
}
#[derive(Clone,Copy)]
pub struct VTable(&'static ());
pub struct VTableMapInstance{
    implementor_type_id: ImplementorTypeId,
    trait_type_id: TraitTypeId,
    v_table: Option<VTable>
}

impl VTableMapInstance {
    pub const fn new(    implementor_type_id: ImplementorTypeId,
    trait_type_id: TraitTypeId,
    v_table: Option<VTable>) -> Self{
        Self{implementor_type_id, trait_type_id, v_table}
    }
}
collect!(VTableMapInstance);
pub trait Castable: Any{
    fn type_name(&self) -> &'static str;
}
impl<T: Any> Castable for T{
    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }
}
type  ImplementorTypeId = TypeId;
type  TraitTypeId = TypeId;

    static VTABLE_REGISTRY: LazyLock<HashMap<ImplementorTypeId, HashMap<TraitTypeId,Option<VTable>>>> = LazyLock::new(||{
        let mut za_hash = HashMap::new();
        for i in inventory::iter::<VTableMapInstance> {
            za_hash.insert(i.implementor_type_id, HashMap::new());
        }
        for i in inventory::iter::<VTableMapInstance> {
            let gotten = za_hash.get_mut(&i.implementor_type_id).unwrap();
            gotten.insert(i.trait_type_id,i.v_table);
        }
        za_hash
    });



/// Gets the vtable
pub(crate) fn get_vtable<TCastTo: ?Sized + 'static + Pointee<Metadata=DynMetadata<TCastTo>>>(obj: &(impl Castable + ?Sized)) -> Result<VTable, CastError>{
    let obj_type_id = obj.type_id();
    let type_registration_maybe = VTABLE_REGISTRY.get(&obj_type_id);

    match type_registration_maybe{
        Some(type_registration) => {
            match type_registration.get(&TypeId::of::<TCastTo>()) {
                None => {
                    Err(CastError::CombinationNotRegistered{trait_name: type_name::<TCastTo>(), trait_id: TypeId::of::<TCastTo>(), type_name: obj.type_name(), type_id: obj_type_id })
                }
                Some(gotten) => {
                    match gotten {
                        None => {
                            Err(CastError::TraitNotImplemented {trait_name: type_name::<TCastTo>(), trait_id: TypeId::of::<TCastTo>(), type_name: obj.type_name(), type_id: obj_type_id })
                        } Some(found) => {
                            Ok(*found)
                        }
                    }
                }
            }
        }
        None => {
            Err(CastError::CombinationNotRegistered {trait_name: type_name::<TCastTo>(), trait_id: TypeId::of::<TCastTo>(), type_name: obj.type_name(), type_id: obj_type_id })
        }
    }

}
pub const fn generate_trait_vtable<Type: 'static,Trait: ?Sized + Pointee<Metadata=DynMetadata<Trait>> + 'static>() -> Option<VTable> {
    struct AsDyn<Type: 'static> {
        kk: PhantomData<fn() -> Type>,
    }
    const trait AsDynImpl<Trait: ?Sized + Pointee<Metadata=DynMetadata<Trait>>>{
        type ToReg : Sized;
        fn vtable_getter() -> Option<VTable>;
    }
    impl<Trait: ?Sized + Pointee<Metadata=DynMetadata<Trait>> + 'static, Type: 'static> const AsDynImpl<Trait> for AsDyn<Type> {
        type ToReg = Type;
        default fn vtable_getter() -> Option<VTable>{
            None
        }
    }
    impl<Type: Unsize<Trait> + 'static,Trait: ?Sized + Pointee<Metadata=DynMetadata<Trait>> + 'static> const AsDynImpl<Trait> for AsDyn<Type> {
        fn vtable_getter()  -> Option<VTable>{
            unsafe{  Some(transmute(metadata(null::<Type>() as *const Trait))) }
        }
    }

    <AsDyn<Type> as AsDynImpl<Trait>>::vtable_getter()
}
struct INVALID;
#[macro_export]
macro_rules! register_types {
    // Entry: two comma-separated lists (trailing commas ok)
    (implementors: [$($impl:ty),* $(,)?], traits: [$($tr:path),* $(,)?]) => {
        // Recur over implementors
        $crate::register_types!(@impls [$($impl),*] @traits [$($tr),*]);
    };

    // Consume one implementor, keep the full traits list intact
    (@impls [$head:ty $(, $tail:ty)*] @traits [$($tr:path),*]) => {
        $crate::register_types!(@for_one_impl $head; [$($tr),*]);
        $crate::register_types!(@impls [$($tail),*] @traits [$($tr),*]);
    };
    // Done with implementors
    (@impls [] @traits [$($tr:path),*]) => {};

    // For a single implementor, munch the traits list one by one
    (@for_one_impl $impl:ty; [$first:path $(, $rest:path)*]) => {
        $crate::register_types!(@emit $impl, $first);
        $crate::register_types!(@for_one_impl $impl; [$($rest),*]);
    };
    // Done with traits for this implementor
    (@for_one_impl $impl:ty; []) => {};

    // The actual submission
    (@emit $impl:ty, $tr:path) => {
        inventory::submit! {

            $crate::trait_registry::VTableMapInstance::new(::core::any::TypeId::of::<$impl>(), ::core::any::TypeId::of::<dyn $tr>(),$crate::trait_registry::generate_trait_vtable::<$impl,dyn $tr>())
        }
        // Optional: enforce `$impl: $tr` at compile time
        // const _: fn() = || { fn _assert<T: $tr>() {} _assert::<$impl>(); };
    };
}