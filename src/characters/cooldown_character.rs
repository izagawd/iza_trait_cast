use std::cell::Cell;
use crate::characters::character::{Character, CharacterBehaviour};


pub struct CooldownData{
    cooldown: Cell<i32>
}


impl Default for CooldownData {
    fn default() -> Self {
        CooldownData{cooldown: Cell::new(5)}
    }
}

pub struct DefaultCooldownCharacterBehaviour;


trait CooldownCharacterBehaviour<TCharacter : CooldownCharacter> : CharacterBehaviour<TCharacter> {
    fn set_cooldown(&self, character: &TCharacter, cooldown: i32){
        character.cooldown_data().cooldown.set(cooldown) 
    }

    fn get_cooldown(&self, character: &TCharacter) -> i32{
        character.cooldown_data().cooldown.get()
    }
}

impl<TCharacter: CooldownCharacter> CharacterBehaviour<TCharacter> for DefaultCooldownCharacterBehaviour {
    fn can_move(&self, character: &TCharacter) -> bool {
        return character.cooldown_data().cooldown.get() == 0;
    }
}
impl<TCharacter : CooldownCharacter> CooldownCharacterBehaviour<TCharacter> for DefaultCooldownCharacterBehaviour {
    
}
impl DefaultCooldownCharacterBehaviour {
    pub fn new(character: &impl Character) -> Self {
        character.add_death_listener(|f|{
            println!("I fucking died")
        });
        DefaultCooldownCharacterBehaviour {}
    }
}



pub trait CooldownCharacter: Character{
    fn cooldown_data(&self) -> &CooldownData;
    fn get_cooldown(&self)->i32;
    
    fn set_cooldown(&self, cooldown: i32);
}

default impl<T : Character<Behaviour : CooldownCharacterBehaviour<T>>> CooldownCharacter for T  {
    fn get_cooldown(&self) -> i32 {
        self.behaviour().get_cooldown(self)
    }
    fn set_cooldown(&self, cooldown: i32) {
        self.behaviour().set_cooldown(self,cooldown)
    }
}