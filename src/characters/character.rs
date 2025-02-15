use std::cell::RefCell;

pub struct CharacterData<TCharacter: Character>{
    health: i32,
    on_death: RefCell<Vec<fn(&TCharacter)>>
}

impl<TCharacter: Character> Default for CharacterData<TCharacter>{
    fn default() -> CharacterData<TCharacter>{
        CharacterData{
            health: 10,
            on_death: Default::default(),
        }
    }
}
pub trait CharacterBehaviour<TCharacter: Character>{
    fn can_move(&self, character: &TCharacter) -> bool;
}

pub trait Character{

    type Behaviour: CharacterBehaviour<Self> + ?Sized where Self: Sized;

    fn add_death_listener(&self,kk: fn(&Self))
    where Self: Sized{
        self.character_data().on_death.borrow_mut().push(kk);
    }
    fn on_death(&self);
    fn behaviour(&self) -> &Self::Behaviour where Self: Sized;
    fn character_data(&self) -> &CharacterData<Self> where Self: Sized;
    fn can_move(&self) -> bool;

    fn health(&self) -> i32;
}
default impl<TCharacter: Character> Character for TCharacter {
    fn can_move(&self) -> bool {
        self.behaviour().can_move(self)
    }
    fn on_death(&self) {
        for i in self.character_data().on_death.borrow().iter(){
            i(self);
        }
    }
    fn health(&self) -> i32 {
        self.character_data().health
    }
}

impl<TCharacter: Character> CharacterBehaviour<TCharacter> for DefaultCharacterBehaviour{
    fn can_move(&self, character: &TCharacter) -> bool {
        true
    }

}
#[derive(Default)]
pub struct DefaultCharacterBehaviour;
