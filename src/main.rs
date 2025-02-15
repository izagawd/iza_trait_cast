#![feature(decl_macro)]
#![feature(specialization)]

use std::mem::MaybeUninit;
use crate::characters::character::Character;
use crate::characters::cooldown_character::CooldownCharacter;
use crate::characters::lily::Lily;

mod characters{
    pub mod character;
    pub mod cooldown_character;
    pub mod lily;
}



fn main() {
    let bruh = &Lily::default();

    bruh.set_cooldown(4);

    println!("{}",bruh.get_cooldown())

}
