use crate::modifier::StatModifier;

// By default (single-threaded) implementation is most optimized by using std::rc
// if one needs Stat to live in a multithreaded environment, enabling sync feature uses std::sync instead
#[cfg(not(feature = "sync"))]
type ReferenceCounted<T> = std::rc::Rc<T>;
#[cfg(not(feature = "sync"))]
type Weak<T> = std::rc::Weak<T>;
#[cfg(not(feature = "sync"))]
type InteriorCell<T> = std::cell::RefCell<T>;
#[cfg(feature = "sync")]
type ReferenceCounted<T> = std::sync::Arc<T>;
#[cfg(feature = "sync")]
type Weak<T> = std::sync::Weak<T>;
#[cfg(feature = "sync")]
type InteriorCell<T> = std::sync::Arc<std::sync::Mutex<T>>;

#[cfg(not(feature = "sync"))]
#[inline]
fn new_interior_cell<T>(value: T) -> InteriorCell<T> {
    InteriorCell::new(value)
}

#[cfg(feature = "sync")]
#[inline]
fn new_interior_cell<T>(value: T) -> InteriorCell<T> {
    std::sync::Arc::new(std::sync::Mutex::new(value))
}

#[cfg(not(feature = "sync"))]
type RefMut<'a, T> = std::cell::RefMut<'a, T>;

#[cfg(feature = "sync")]
type RefMut<'a, T> = std::sync::MutexGuard<'a, T>;

#[cfg(not(feature = "sync"))]
#[inline]
fn borrow_cell<T>(cell: &InteriorCell<T>) -> std::cell::RefMut<T> {
    cell.borrow_mut()
}

#[cfg(feature = "sync")]
#[inline]
fn borrow_cell<T>(cell: &InteriorCell<T>) -> std::sync::MutexGuard<T> {
    // cell.try_lock().unwrap()
    cell.lock().unwrap()
}

/// This handle is returned from calling ```stat.add_modifier()``` (technically it's returned in the Ok, result).
///
/// The handle controls the validity of a modifier.
/// Once dropped, the modifier is automatically removed from the [`super::Stat`] that created it.
pub type StatModifierHandle = ReferenceCounted<StatModifierHandleTag>;

/// Just an empty 'flavor' struct, to indicate that the [`StatModifierHandle`] is an owner of some value
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StatModifierHandleTag;

/// A value that can be modified through [`super::StatModifier`]
///
/// ```const M: usize``` decides how many modifiers a stat can maximally hold (modifier are internally an array on the stack)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stat<const M: usize> {
    pub base_value: f32,
    // calculated from base_value and modifiers
    #[cfg_attr(feature = "serde", serde(skip, default = "default_value"))]
    value: InteriorCell<f32>,

    #[cfg_attr(feature = "serde", serde(skip, default = "default_modifiers"))]
    modifiers: InteriorCell<[Option<ModifierMeta>; M]>,
}

#[cfg(feature = "serde")]
fn default_modifiers<const M: usize>() -> InteriorCell<[Option<ModifierMeta>; M]> {
    const NONE: Option<ModifierMeta> = None;
    new_interior_cell([NONE; M])
}

#[cfg(feature = "serde")]
fn default_value() -> InteriorCell<f32> {
    new_interior_cell(0.0f32)
}

/// create a stat from i32 (Stat is always internally a f32)
impl<const M: usize> From<i32> for Stat<M> {
    fn from(value: i32) -> Self {
        Self::new(value as f32)
    }
}

/// create a stat from i32 (Stat is always internally a f32)
impl<const M: usize> From<f32> for Stat<M> {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug)]
struct ModifierMeta {
    modifier: StatModifier,
    order: i32,
    owner_modifier_weak: Weak<StatModifierHandleTag>,
}

/// This stat can't hold any more modifiers.
/// The [`Stat`] M size should be carefully selected. [`Stat<3>`] [`Stat<7>`]
#[derive(Debug, Clone, Copy)]
pub struct ModifiersFullError;

impl<const M: usize> Default for Stat<M> {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl<const M: usize> Stat<M> {
    /// ```
    /// // EXAMPLE: Creates a stat that can hold a maximum of 3 modifiers
    /// # use game_stat::prelude::*;
    /// let attack_stat: Stat<3> = Stat::new(0.0);
    /// let attack_stat = Stat::<3>::new(0.0);
    /// ```
    pub fn new(base_value: f32) -> Self {
        const NONE: Option<ModifierMeta> = None;
        Self {
            base_value,
            value: new_interior_cell(base_value),
            modifiers: new_interior_cell([NONE; M]),
            // value: InteriorCell::new(base_value),
            // modifiers: InteriorCell::new([NONE; M]),
        }
    }

    /// Add a modifier using the default order. [`super::StatModifier::default_order()`]
    /// panics if refcell is borrowed
    pub fn add_modifier(
        &mut self,
        modifier: StatModifier,
    ) -> Result<StatModifierHandle, ModifiersFullError> {
        // We have to update the modifiers array in case one has been dropped.
        // The modifier array could be full of data, yet have modifiers that aren't valid.
        // If we drop a modifier and then add one right away, there should be space for it to be added.
        // This ensures the array is up to date.
        self.update_modifiers();
        let handle = match borrow_cell(&self.modifiers)
            .iter_mut()
            .find(|m| m.is_none())
        {
            Some(modifier_option) => {
                let key = ReferenceCounted::new(StatModifierHandleTag);
                *modifier_option = Some(ModifierMeta {
                    order: modifier.default_order(),
                    modifier,
                    owner_modifier_weak: ReferenceCounted::downgrade(&key),
                });
                Ok(key)
            }
            None => {
                return Err(ModifiersFullError);
            }
        };
        self.calculate_internal_value();
        handle
    }

    /// panics if refcell is borrowed
    pub fn add_modifier_with_order(
        &mut self,
        modifier: StatModifier,
        order: i32,
    ) -> Result<StatModifierHandle, ModifiersFullError> {
        // We have to update the modifiers array in case one has been dropped.
        // The modifier array could be full of data, yet have modifiers that aren't valid.
        // If we drop a modifier and then add one right away, there should be space for it to be added.
        // This ensures the array is up to date.
        self.update_modifiers();
        let handle = match borrow_cell(&self.modifiers)
            .iter_mut()
            .find(|m| m.is_none())
        {
            Some(modifier_option) => {
                let key = ReferenceCounted::new(StatModifierHandleTag);
                *modifier_option = Some(ModifierMeta {
                    modifier,
                    owner_modifier_weak: ReferenceCounted::downgrade(&key),
                    order,
                });
                Ok(key)
            }
            None => {
                return Err(ModifiersFullError);
            }
        };

        // value needs to update
        self.calculate_internal_value();
        handle
    }

    // check if any modifiers have been dropped, and update the value + array
    /// panics if refcell is borrowed
    fn update_modifiers(&self) {
        let any_modifier_dropped = borrow_cell(&self.modifiers)
            .iter()
            .filter_map(|m| m.as_ref())
            .any(|m| m.owner_modifier_weak.upgrade().is_none());
        if any_modifier_dropped {
            self.calculate_internal_value();
        }
    }

    /// returns base value with modifiers applied from self AND other stats's modifiers
    /// the other_stat's modifiers are all applied after 'self' applies it's modifiers
    /// the base value from other_stat is not taken into any account
    /// panics if refcell is borrowed
    pub fn value_with_integrated_modifiers(&mut self, other_stat: &Self) -> f32 {
        other_stat.update_modifiers();
        let highest_order = self.highest_order();
        // temporarily hold handles
        const NONE: Result<StatModifierHandle, ModifiersFullError> = Err(ModifiersFullError);
        let mut handles: [Result<StatModifierHandle, ModifiersFullError>; M] = [NONE; M];

        let mut other_modifiers = borrow_cell(&other_stat.modifiers);
        for (i, modifier) in other_modifiers
            .iter_mut()
            .filter_map(|modifier| modifier.as_ref())
            .enumerate()
        {
            handles[i] = self.add_modifier_with_order(
                modifier.modifier.clone(),
                highest_order + 1 + modifier.order,
            );
        }
        self.value()
    }

    /// Returns the highest order of all modifiers
    /// panics if refcell is borrowed
    pub fn highest_order(&self) -> i32 {
        self.update_modifiers();
        let modifiers = borrow_cell(&self.modifiers);
        modifiers
            .iter()
            .flatten()
            .map(|modifier_meta| modifier_meta.order)
            .max()
            .unwrap_or(0)
    }

    /// Returns the internal base_value with modifiers applied
    /// panics if refcell is borrowed
    pub fn value(&self) -> f32 {
        self.update_modifiers();
        *borrow_cell(&self.value)
    }

    /// Returns the INPUT base_value with modifiers applied
    /// panics if refcell is borrowed
    pub fn value_with_base(&self, base_value: f32) -> f32 {
        let mut value = base_value;
        // Order the modifiers
        let mut modifiers = borrow_cell(&self.modifiers);
        Self::order_modifiers(&mut modifiers);
        Self::apply_modifiers_to_value(modifiers, &mut value);
        value
    }

    /// order modifiers and apply to base value
    /// panics if refcell is borrowed
    fn calculate_internal_value(&self) {
        let mut value = self.base_value;

        // Order the modifiers
        let mut modifiers = borrow_cell(&self.modifiers);
        Self::order_modifiers(&mut modifiers);
        Self::apply_modifiers_to_value(modifiers, &mut value);
        let mut internal_value = borrow_cell(&self.value);
        *internal_value = value;
    }
    // modifiers: InteriorCell<[Option<ModifierMeta>; M]>,

    fn order_modifiers(modifiers: &mut RefMut<[Option<ModifierMeta>; M]>) {
        use std::cmp::Ordering;
        modifiers.sort_by(|m1_option, m2_option| {
            if let Some(m1) = m1_option {
                if let Some(m2) = m2_option {
                    m1.order.cmp(&m2.order)
                } else {
                    Ordering::Less
                }
            } else {
                Ordering::Greater
            }
        });
    }

    fn apply_modifiers_to_value(mut modifiers: RefMut<[Option<ModifierMeta>; M]>, value: &mut f32) {
        for modifier_meta_option in modifiers.iter_mut() {
            if let Some(modifier_meta) = modifier_meta_option {
                match modifier_meta.owner_modifier_weak.upgrade() {
                    Some(_key) => modifier_meta.modifier.apply(value),
                    // owner has dropped the modifier, make this modifier available again
                    None => *modifier_meta_option = None,
                }
            }
        }
    }
}
