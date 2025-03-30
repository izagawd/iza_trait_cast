use crate::trait_registry::{get_vtable, Castable, TraitVTableRegisterer, TraitVTableRegistry, VTableError};
use crate::handy_functions::{generic_transmute, is_trait_generic};
use std::any::type_name;
use crate::handy_functions::comptime_str_eq;
use std::any::Any;
// macro, to avoid repeating code
macro_rules! cast_reference {
    ($TTo:ty, $from:expr, $from_ty:ty, $reg:expr) => {
        {

            // checks if generic input is a dyn T
            const {
                assert!(is_trait_generic::<$TTo>(),"TTo must be a dyn Trait!! eg:\n cast::<dyn Any>(&from)");
                assert!(is_trait_generic::<$from_ty>(),"Inputed type to cast must be a dyn Trait!! eg:\n cast::<dyn Any>(&from)")
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

fn cast_ref_hidden<'a,TTo: ?Sized + 'static, TFrom: Castable + ?Sized>(from: &TFrom, reg: &TraitVTableRegistry<impl TraitVTableRegisterer>) -> Result<&'a TTo, VTableError> {
    cast_reference!(TTo,from,TFrom,reg)
}


pub fn cast_ref<'a,TTo: ?Sized + 'static>(from: &(impl Castable + ?Sized), reg: &TraitVTableRegistry<impl TraitVTableRegisterer>) -> Result<&'a TTo, VTableError> {
    cast_ref_hidden(from, reg)
}

fn cast_mut_hidden<'a,TTo: ?Sized + 'static, TFrom: Castable + ?Sized>(from: &mut TFrom, reg: &TraitVTableRegistry<impl TraitVTableRegisterer>) -> Result<&'a mut TTo, VTableError> {
    cast_reference!(TTo,from,TFrom,reg)
}
pub fn cast_mut<'a,TTo: ?Sized + 'static>(from: &mut (impl Castable + ?Sized), reg: &TraitVTableRegistry<impl TraitVTableRegisterer>) -> Result<&'a mut TTo, VTableError> {
    cast_mut_hidden(from, reg)
}

