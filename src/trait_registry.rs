use std::any::{Any, TypeId};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem::transmute;
use std::ptr::DynMetadata;

/// A container for storing and mapping vtables of all registered trait implementations for a concrete type.
///
/// This holder associates a concrete type with the vtables of each trait it implements. It uses a
/// mapping from each trait’s `TypeId` to a corresponding [`VTableContainer`], which in turn stores the
/// vtable pointer for that trait implementation. The generic parameter `T` must implement
/// [`TraitVTableRegisterer`], which provides the mechanism for registering trait vtables for the type.
///
/// The [`VTableContainer`] not only holds the vtable pointers but also verifies that the registered
/// object matches the expected concrete type before insertion.
pub struct TraitVTableRegistry<T: TraitVTableRegisterer>{
    registerer: T,
    trait_registration_mapper: HashMap<TypeId, TypeVTableMapper>
}
impl<T: Default + TraitVTableRegisterer> Default for TraitVTableRegistry<T> {
    fn default() -> Self {
        Self{
            registerer: T::default(),
            trait_registration_mapper: HashMap::new()
        }
    }
}
impl<TraitReg: TraitVTableRegisterer> TraitVTableRegistry<TraitReg> {
    /// Used to include a type for casting, so u can cast that type to another trait even when the compiler does not know the underlying type
    pub fn register_type<T: Any>(&mut self) {
        let type_id = TypeId::of::<T>();
        let type_registration = match self.trait_registration_mapper.entry(type_id) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(TypeVTableMapper::new(type_id)),
        };
        self.registerer.register_trait_vtables_for_type::<T>(type_registration);
    }
}



/// ## Example
///
/// ```
/// use std::any::Any;
/// use iza_trait_cast::register_trait_for_type;///
///
///
/// use iza_trait_cast::trait_registry::{TraitVTableRegisterer, TypeVTableMapper};
///
/// struct MyRegisterer;
///
/// trait Other : Any{}
///
/// impl TraitVTableRegisterer for MyRegisterer{
///     fn register_trait_vtables_for_type<T: Any>(&self, registry: &mut TypeVTableMapper) {
///        register_trait_for_type!(dyn Other, T, registry);
///        // now if type T implements dyn Other, it will be able to be casted to it
///        // even when the compiler does not know the concrete type of the object
///    }
/// }
/// ```
#[macro_export]
#[allow_internal_unstable(specialization, ptr_metadata)]
macro_rules! register_trait_for_type {
    (dyn $trt:path, $typ:ty,  $reg:ident) => {
        {
            type Metadata = ::std::ptr::DynMetadata<dyn $trt>;
            struct AsDyn<T: ?Sized>{
                dd: ::std::marker::PhantomData<T>,
            }
            trait AsDynImpl{
                fn as_dyn() -> Option<Metadata>;
            }
            impl<T> AsDynImpl for AsDyn<T> {
                default fn as_dyn() -> Option<Metadata>{
                    None
                }
            }
            impl<T: $trt> AsDynImpl for AsDyn<T> {
                fn as_dyn() -> Option<Metadata>{
                    Some(::std::ptr::metadata(::std::ptr::null::<T>() as *const dyn $trt))
                }
            }
            if let Some(gotten) =AsDyn::<$typ>::as_dyn(){
               unsafe{ ::iza_trait_cast::trait_registry::TypeVTableMapper::register_vtable::<_,$typ>($reg, gotten);}
            };
        }
    };
}



///
///
/// Used to register the vtables for types.
/// The implementation is used to mention the traits to consider for casting,
/// so each type registered will be able to be casted to that trait,
/// if it implements the trait
///
/// ## Example
///
/// ```
/// use std::any::Any;
/// use iza_trait_cast::register_trait_for_type;///
///
///
/// use iza_trait_cast::trait_registry::{TraitVTableRegisterer, TypeVTableMapper};
///
/// struct MyRegisterer;
///
/// trait Other : Any{}
///
/// impl TraitVTableRegisterer for MyRegisterer{
///     fn register_trait_vtables_for_type<T: Any>(&self, registry: &mut TypeVTableMapper) {
///        register_trait_for_type!(dyn Other, T, registry);
///        // now if type T implements dyn Other, it will be able to be casted to it
///        // even when the compiler does not know the concrete type of the object
///    }
/// }
/// ```
///
pub trait TraitVTableRegisterer {
    /// ## Example
    ///
    /// ```
    /// use std::any::Any;
    /// use iza_trait_cast::register_trait_for_type;
    ///
    ///
    /// use iza_trait_cast::trait_registry::{TraitVTableRegisterer, TypeVTableMapper};
    ///
    /// struct MyRegisterer;
    ///
    /// trait Other : Any{}
    ///
    /// impl TraitVTableRegisterer for MyRegisterer{
    ///     fn register_trait_vtables_for_type<T: Any>(&self, registry: &mut TypeVTableMapper) {
    ///        register_trait_for_type!(dyn Other, T, registry);
    ///        // now if type T implements dyn Other, it will be able to be casted to it
    ///        // even when the compiler does not know the concrete type of the object
    ///    }
    /// }
    /// ```
    fn register_trait_vtables_for_type<T: Any>(&self,  registry: &mut TypeVTableMapper){

    }
}



/// Gets the vtable
pub(crate) fn get_vtable(obj: &(impl Any + ?Sized), trait_type_id: TypeId, type_registry: &TraitVTableRegistry<impl TraitVTableRegisterer>) -> Result<&'static (), &'static str>{
    let obj_type_id = obj.type_id();
    let type_registration_maybe =type_registry.trait_registration_mapper.get(&obj_type_id);
    match type_registration_maybe{
        Some(type_registration) => {
            match type_registration.vtables.get(&trait_type_id) {
                None => {
                    Err("Trait not implemented")
                }
                Some(gotten) => {
                    Ok(*gotten)
                }
            }
        }
        None => {
            Err("Type not registered")
        }
    }

}
/// Holds vtables of traits for a type
pub struct TypeVTableMapper {
    concrete_type_id: TypeId,
    vtables: HashMap<TypeId, &'static ()>
}

impl TypeVTableMapper {
    /// Registers the vtable of trait TCastTo for the object
    pub unsafe fn register_vtable<TCastTo: Any + ?Sized, TType: Any>(&mut self,   vtable: DynMetadata<TCastTo>) {

        self.vtables.insert(TypeId::of::<TCastTo>(), transmute(vtable));
    }
    fn new(concrete_type_id: TypeId) -> TypeVTableMapper {
        TypeVTableMapper {concrete_type_id, vtables: HashMap::new()}
    }
}
