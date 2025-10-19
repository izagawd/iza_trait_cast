use std::alloc::Allocator;
use std::fmt::{Debug, Formatter};
use std::marker::Unsize;
use std::ptr;
use crate::handy_functions::generic_transmute;
use crate::trait_registry::{get_vtable, Castable,CastError};
use std::ptr::{DynMetadata, Pointee};
use std::rc::Rc;
use std::sync::Arc;

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
pub struct CastErrorWith<T>{
    pub error: CastError,
    pub with: T
}

impl<T> CastErrorWith<T> {
    fn new(error: CastError, with: T) -> Self {
        Self { error, with }
    }
}
impl<T> Debug for CastErrorWith<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}
#[inline]
pub fn trait_cross_cast_rc<TTo: ?Sized + 'static + Pointee<Metadata=DynMetadata<TTo>>, From: Unsize<dyn Castable> + ?Sized, A: Allocator>(from: Rc<From, A>) -> Result<Rc<TTo, A>, CastErrorWith<Rc<From,A>>> {
    unsafe {
        let gotten_rc : (*const From, A) = Rc::into_raw_with_allocator(from);
        let as_ref = &*gotten_rc.0;
        let casted = trait_cross_cast_ref::<TTo>(as_ref);
        match casted {
            Ok(casted) => {
                return Ok(Rc::from_raw_in(casted, gotten_rc.1))
            }
            Err(err) => {
                Err( CastErrorWith {
                    with: Rc::from_raw_in(gotten_rc.0, gotten_rc.1),
                    error: err
                })
            }
        }
    }
}

#[inline]
pub fn trait_cross_cast_box<TTo: ?Sized + 'static + Pointee<Metadata=DynMetadata<TTo>>, From: Unsize<dyn Castable> + ?Sized, A: Allocator>(from: Box<From, A>) -> Result<Box<TTo,A>, CastErrorWith<Box<From,A>>> {
    unsafe {
        let gotten_rc: (*mut From, A) = Box::into_raw_with_allocator(from);
        let as_ref = &mut *gotten_rc.0;
        let casted = crate::cast_fns::trait_cross_cast_mut::<TTo>(as_ref);
        match casted {
            Ok(casted) => {
                return Ok(Box::from_raw_in(casted, gotten_rc.1))
            }
            Err(err) => {
                Err( CastErrorWith {
                    with: Box::from_raw_in(gotten_rc.0, gotten_rc.1),
                    error: err
                })
            }
        }
    }
}
#[inline]
pub fn trait_cross_cast_arc<TTo: ?Sized + 'static + Pointee<Metadata=DynMetadata<TTo>>, From: Unsize<dyn Castable> + ?Sized, A: Allocator>(from: Arc<From, A>) -> Result<Arc<TTo, A>, CastErrorWith<Arc<From,A>>> {
    unsafe {
        let gotten_rc :  (*const From, A)  = Arc::into_raw_with_allocator(from);
        let as_ref = &*gotten_rc.0;
        let casted = trait_cross_cast_ref::<TTo>(as_ref);
        match casted {
            Ok(casted) => {
                return Ok(Arc::from_raw_in(casted, gotten_rc.1))
            }
            Err(err) => {
                Err( CastErrorWith {
                    with: Arc::from_raw_in(gotten_rc.0, gotten_rc.1),
                    error: err
                })
            }
        }
    }
}
#[inline]
pub fn trait_cross_cast_ref<'a,TTo: ?Sized + 'static + Pointee<Metadata=DynMetadata<TTo>>>(from: &dyn Castable) -> Result<&'a TTo, CastError> {
    unsafe {
        let vtable = get_vtable::<TTo>(from);
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

#[inline]
pub fn trait_cross_cast_mut<'a,TTo: ?Sized + 'static + Pointee<Metadata=DynMetadata<TTo>>>(from: &mut dyn  Castable) -> Result<&'a mut TTo, CastError> {
    unsafe {
        let vtable = get_vtable::<TTo>(from);
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

