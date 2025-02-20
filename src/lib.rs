#![feature(allocator_api)]

mod cast_fns;
mod trait_registry;
mod unsafe_fns;

#[cfg(test)]
mod tests {
    use crate::trait_registry::{Castable, TypeRegistration};
    use super::*;

    trait Base :  Castable{
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
    impl Castable for TestStruct{
        fn register(&self, type_registration: &mut TypeRegistration) {
            type_registration.register::<dyn Child>(self);

        }
    }
    impl Base for TestStruct{
        fn name(&self) -> &'static str {
            "TestStruct"
        }
    }
    #[test]
    fn cast_test() {
        let as_base : &dyn Base =  &TestStruct;
        if let Some(casted)= cast_fns::cast_ref::<dyn Child>(as_base){
            let fav_food= casted.favorite_food();
            assert_eq!("Chicken", fav_food, "incorrect vtable was generated");
        } else{
            assert!(false,"Casting failed despite registering the vtable for dyn Child");
        }


    }



}
