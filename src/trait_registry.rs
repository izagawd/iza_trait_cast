use std::any::{type_name, type_name_of_val, Any, TypeId};
use std::collections::HashMap;
use std::ptr::addr_eq;
use std::sync::{LazyLock, RwLock};

static TYPE_REGISTRY : LazyLock<RwLock<HashMap<TypeId, TypeRegistration>>> =
    LazyLock::new(Default::default);


pub trait Castable  : Any{
    fn register(&self, type_registration: &mut TypeRegistration){}
}

 unsafe fn generic_transmute<T, U>(t: T) -> U {
    const { assert!(size_of::<T>() == size_of::<U>(),"To transmute, both types must be of the same size") }; // sanity check
    let t = core::mem::ManuallyDrop::new(t);
    core::mem::transmute_copy(&t)
}

fn get_vtable(input: &(impl Castable + ?Sized), trait_type_id: TypeId) -> Option<&'static ()>{
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

pub struct TypeRegistration{
    concrete_type_id: TypeId,
    vtables: HashMap<TypeId, &'static ()>
}

impl TypeRegistration{

}

impl TypeRegistration {
    pub fn register<'a,TCastTo: Any + ?Sized + 'static>
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
macro_rules! cast {
    () => {};
}
pub(crate) fn cast_ref<TTo: ?Sized + 'static>(from: &dyn Castable) -> Option<&TTo> {
    assert!(type_name::<TTo>().starts_with("dyn ") &&
    size_of::<&TTo>() == size_of::<&dyn Any>(),
            "TTo must be a dyn Trait!! eg:\n cast::<dyn Any>(&from);");

    let vtable =  get_vtable(from,TypeId::of::<TTo>());
    if let Some(vtable) = vtable
    {
        let mut to_v : (& (), &'static ()) = unsafe { generic_transmute(from) };
        to_v.1 = vtable;
        Some(unsafe{generic_transmute(
            to_v
        )})
    } else{
        None
    }
}
pub(crate) fn cast_<TTo: ?Sized + 'static>(from: &dyn Castable) -> Option<&TTo> {
    assert!(type_name::<TTo>().starts_with("dyn ") &&
                size_of::<&TTo>() == size_of::<&dyn Any>(),
            "TTo must be a dyn Trait!! eg:\n cast::<dyn Any>(&from);");

    let vtable =  get_vtable(from,TypeId::of::<TTo>());
    if let Some(vtable) = vtable
    {

        let mut to_v : (& (), &'static ()) = unsafe { generic_transmute(from) };
        to_v.1 = vtable;
        Some(unsafe{generic_transmute(
            to_v
        )})
    } else{
        None
    }
}
