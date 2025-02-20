pub(crate) unsafe fn generic_transmute<T, U>(t: T) -> U {
    const { assert!(size_of::<T>() == size_of::<U>(),"To transmute, both types must be of the same size") }; // sanity check
    let t = core::mem::ManuallyDrop::new(t);
    core::mem::transmute_copy(&t)
}
