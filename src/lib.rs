pub mod modifier;
pub mod stat;

pub mod prelude {
    pub use crate::modifier::StatModifier;
    pub use crate::stat::{ModifierKey, Stat};
}
