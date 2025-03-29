use crate::trait_registry::{get_vtable, Castable, TraitVTableRegisterer, TraitVTableRegistry, VTableError};
use crate::unsafe_fns::generic_transmute;
use std::any::type_name;
use std::any::Any;
// macro, to avoid repeating code
macro_rules! cast_reference {
    ($TTo:ty, $from:expr, $reg:expr) => {
        {
            assert!(type_name::<$TTo>().starts_with("dyn ") &&
                        size_of::<&$TTo>() == size_of::<&dyn Any>(),
            "TTo must be a dyn Trait!! eg:\n cast::<dyn Any>(&from);");

            let vtable =  get_vtable::<$TTo>($from,$reg);
            match vtable {
                Ok(vtable) => {
                    let mut to_v : (& (), &'static ()) =

                    unsafe {
                        // as_Any exists, to convert it into a &dyn T
                        // to better modify the vtable.
                        generic_transmute($from)
                    };

                    to_v.1 = vtable;
                    Ok(unsafe{generic_transmute(
                        to_v
                    )})
                }
                Err(err) => {
                    Err(err)
                }
            }
        }
    };
}

pub fn cast_ref<'a,TTo: ?Sized + 'static>(from: &(impl Castable + ?Sized), reg: &TraitVTableRegistry<impl TraitVTableRegisterer>) -> Result<&'a TTo, VTableError> {
    cast_reference!(TTo,from,reg)
}


pub fn cast_mut<'a,TTo: ?Sized + 'static>(from: &mut (impl Castable + ?Sized), reg: &TraitVTableRegistry<impl TraitVTableRegisterer>) -> Result<&'a mut TTo, VTableError> {
    cast_reference!(TTo,from,reg)
}

