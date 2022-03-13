//! [![](https://github.com/TanTanDev/game_stat/blob/main/branding/gamestat_logo.png)]()
//! **game_stat** gives you the power to modify some base value through modifiers. Most commonly seen in video games
//!
//! ```
//! # use game_stat::prelude::*;
//! let mut armor_stat: Stat<2> = Stat::new(10f32);
//! {
//!     let _modifier_handle = armor_stat.add_modifier(StatModifier::Flat(5f32));
//!     println!("armor_stat is: {} it should be 15!", armor_stat.value());
//! }
//! println!("armor_stat is: {}, It should be 10!", armor_stat.value());
//! ```
//! * [`Stat<2>`] is a stat that can hold a maximum of 2 modifiers. (the modifiers is an array internally, carefully select a sensible value)
//! * ```armor_stat.value()``` returns our stat value based on what modifiers are active.
//! * We add a [`StatModifier`], it is valid as long as the [`StatModifierHandle`] that is returned from [`Stat::add_modifier()`] exists, which is why our value goes back to 10 when it gets dropped from the stack

mod modifier;
mod stat;
pub use crate::modifier::*;
pub use crate::stat::*;

pub mod prelude {
    pub use crate::modifier::StatModifier;
    pub use crate::stat::{Stat, StatModifierHandle};
}
