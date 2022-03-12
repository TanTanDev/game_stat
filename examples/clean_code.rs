use game_stat::prelude::*;

// just a reminder that you can type alias 
// it's nice having to avoid Writing Stat<N> which can look a bit odd
type GameStat = Stat<2>;

fn main() {
    let mut _armor_stat: GameStat = Stat::new(10f32);
    let mut _attack_damage_stat: GameStat = Stat::new(10f32);
}
