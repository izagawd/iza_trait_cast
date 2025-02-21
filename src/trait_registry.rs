use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};
use crate::unsafe_fns::generic_transmute;

/// Stores all vtables for all types
static TYPE_REGISTRY : LazyLock<RwLock<HashMap<TypeId, TypeRegistration>>> =
    LazyLock::new(Default::default);

pub(crate) trait AsCastable{
    fn as_castable(&self) -> &dyn Castable;
}
impl<T: Castable> AsCastable for T{
    fn as_castable(&self) -> &dyn Castable{
        self
    }
}
pub trait Castable  : Any + AsCastable{
    fn register(&self, type_registration: &mut TypeRegistration){}
}



pub(crate) fn get_vtable(input: &(impl Castable + ?Sized), trait_type_id: TypeId) -> Option<&'static ()>{
    let obj_type_id = input.type_id();
    let temp_val =TYPE_REGISTRY.read().unwrap();
    let type_registration_maybe = temp_val.get(&obj_type_id);
    let mut type_registration;
    match type_registration_maybe {
        None => {
            drop(temp_val);
            let  mut temp_registration = TypeRegistration::new(obj_type_id);


            input.register(&mut temp_registration);
            let mut unwrapped =TYPE_REGISTRY.write().unwrap();
            unwrapped.insert(obj_type_id, temp_registration);
            type_registration =  unwrapped.get(&obj_type_id).unwrap();

            return Some(*(type_registration.vtables.get(&trait_type_id)?))
        },
        Some(gotten) => {
            type_registration = gotten;
        }
    }
    return Some(*(type_registration.vtables.get(&trait_type_id)?))
}
/// Used to register vtables for dyn Types
pub struct TypeRegistration{
    concrete_type_id: TypeId,
    vtables: HashMap<TypeId, &'static ()>
}

impl TypeRegistration {
    pub fn register<'a,TCastTo: Castable + ?Sized + 'static>
    (&mut self, object_to_register_for: &'a TCastTo)
    where &'a TCastTo: Into<&'a TCastTo>{
        assert!(object_to_register_for.type_id() == self.concrete_type_id,
        "Inputted object's TypeId does not match the expected type");
        let split : (& (), &'static ()) = unsafe{
            generic_transmute(object_to_register_for)
        };
        self.vtables.insert(TypeId::of::<TCastTo>(), split.1);
    }
    fn new(concrete_type_id: TypeId) -> TypeRegistration {
        TypeRegistration{concrete_type_id, vtables: HashMap::new()}
    }
}
