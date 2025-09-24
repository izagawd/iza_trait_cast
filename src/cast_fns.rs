use crate::handy_functions::generic_transmute;
use crate::trait_registry::{get_vtable, Castable, TraitVTableRegisterer, TraitVTableRegistry, CastError};
use std::ptr::{DynMetadata, Pointee};

// macro, to avoid repeating code
macro_rules! cast_reference {
    ($TTo:ty, $from:expr,  $reg:expr) => {
        {
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



pub fn cast_ref<'a,TTo: ?Sized + 'static + Pointee<Metadata=DynMetadata<TTo>>>(from: &dyn Castable, reg: &TraitVTableRegistry<impl TraitVTableRegisterer>) -> Result<&'a TTo, CastError> {
    cast_reference!(TTo,from,reg)
}


pub fn cast_mut<'a,TTo: ?Sized + 'static + Pointee<Metadata=DynMetadata<TTo>>>(from: &mut dyn  Castable, reg: &TraitVTableRegistry<impl TraitVTableRegisterer>) -> Result<&'a mut TTo, CastError> {
    cast_reference!(TTo,from,reg)
}

