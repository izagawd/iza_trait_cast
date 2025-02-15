use crate::characters::character::{Character, CharacterBehaviour, CharacterData};
use crate::characters::cooldown_character::{CooldownCharacter, DefaultCooldownCharacterBehaviour, CooldownData};



pub struct Lily{
    character_data: CharacterData<Self>,
    cooldown_character_behaviour: Option<DefaultCooldownCharacterBehaviour>,
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
            Some(DefaultCooldownCharacterBehaviour::new(&created));

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
    type Behaviour = DefaultCooldownCharacterBehaviour;
    fn behaviour(&self) -> &Self::Behaviour
    where
        Self: Sized
    {
        self.cooldown_character_behaviour.as_ref().unwrap()
    }

    fn character_data(&self) -> &CharacterData<Self> {
        &self.character_data
    }


}