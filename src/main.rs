
use std::any::{Any, TypeId};
use crate::trait_registry::*;
mod trait_registry;

trait Character : Castable{
    fn name(&self) -> &'static str;
}

trait SuperHuman : Character{
    fn power_level(&self) -> i32;
}

struct Lily;

impl Castable for Lily {
    fn register(&self, type_registration: &mut TypeRegistration) {
        type_registration.register::<dyn Character>(self);
        type_registration.register::<dyn SuperHuman>(self);
    }
}
impl Character for Lily{
    fn name(&self) -> &'static str{
        "Lily"
    }
}


impl SuperHuman for Lily{
    fn power_level(&self) -> i32 {
        5
    }
}

static KK: i32 = 5;
pub fn main(){

    let kk : &dyn Character = &Lily;
    match cast_ref::<dyn SuperHuman>(kk){
        None => {
            println!("NOO");
        }
        Some(std) => {
            println!("{}",std.power_level());
        }
    }
}