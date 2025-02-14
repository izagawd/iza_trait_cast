use crate::characters::character::{Character, CharacterBehaviour, CharacterData};
use crate::characters::cooldown_character::{CooldownCharacter, CooldownCharacterBehaviour, CooldownData};



pub struct Lily{
    character_data: CharacterData<Self>,
    cooldown_character_behaviour: Option<CooldownCharacterBehaviour>,
    cooldown_character_data: CooldownData,
}
impl Default for Lily{
    fn default() -> Self{
        let mut created = Lily{
            character_data: Default::default(),
            cooldown_character_behaviour: Default::default(),
            cooldown_character_data: Default::default(),
        };

        created.cooldown_character_behaviour =
            Some(CooldownCharacterBehaviour::new(&created));

        created.add_death_listener(|myself|{
           println!("I AM THE YAPPER");
        });
        created
    }
}
impl CooldownCharacter for Lily{
    fn cooldown_data(&self) -> &CooldownData {
        &self.cooldown_character_data
    }
}
impl Character for Lily {
    fn behaviour(&self) -> &dyn CharacterBehaviour<Self>
    where
        Self: Sized
    {
        self.cooldown_character_behaviour.as_ref().unwrap()
    }

    fn character_data(&self) -> &CharacterData<Self> {
        &self.character_data
    }


}