#![feature(allocator_api)]
#![feature(specialization)]
#![feature(ptr_metadata)]
#![feature(allow_internal_unstable)]
#![feature(const_type_name)]
#![feature(type_alias_impl_trait)]
#![feature(unsize)]
pub mod trait_registry;
mod handy_functions;
pub mod cast_fns;


use crate::trait_registry::{Castable, TypeVTableMapper};


use crate::trait_registry::{TraitVTableRegisterer, TraitVTableRegistry};
use std::any::Any;
#[cfg(test)]
mod tests {
    use std::any::type_name;
use std::any;
    use std::any::TypeId;
    use crate::trait_registry::{RegistererHelper, CastError};
    use super::*;

    // Define our test traits.
    trait Base: Castable {
        fn name(&self) -> &'static str;
    }
    trait Child: Base {
        fn favorite_food(&self) -> &'static str;
    }

    // Test type that implements both Base and Child.
    struct TestStruct;
    impl Base for TestStruct {
        fn name(&self) -> &'static str {
            "TestStruct"
        }
    }
    impl Child for TestStruct {
        fn favorite_food(&self) -> &'static str {
            "Chicken"
        }
    }

    // Test type that implements only Base.
    struct BaseOnly;
    impl Base for BaseOnly {
        fn name(&self) -> &'static str {
            "BaseOnly"
        }
    }
    // Note: BaseOnly does NOT implement Child.

    #[derive(Default)]
    struct TestRegisterer;
    impl TraitVTableRegisterer for TestRegisterer {
        fn register_trait_vtables_for_type<T: Any>(&self, registry: &mut RegistererHelper<T>) {
            registry.register_trait_for_type::<dyn Base>();
            registry.register_trait_for_type::<dyn Child>();
        }
    }

    // Test that a valid cast returns correct results.
    #[test]
    fn vtable_validity_test() {
        let as_base: &dyn Base = &TestStruct;
        let mut vtable_holder = TraitVTableRegistry::<TestRegisterer>::default();
        vtable_holder.register_type::<TestStruct>();

        if let Ok(casted) = cast_fns::cast_ref::<dyn Child>(as_base, &vtable_holder) {
            assert_eq!(casted.favorite_food(), "Chicken", "incorrect vtable was generated");
            let as_base: &dyn Base = casted;
            assert_eq!(as_base.name(), "TestStruct", "incorrect vtable was generated");
        } else {
            panic!("Casting failed despite registering the vtable for dyn Child");
        }
    }
    #[test]
    fn registered_type_checker(){
        let mut vtable_holder = TraitVTableRegistry::<TestRegisterer>::default();
        vtable_holder.register_type::<TestStruct>();
        assert!(vtable_holder.is_type_registered(&TypeId::of::<TestStruct>()),"Type not properly registered, or method that checks if a type is registered is not working correctly");
        assert!(!vtable_holder.is_type_registered(&TypeId::of::<i32>()),"Says the type i32 is registered, despitemthe fact that it is not");
    }
    // Test that casting without registering the type returns a TypeNotRegistered error.
    #[test]
    fn unregistered_type_error_test() {
        let as_base: &dyn Base = &TestStruct;
        // Note: We do not call register_type::<TestStruct>()
        let vtable_holder = TraitVTableRegistry::<TestRegisterer>::default();

        let result = cast_fns::cast_ref::<dyn Child>(as_base, &vtable_holder);
        match result {
            Err(CastError::TypeNotRegistered { type_name, type_id}) => {
                assert_eq!(type_name,any::type_name::<TestStruct>(), "Incorrect type name");
                assert_eq!(type_id, TypeId::of::<TestStruct>(), "Incorrect type id");

            }
            _ => assert!(false, "Did not return valid enum variant"),
        }
    }


    // Test that casting to a trait that the type does not implement returns a TraitNotImplemented error.
    #[test]
    fn trait_not_implemented_error_test() {
        let as_base: &dyn Base = &BaseOnly;
        let mut vtable_holder = TraitVTableRegistry::<TestRegisterer>::default();
        vtable_holder.register_type::<BaseOnly>();

        let result = cast_fns::cast_ref::<dyn Child>(as_base, &vtable_holder);
        match result {
            Err(CastError::TraitNotImplemented { trait_name, trait_id: trait_type_id, type_name, type_id }) => {
                assert_eq!(type_name,any::type_name::<BaseOnly>(), "Incorrect type name");
                assert_eq!(type_id, TypeId::of::<BaseOnly>(), "Incorrect type id");
                assert_eq!(any::type_name::<dyn Child>(), trait_name, "Invalid trait name");
                assert_eq!(TypeId::of::<dyn Child>(), trait_type_id, "Incorrect trait id");
            }
            _ => panic!("Expected a TraitNotImplemented error"),
        }
    }

    //  Testing the mutable casting functionality.
    #[test]
    fn mutable_cast_validity_test() {
        let mut test_instance = TestStruct;
        let as_base: &mut dyn Base = &mut test_instance;
        let mut vtable_holder = TraitVTableRegistry::<TestRegisterer>::default();
        vtable_holder.register_type::<TestStruct>();

        let result = cast_fns::cast_mut::<dyn Child>(as_base, &vtable_holder);
        match result {
            Ok(child) => {
                assert_eq!(child.favorite_food(), "Chicken", "Mutable cast did not produce correct behavior");
            }
            Err(e) => panic!("Mutable casting failed with error: {:?}", e),
        }
    }
    #[test]
    fn trait_not_registered_test() {
        // Define a trait that we will not register.
        trait UnregisteredTrait: Castable {
            fn do_something(&self) -> &'static str;
        }
        // A type that implements UnregisteredTrait.
        struct UnregisteredStruct;
        impl UnregisteredTrait for UnregisteredStruct {
            fn do_something(&self) -> &'static str {
                "nothing"
            }
        }

        // Create a registerer that does not register UnregisteredTrait.
        #[derive(Default)]
        struct EmptyRegisterer;
        impl TraitVTableRegisterer for EmptyRegisterer {
            fn register_trait_vtables_for_type<T: Any>(&self, registry: &mut RegistererHelper<T>) {
                // Intentionally do nothing.
            }
        }

        // Create an instance and a registry using our EmptyRegisterer.
        let as_trait: &dyn UnregisteredTrait = &UnregisteredStruct;
        let mut registry = TraitVTableRegistry::<EmptyRegisterer>::default();
        registry.register_type::<UnregisteredStruct>();

        // Attempt to cast to UnregisteredTrait, expecting an error.
        let result = cast_fns::cast_ref::<dyn UnregisteredTrait>(as_trait, &registry);
        match result {
            Err(CastError::TraitNotRegistered { trait_name, trait_id: trait_type_id }) => {
                assert_eq!(trait_name, type_name::<dyn UnregisteredTrait>(), "Error: Trait name did not match");
                assert_eq!(trait_type_id, TypeId::of::<dyn UnregisteredTrait>(), "Error: trait type  did not match");
            },
            _ => panic!("Expected a TraitNotRegistered error because the trait was not registered"),
        }
    }
}