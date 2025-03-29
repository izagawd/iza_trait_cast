#![feature(allocator_api)]
#![allow(warnings)]
#![feature(specialization)]
#![feature(ptr_metadata)]
#![feature(allow_internal_unstable)]
pub mod trait_registry;
mod unsafe_fns;
pub mod cast_fns;


#[cfg(test)]
mod tests {
    use super::*;
    use crate::trait_registry::{TraitVTableRegisterer, TraitVTableRegistry, TypeVTableMapper};
    use std::any::Any;
    use std::ops::Deref;
    use std::ptr;
    use std::ptr::DynMetadata;

    trait Base :  Any{
        fn name(&self) -> &'static str;

    }
    trait Child : Base{
        fn favorite_food(&self) -> &'static str;
    }
    struct TestStruct;

    impl Child for TestStruct{
        fn favorite_food(&self) -> &'static str {
            "Chicken"
        }
    }

    impl Base for TestStruct{
        fn name(&self) -> &'static str {
            "TestStruct"
        }
    }
    #[derive(Default)]
    struct TestRegisterer;
    impl TraitVTableRegisterer for TestRegisterer{
        fn register_trait_vtables_for_type<T: Any>(&self,  registry: &mut TypeVTableMapper) {
            register_trait_for_type!(dyn Base, T,  registry);
            register_trait_for_type!(dyn Child, T, registry);
        }
    }
    #[test]
    fn cast_test() {
        let gotten : DynMetadata<dyn Any> = ptr::metadata(ptr::null::<u32>() as *const dyn Any);
        let as_base : &dyn Base =  &TestStruct;
        let mut vtable_holder = TraitVTableRegistry::<TestRegisterer>::default();

        vtable_holder.register_type::<TestStruct>();
        if let Ok(casted)= cast_fns::cast_ref::<dyn Child>(as_base, &vtable_holder){
            let fav_food= casted.favorite_food();
            let as_base : &dyn Base = casted;
            let name = as_base.name();
            assert_eq!("Chicken", fav_food, "incorrect vtable was generated");
            assert_eq!("TestStruct", name, "incorrect vtable was generated");
        } else{
            assert!(false,"Casting failed despite registering the vtable for dyn Child");
        }
    }

}
