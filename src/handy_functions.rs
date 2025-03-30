use std::any::{type_name, Any};

pub(crate) unsafe fn generic_transmute<T, U>(t: T) -> U {
    const { assert!(size_of::<T>() == size_of::<U>(),"To transmute, both types must be of the same size") }; // sanity check
    let t = core::mem::ManuallyDrop::new(t);
    core::mem::transmute_copy(&t)
}
pub  const  fn is_trait_generic<T: ?Sized>() -> bool {

    let splitted_at = type_name::<T>().split_at_checked(4);
    match splitted_at {
        None => {
            false
        }
        Some(splitted_at) => {
            return  comptime_str_eq(splitted_at.0, "dyn ") && size_of::<&T>() == size_of::<&dyn Any>();
        }
    }

}
pub const fn comptime_str_eq(a: &str, b: &str) -> bool {

    if a.len() != b.len() {
        return false;
    }
    let bytes_a = a.as_bytes();
    let bytes_b = b.as_bytes();
    let mut i = 0;
    while i < a.len() {
        if bytes_a[i] != bytes_b[i] {
            return false;
        }
        i += 1;
    }
    true
}