use crate::trait_registry::{get_vtable, Castable, TraitVTableRegisterer, TraitVTableRegistry, VTableError};
use crate::handy_functions::{generic_transmute, is_trait_generic};
use std::any::type_name;
use crate::handy_functions::comptime_str_eq;
use std::any::Any;
// macro, to avoid repeating code
macro_rules! cast_reference {
    ($TTo:ty, $from:expr,  $reg:expr) => {
        {

            // checks if generic input is a dyn T
            const {
                assert!(is_trait_generic::<$TTo>(),"TTo must be a dyn Trait!! eg:\n cast::<dyn Any>(&from)");

            }

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



pub fn cast_ref<'a,TTo: ?Sized + 'static>(from: &dyn Castable, reg: &TraitVTableRegistry<impl TraitVTableRegisterer>) -> Result<&'a TTo, VTableError> {
    cast_reference!(TTo,from,reg)
}


pub fn cast_mut<'a,TTo: ?Sized + 'static>(from: &mut dyn  Castable, reg: &TraitVTableRegistry<impl TraitVTableRegisterer>) -> Result<&'a mut TTo, VTableError> {
    cast_reference!(TTo,from,reg)
}

