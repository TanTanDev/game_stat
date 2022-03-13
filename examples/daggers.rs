use game_stat::prelude::*;
// info: A more complex example for how you could integrate this library in a game
// I don't think anyone would ever write it like this, but nontheless, some example code :)

// in 'this specific game' we will never need more than 2 stat modifiers
const MAX_STAT_MODIFIERS: usize = 2;

pub struct DaggerItem {
    name: String,
    // our game data probably shouldn't deal with modifier directly
    // a modifier will be later created using this value
    attack_damage: f32,
}

pub struct Player {
    inventory: Vec<DaggerItem>,
    hand: Option<(DaggerItem, StatModifierHandle)>,
    attack_damage_stat: Stat<MAX_STAT_MODIFIERS>,
}

impl Player {
    pub fn new(base_attack_damage: f32) -> Self {
        Self {
            inventory: Vec::with_capacity(4),
            hand: None,
            attack_damage_stat: Stat::new(base_attack_damage),
        }
    }

    pub fn add_item(&mut self, dagger: DaggerItem) {
        self.inventory.push(dagger);
    }

    pub fn unequip_item(&mut self) {
        // returns currently equiped item into inventory
        // THE MODIFIER WILL BE DROPPED because self.hand holds the modifier key
        if let Some(previous_equip) = self.hand.take() {
            self.inventory.push(previous_equip.0);
        }
    }

    pub fn equip_item_from_index(&mut self, i: usize) {
        // return previous equipment to inventory
        if let Some(previous_equip) = self.hand.take() {
            self.inventory.push(previous_equip.0);
        }
        // move from inventory to hand
        let new_dagger = self.inventory.remove(i);
        // extract the modifier from the dagger stats
        let modifier_key_result = self
            .attack_damage_stat
            .add_modifier(StatModifier::Flat(new_dagger.attack_damage));

        // adding a modifier can fail if we exceed the max amount of modifiers
        // T should be carefully selected for for Stat<T>
        let modifier_key = modifier_key_result.expect("");
        self.hand = Some((new_dagger, modifier_key));
    }

    pub fn hurt_monster(&mut self, monster: &mut Monster) {
        monster.health -= self.attack_damage_stat.value();

        // just some flavoring
        let attack_method = match &self.hand {
            Some(dagger) => &dagger.0.name,
            None => "just his hands",
        };
        println!(
            "using: {}, monster took: {} damage! now it has {} health",
            attack_method,
            self.attack_damage_stat.value(),
            monster.health
        );
    }
}

pub struct Monster {
    health: f32,
}

fn main() {
    let mut player = Player::new(1f32);
    player.add_item(DaggerItem {
        name: "stabby stabby".to_string(),
        attack_damage: 50f32,
    });
    let mut monster = Monster { health: 100f32 };

    player.hurt_monster(&mut monster);

    // our next attack will hurt!
    player.equip_item_from_index(0);
    player.hurt_monster(&mut monster);

    // I'm weak again...
    player.unequip_item();
    player.hurt_monster(&mut monster);
}
