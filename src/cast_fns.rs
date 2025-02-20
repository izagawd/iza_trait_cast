use std::alloc::Allocator;
use std::rc::Rc;
use std::sync::Arc;
use crate::trait_registry::Castable;
use std::any::type_name;
use std::any::Any;
use crate::trait_registry::get_vtable;
use std::any::TypeId;
use crate::unsafe_fns::generic_transmute;
// macro, to avoid repeating code
macro_rules! cast_reference {
    ($TTo:ty, $from:expr) => {
        {
            assert!(type_name::<$TTo>().starts_with("dyn ") &&
                        size_of::<&$TTo>() == size_of::<&dyn Any>(),
            "TTo must be a dyn Trait!! eg:\n cast::<dyn Any>(&from);");

            let vtable =  get_vtable($from,TypeId::of::<$TTo>());
            if let Some(vtable) = vtable
            {

                let mut to_v : (& (), &'static ()) =

                unsafe {
                    // as_castable exists, to convert it into a &dyn T
                    // to better modify the vtable.
                    generic_transmute($from.as_castable())
                };

                to_v.1 = vtable;
                Some(unsafe{generic_transmute(
                    to_v
                )})
            } else{
                None
            }
        }
    };
}

pub fn cast_ref<TTo: ?Sized + 'static>(from: &(impl Castable + ?Sized)) -> Option<&TTo> {
    cast_reference!(TTo,from)
}


pub fn cast_mut<TTo: ?Sized + 'static>(from: &mut (impl Castable + ?Sized)) -> Option<&mut TTo> {
    cast_reference!(TTo,from)
}

pub fn cast_box<TTo: ?Sized + 'static,Type: Castable + ?Sized,A: Allocator>(mut from: Box<Type,A>) -> Result<Box<TTo,A>, Box<Type,A>> {
    let raw = Box::into_raw_with_allocator(from);
    // Patch the vtable on our unique pointer (we have mutable access here)
    let casted = cast_mut::<TTo>(unsafe { &mut *raw.0 });
    match casted {
        None => unsafe {
            return Err(Box::from_raw_in(raw.0,raw.1));
        }
        Some(casted) => unsafe {
            return Ok(Box::from_raw_in(casted,raw.1));
        }
    }
}

pub fn cast_rc<TTo: ?Sized + 'static,Type: Castable + ?Sized,A: Allocator>(mut from: Rc<Type,A>) -> Result<Rc<TTo,A>, Rc<Type,A>> {
    let raw = Rc::into_raw_with_allocator(from);
    // Patch the vtable on our unique pointer (we have mutable access here)
    let casted = cast_ref::<TTo>(unsafe { & *raw.0 });
    match casted {
        None => unsafe {
            return Err(Rc::from_raw_in(raw.0,raw.1));
        }
        Some(casted) => unsafe {
            return Ok(Rc::from_raw_in(casted,raw.1))
        }
    }
}

pub fn cast_arc<TTo: ?Sized + 'static,Type: Castable + ?Sized,A: Allocator>(mut from: Arc<Type,A>) -> Result<Arc<TTo,A>, Arc<Type,A>> {
    let raw = Arc::into_raw_with_allocator(from);
    // Patch the vtable on our unique pointer (we have mutable access here)
    let casted = cast_ref::<TTo>(unsafe { & *raw.0 });
    match casted {
        None => unsafe {
            return Err(Arc::from_raw_in(raw.0,raw.1));
        }
        Some(casted) => unsafe {
            return Ok(Arc::from_raw_in(casted,raw.1))
        }
    }
}