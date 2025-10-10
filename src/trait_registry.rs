use std::any::{type_name, Any, TypeId};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::marker::{PhantomData, Unsize};
use std::mem::transmute;
use std::ptr::{metadata, null, DynMetadata, Pointee};

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
    pub fn new(registerer: T) -> Self {
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
        let mut helper = RegistererHelper::<T>{
            phantom: PhantomData,
            registered_traits: &mut self.registered_traits,
            type_vtable_mapper: type_registration
        };
        self.registerer.register_trait_vtables_for_type(&mut helper);

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
/// use iza_trait_cast::trait_registry::{RegistererHelper, TraitVTableRegisterer, TypeVTableMapper};
///
/// struct MyRegisterer;
///
/// trait Other : Any{}
///
/// impl TraitVTableRegisterer for MyRegisterer{
///     fn register_trait_vtables_for_type(&self, registerer_helper: &mut RegistererHelper<impl Any>) {
///         registerer_helper.register_trait_vtables::<dyn Other>();
///        // now if type T implements dyn Other, it will be able to be casted to it
///        // even when the compiler does not know the concrete type of the object
///    }
/// }
/// ```





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
/// use iza_trait_cast::trait_registry::{RegistererHelper, TraitVTableRegisterer, TypeVTableMapper};
///
/// struct MyRegisterer;
///
/// trait Other : Any{}
///
/// impl TraitVTableRegisterer for MyRegisterer{
///     fn register_trait_vtables_for_type(&self, registerer_helper: &mut RegistererHelper<impl Any>) {
///         registerer_helper.register_trait_vtables::<dyn Other>();
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
    ///
    ///
    /// use iza_trait_cast::trait_registry::{RegistererHelper, TraitVTableRegisterer, TypeVTableMapper};
    ///
    /// struct MyRegisterer;
    ///
    /// trait Other : Any{}
    ///
    /// impl TraitVTableRegisterer for MyRegisterer{
    ///     fn register_trait_vtables_for_type(&self, registerer_helper: &mut RegistererHelper<impl Any>) {
    ///         registerer_helper.register_trait_vtables::<dyn Other>();
    ///        // now if type T implements dyn Other, it will be able to be casted to it
    ///        // even when the compiler does not know the concrete type of the object
    ///    }
    /// }
    /// ```
    fn register_trait_vtables_for_type(&self, registerer_helper: &mut RegistererHelper<impl Any>){

    }
}

pub enum CastError {
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
impl Debug for CastError {
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
pub(crate) fn get_vtable<TCastTo: ?Sized + 'static + Pointee<Metadata=DynMetadata<TCastTo>>>(obj: &(impl Castable + ?Sized),  type_registry: &TraitVTableRegistry<impl TraitVTableRegisterer>) -> Result<&'static (), CastError>{
    let obj_type_id = obj.type_id();
    let type_registration_maybe =type_registry.trait_registration_mapper.get(&obj_type_id);
    match type_registration_maybe{
        Some(type_registration) => {
            match type_registration.vtables.get(&TypeId::of::<TCastTo>()) {
                None => {
                    if type_registry.is_trait_registered(&TypeId::of::<TCastTo>()) {
                        Err(CastError::TraitNotImplemented {trait_name: type_name::<TCastTo>(), trait_id: TypeId::of::<TCastTo>(), type_id: obj_type_id, type_name: obj.type_name()})
                    } else{
                        Err(CastError::TraitNotRegistered{trait_name: type_name::<TCastTo>(), trait_id: TypeId::of::<TCastTo>()})
                    }
                }
                Some(gotten) => {
                    Ok(*gotten)
                }
            }
        }
        None => {
            Err(CastError::TypeNotRegistered {type_id: obj_type_id, type_name: obj.type_name()})
        }
    }

}
pub struct RegistererHelper<'a,T: 'static> {
    phantom: PhantomData<&'a T>,
    type_vtable_mapper: &'a mut TypeVTableMapper,
    registered_traits: &'a mut HashSet<TypeId>,

}

impl<'a,Type: 'static> RegistererHelper<'a,Type>{

    pub fn register_trait_vtables<Trait: ?Sized + Pointee<Metadata=DynMetadata<Trait>> + 'static>(
         &mut self){
        struct AsDyn<Type: 'static> {
            kk: PhantomData<fn() -> Type>,
        }
        trait AsDynImpl<Trait: ?Sized + Pointee<Metadata=DynMetadata<Trait>>>{
            type ToReg : Sized;
            fn register(registry: &mut RegistererHelper<Self::ToReg>);
        }
        impl<Trait: ?Sized + Pointee<Metadata=DynMetadata<Trait>> + 'static, Type: 'static> AsDynImpl<Trait> for AsDyn<Type> {
            type ToReg = Type;
            default fn register(registry: &mut RegistererHelper<'_, <AsDyn<Type> as AsDynImpl<Trait>>::ToReg>){

            }
        }
        impl<Type: Unsize<Trait> + 'static,Trait: ?Sized + Pointee<Metadata=DynMetadata<Trait>> + 'static> AsDynImpl<Trait> for AsDyn<Type> {
            fn register(registry: &mut RegistererHelper<Type>) {
                registry.type_vtable_mapper.register_vtable::<Trait,Type>()

            }
        }
        <AsDyn<Type> as AsDynImpl<Trait>>::register(self);
        self.registered_traits.insert(TypeId::of::<Trait>());
    }

}
/// Holds vtables of traits for a type
#[derive(Default)]
pub struct TypeVTableMapper {
    vtables: HashMap<TypeId, &'static ()>
}

impl TypeVTableMapper {
    /// Registers the vtable of trait TCastTo for the object
    pub fn register_vtable<TCastTo: 'static + ?Sized + Pointee<Metadata=DynMetadata<TCastTo>>, TType: 'static + Unsize<TCastTo>>(&mut self) {
        unsafe{
            self.vtables.insert(TypeId::of::<TCastTo>(), transmute(metadata(null::<TType>() as *const TCastTo)));
        }
    }
    pub fn new() -> TypeVTableMapper {
        TypeVTableMapper { vtables: HashMap::new()}
    }
}
