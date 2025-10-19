#![feature(allocator_api)]
#![feature(specialization)]
#![feature(ptr_metadata)]
#![feature(unsize)]
#![feature(const_trait_impl)]

pub mod trait_registry;
mod handy_functions;
pub mod cast_fns;


use crate::trait_registry::{Castable};
use std::any::Any;
#[cfg(test)]
mod tests {
    use std::any::type_name;
use std::any;
    use std::any::TypeId;
    use crate::trait_registry::{ CastError};
    use super::*;

    // Define our test traits.
    trait Base: Castable {
        fn name(&self) -> &'static str;
    }
    trait Child: Base {
        fn favorite_food(&self) -> &'static str;
    }

    // Test type that implements both Base and Child.
    struct TestStruct{
        favorite_food: &'static str,
        name: &'static str,
    }
    impl Base for TestStruct {
        fn name(&self) -> &'static str {
            self.name
        }
    }
    impl Child for TestStruct {
        fn favorite_food(&self) -> &'static str {
            self.favorite_food
        }
    }
    impl TestStruct{
        fn new() -> Self{
            TestStruct{
                favorite_food: "Chicken",
                name: "TestStruct"
            }
        }
    }
    // Test type that implements only Base.
    struct BaseOnly{
        name: &'static str,
    }
    impl Base for BaseOnly {
        fn name(&self) -> &'static str {
            self.name
        }
    }

    impl BaseOnly{
        fn new() -> Self{
            Self{
                name: "BaseOnly"
            }
        }
    }
    // Note: BaseOnly does NOT implement Child.


    register_types!{
        implementors: [TestStruct],

        traits: [Base, Child]
    }
    // Test that a valid cast returns correct results.
    #[test]
    fn vtable_validity_test() {
        let as_base: &dyn Base = &TestStruct::new();




        if let Ok(casted) = cast_fns::cast_ref::<dyn Child>(as_base) {
            assert_eq!(casted.favorite_food(), "Chicken", "incorrect vtable was generated");
            let as_base: &dyn Base = casted;
            assert_eq!(as_base.name(), "TestStruct", "incorrect vtable was generated");
        } else {
            panic!("Casting failed despite registering the vtable for dyn Child");
        }
    }

    struct UnregisteredType;

    impl Base for UnregisteredType{
        fn name(&self) -> &'static str {
            "UnregisteredType"
        }
    }

    // Test that casting without registering the type returns a CombinationNotRegistered error.
    #[test]
    fn unregistered_type_error_test() {
        let as_base: &dyn Base = &UnregisteredType;


        let result = cast_fns::cast_ref::<dyn Child>(as_base);
        match result {
            Err(CastError::CombinationNotRegistered { type_name, type_id, ..}) => {
                assert_eq!(type_name,any::type_name::<UnregisteredType>(), "Incorrect type name");
                assert_eq!(type_id, TypeId::of::<UnregisteredType>(), "Incorrect type id");

            }
            _ => assert!(false, "Did not return valid enum variant"),
        }
    }
    register_types!{
        implementors: [BaseOnly],
        traits: [Child]
    }

    // Test that casting to a trait that the type does not implement returns a TraitNotImplemented error.
    #[test]
    fn trait_not_implemented_error_test() {
        let as_base: &dyn Base = &BaseOnly::new();


        let result = cast_fns::cast_ref::<dyn Child>(as_base);
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
        let mut test_instance = TestStruct::new();
        let as_base: &mut dyn Base = &mut test_instance;

        let result = cast_fns::cast_mut::<dyn Child>(as_base);
        match result {
            Ok(child) => {
                assert_eq!(child.favorite_food(), "Chicken", "Mutable cast did not produce correct behavior");
            }
            Err(e) => panic!("Mutable casting failed with error: {:?}", e),
        }
    }

    // testing that it gives a CombinationNotRegistered error when checking if a type implements a trait, and that fact has not been registered
    #[test]
    fn trait_not_registered_test() {
        // Define a trait that we will not register.
        trait UnregisteredTrait: Castable {
            fn do_something(&self) -> &'static str;
        }
        // A type that implements UnregisteredTrait.
        struct RegisteredStruct;
        impl UnregisteredTrait for RegisteredStruct {
            fn do_something(&self) -> &'static str {
                "nothing"
            }
        }
        register_types!(

            implementors: [RegisteredStruct],
            traits: []
        );
        // Create an instance and a registry using our EmptyRegisterer.
        let as_trait: &dyn UnregisteredTrait = &RegisteredStruct;


        // Attempt to cast to UnregisteredTrait, expecting an error.
        let result = cast_fns::cast_ref::<dyn UnregisteredTrait>(as_trait);
        match result {
            Err(CastError::CombinationNotRegistered { trait_name, trait_id: trait_type_id, .. }) => {
                assert_eq!(trait_name, type_name::<dyn UnregisteredTrait>(), "Error: Trait name did not match");
                assert_eq!(trait_type_id, TypeId::of::<dyn UnregisteredTrait>(), "Error: trait type  did not match");
            },

            _ => panic!("Expected a CombinationNotRegistered error because the trait was not registered"),
        }
    }
}