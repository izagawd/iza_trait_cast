use std::ptr;
use crate::handy_functions::generic_transmute;
use crate::trait_registry::{get_vtable, Castable,TraitVTableRegistry, CastError};
use std::ptr::{DynMetadata, Pointee};

// macro, to avoid repeating code
macro_rules! cast_reference {
    ($TTo:ty, $from:expr,  $reg:expr) => {
        {
            unsafe{
                let vtable =  get_vtable::<$TTo>($from,$reg);
                match vtable {
                    Ok(vtable) => {
                        let gotten : *const $TTo = ptr::from_raw_parts(ptr::addr_of!($from),generic_transmute(vtable));
                        Ok(&*gotten)
                    }
                    Err(err) => {
                        Err(err)
                    }
                }
            }

        }
    };
}




pub fn cast_ref<'a,TTo: ?Sized + 'static + Pointee<Metadata=DynMetadata<TTo>>>(from: &dyn Castable, reg: &TraitVTableRegistry) -> Result<&'a TTo, CastError> {
    unsafe {
        let vtable = get_vtable::<TTo>(from, reg);
        match vtable {
            Ok(vtable) => {
                let gotten: *const TTo = ptr::from_raw_parts(from as *const dyn Castable as *const (), generic_transmute(vtable));
                Ok(&*gotten)
            }
            Err(err) => {
                Err(err)
            }
        }
    }
}


pub fn cast_mut<'a,TTo: ?Sized + 'static + Pointee<Metadata=DynMetadata<TTo>>>(from: &mut dyn  Castable, reg: &TraitVTableRegistry) -> Result<&'a mut TTo, CastError> {
    unsafe {
        let vtable = get_vtable::<TTo>(from, reg);
        match vtable {
            Ok(vtable) => {
                let gotten: *mut TTo = ptr::from_raw_parts_mut(from as *mut dyn Castable as *mut (), generic_transmute(vtable));
                Ok(&mut *gotten)
            }
            Err(err) => {
                Err(err)
            }
        }
    }
}

