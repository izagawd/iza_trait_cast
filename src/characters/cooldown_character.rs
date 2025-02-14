use crate::characters::character::{Character, CharacterBehaviour};


pub struct CooldownData{
    cooldown: i32
}


impl Default for CooldownData {
    fn default() -> Self {
        CooldownData{cooldown: 5}
    }
}

pub struct CooldownCharacterBehaviour;

impl  CooldownCharacterBehaviour {
    pub fn new(character: &impl Character) -> Self {
        character.add_death_listener(|f|{
            println!("I fucking died")
        });
        CooldownCharacterBehaviour{}
    }
}
pub trait CooldownCharacter: Character{
    fn cooldown_data(&self) -> &CooldownData;
}
impl<TCharacter: CooldownCharacter> CharacterBehaviour<TCharacter> for CooldownCharacterBehaviour {
    fn can_move(&self, character: &TCharacter) -> bool {
        return character.cooldown_data().cooldown == 0;
    }
}