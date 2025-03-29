use std::any::{type_name, Any, TypeId};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
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
    trait_registration_mapper: HashMap<TypeId, TypeVTableMapper>,
    registered_traits: HashSet<TypeId>
}

impl<T: Default + TraitVTableRegisterer> Default for TraitVTableRegistry<T> {
    fn default() -> Self {
        Self{
            registered_traits: HashSet::new(),
            registerer: T::default(),
            trait_registration_mapper: HashMap::new()
        }
    }
}
impl<T: TraitVTableRegisterer> TraitVTableRegistry<T> {
    fn new(registerer: T) -> Self {
        Self{
            registerer,
            trait_registration_mapper: HashMap::new(),
            registered_traits: HashSet::new()
        }
    }
}
impl<TraitReg: TraitVTableRegisterer> TraitVTableRegistry<TraitReg> {
    /// Used to include a type for casting, so u can cast that type to another trait even when the compiler does not know the underlying type
    pub fn register_type<T: Any>(&mut self) {
        let type_id = TypeId::of::<T>();
        let mut type_registration = match self.trait_registration_mapper.entry(type_id) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(TypeVTableMapper::new()),
        };
        let mut helper = RegistererHelper{
            registered_traits: &mut self.registered_traits,
            type_vtable_mapper: type_registration
        };
        self.registerer.register_trait_vtables_for_type::<T>(&mut helper);

    }
    #[inline]
    pub fn is_trait_registered(&self,type_id: &TypeId) -> bool {
        self.registered_traits.contains(type_id)
    }
    #[inline]
    pub fn is_type_registered(&self,type_id: &TypeId) -> bool {
        self.trait_registration_mapper.contains_key(type_id)
    }
}



/// ## Example
///
/// ```
/// use std::any::Any;
/// use iza_trait_cast::register_trait_for_type;
///
///
/// use iza_trait_cast::trait_registry::{RegistererHelper, TraitVTableRegisterer, TypeVTableMapper};
///
/// struct MyRegisterer;
///
/// trait Other : Any{}
///
/// impl TraitVTableRegisterer for MyRegisterer{
///     fn register_trait_vtables_for_type<T: Any>(&self, registerer_helper: &mut RegistererHelper) {
///        register_trait_for_type!(dyn Other, T, registerer_helper);
///        // now if type T implements dyn Other, it will be able to be casted to it
///        // even when the compiler does not know the concrete type of the object
///    }
/// }
/// ```
#[macro_export]
#[allow_internal_unstable(specialization, ptr_metadata)]
macro_rules! register_trait_for_type {
    (dyn $trt:path, $typ:ty,  $reg:ident) =>
    {
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

            unsafe{
                if let Some(gotten) = <AsDyn<$typ> as AsDynImpl>::as_dyn(){
                    $crate::trait_registry::RegistererHelper::register_vtable::<_,$typ>($reg, gotten);

                };
                $crate::trait_registry::RegistererHelper::register_trait::<dyn $trt>($reg);
            }


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
/// use iza_trait_cast::register_trait_for_type;
///
///
/// use iza_trait_cast::trait_registry::{RegistererHelper, TraitVTableRegisterer, TypeVTableMapper};
///
/// struct MyRegisterer;
///
/// trait Other : Any{}
///
/// impl TraitVTableRegisterer for MyRegisterer{
///     fn register_trait_vtables_for_type<T: Any>(&self, registerer_helper: &mut RegistererHelper) {
///        register_trait_for_type!(dyn Other, T, registerer_helper);
///        // now if type T implements dyn Other, it will be able to be casted to it
///        // even when the compiler does not know the concrete type of the object
///    }
/// }
/// ```
pub trait TraitVTableRegisterer {
    /// ## Example
    ///
    /// ```
    /// use std::any::Any;
    /// use iza_trait_cast::register_trait_for_type;
    ///
    ///
    /// use iza_trait_cast::trait_registry::{RegistererHelper, TraitVTableRegisterer, TypeVTableMapper};
    ///
    /// struct MyRegisterer;
    ///
    /// trait Other : Any{}
    ///
    /// impl TraitVTableRegisterer for MyRegisterer{
    ///     fn register_trait_vtables_for_type<T: Any>(&self, registerer_helper: &mut RegistererHelper) {
    ///        register_trait_for_type!(dyn Other, T, registerer_helper);
    ///        // now if type T implements dyn Other, it will be able to be casted to it
    ///        // even when the compiler does not know the concrete type of the object
    ///    }
    /// }
    /// ```
    fn register_trait_vtables_for_type<T: Any>(&self, registerer_helper: &mut RegistererHelper){

    }
}

pub enum VTableError{
    TraitNotImplemented{
        trait_name: &'static str,
        trait_id: TypeId,
        type_name: &'static str,
        type_id: TypeId,
    },
    TraitNotRegistered{
        trait_name: &'static str,
        trait_id: TypeId,
    },
    TypeNotRegistered{
        type_name: &'static str,
        type_id: TypeId,

    }

}
impl Debug for VTableError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TraitNotImplemented{trait_name, type_name,.. } => {
                f.write_fmt(format_args!("trait '{trait_name}' not implemented by the underlying concrete type '{type_name}'"))
            },
            Self::TraitNotRegistered{trait_name, .. } => {
                f.write_fmt(format_args!("trait '{trait_name}' not registered"))
            },
            Self::TypeNotRegistered{ type_name, .. } => {
                f.write_fmt(format_args!("type '{type_name}' not registered"))
            }
        }

    }
}
pub trait Castable: Any{
    fn type_name(&self) -> &'static str;
}
impl<T: Any> Castable for T{
    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }
}
/// Gets the vtable
pub(crate) fn get_vtable<TCastTo: ?Sized + 'static>(obj: &(impl Castable + ?Sized),  type_registry: &TraitVTableRegistry<impl TraitVTableRegisterer>) -> Result<&'static (), VTableError>{
    let obj_type_id = obj.type_id();
    let type_registration_maybe =type_registry.trait_registration_mapper.get(&obj_type_id);
    match type_registration_maybe{
        Some(type_registration) => {
            match type_registration.vtables.get(&TypeId::of::<TCastTo>()) {
                None => {
                    if type_registry.is_trait_registered(&TypeId::of::<TCastTo>()) {
                        Err(VTableError::TraitNotImplemented {trait_name: type_name::<TCastTo>(), trait_id: TypeId::of::<TCastTo>(), type_id: obj_type_id, type_name: obj.type_name()})
                    } else{
                        Err(VTableError::TraitNotRegistered{trait_name: type_name::<TCastTo>(), trait_id: TypeId::of::<TCastTo>()})
                    }
                }
                Some(gotten) => {
                    Ok(*gotten)
                }
            }
        }
        None => {
            Err(VTableError::TypeNotRegistered {type_id: obj_type_id, type_name: obj.type_name()})
        }
    }

}
pub struct RegistererHelper<'a>{
    type_vtable_mapper: &'a mut TypeVTableMapper,
    /// Used to keep track of the traits that are registered
    registered_traits: &'a mut HashSet<TypeId>,
}

impl<'a> RegistererHelper<'a>{
    // registers vtable for a type
    pub unsafe fn register_vtable<TCastTo: 'static + ?Sized, TType: Any>(&mut self,   vtable: DynMetadata<TCastTo>) {
        self.type_vtable_mapper.register_vtable::<TCastTo,TType>(vtable);

    }
    /// Used to make the TraitRegistry note that the trait has been registered
    pub unsafe fn register_trait<TTrait: 'static + ?Sized>(&mut self){
        self.registered_traits.insert(TypeId::of::<TTrait>());
    }
}
/// Holds vtables of traits for a type
#[derive(Default)]
pub struct TypeVTableMapper {
    vtables: HashMap<TypeId, &'static ()>
}

impl TypeVTableMapper {
    /// Registers the vtable of trait TCastTo for the object
    unsafe fn register_vtable<TCastTo: 'static + ?Sized, TType: Any>(&mut self,   vtable: DynMetadata<TCastTo>) {
        self.vtables.insert(TypeId::of::<TCastTo>(), transmute(vtable));
    }
    fn new() -> TypeVTableMapper {
        TypeVTableMapper { vtables: HashMap::new()}
    }
}
