use crate::modifier::StatModifier;
use tinyvec::{ArrayVec, TinyVec};

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
#[derive(Copy, Clone, Debug, Default)]
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

    #[cfg_attr(feature = "serde", serde(skip))]
    modifiers: InteriorCell<TinyVec<[ModifierMeta; M]>>,
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

impl Default for ModifierMeta {
    fn default() -> Self {
        Self {
            modifier: StatModifier::default(),
            order: 0,
            owner_modifier_weak: Weak::default(),
        }
    }
}

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
        let modifiers = TinyVec::Inline(ArrayVec::<[ModifierMeta; M]>::default());
        Self {
            base_value,
            value: new_interior_cell(base_value),
            modifiers: new_interior_cell(modifiers),
        }
    }

    /// Add a modifier using the default order. [`super::StatModifier::default_order()`]
    /// panics if refcell is borrowed
    pub fn add_modifier(&mut self, modifier: StatModifier) -> StatModifierHandle {
        // We have to update the modifiers array in case one has been dropped.
        // The modifier array could be full of data, yet have modifiers that aren't valid.
        // If we drop a modifier and then add one right away, there should be space for it to be added.
        // This ensures the array is up to date.
        self.update_modifiers();

        let handle = ReferenceCounted::new(StatModifierHandleTag);
        let meta = ModifierMeta {
            order: modifier.default_order(),
            modifier,
            owner_modifier_weak: ReferenceCounted::downgrade(&handle),
        };

        let mut modifiers = borrow_cell(&self.modifiers);
        if modifiers.len() + 1 > modifiers.capacity() {
            modifiers.move_to_the_heap();
        }
        modifiers.push(meta);
        drop(modifiers);

        self.calculate_internal_value();
        handle
    }

    /// panics if refcell is borrowed
    pub fn add_modifier_with_order(
        &mut self,
        modifier: StatModifier,
        order: i32,
    ) -> StatModifierHandle {
        // We have to update the modifiers array in case one has been dropped.
        // The modifier array could be full of data, yet have modifiers that aren't valid.
        // If we drop a modifier and then add one right away, there should be space for it to be added.
        // This ensures the array is up to date.
        self.update_modifiers();
        let handle = ReferenceCounted::new(StatModifierHandleTag);
        let modifier_meta = ModifierMeta {
            modifier,
            owner_modifier_weak: ReferenceCounted::downgrade(&handle),
            order,
        };

        let mut modifiers = borrow_cell(&self.modifiers);
        if modifiers.len() + 1 > modifiers.capacity() {
            modifiers.move_to_the_heap();
        }
        modifiers.push(modifier_meta);
        drop(modifiers);

        // value needs to update
        self.calculate_internal_value();
        handle
    }

    // check if any modifiers have been dropped, and update the value + array
    /// panics if refcell is borrowed
    fn update_modifiers(&self) {
        let mut modifiers = borrow_cell(&self.modifiers);
        let mut any_modifier_dropped = false;

        modifiers.retain(|m| {
            let retain = m.owner_modifier_weak.upgrade().is_some();
            if !retain {
                any_modifier_dropped = true;
            }
            retain
        });
        drop(modifiers);

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

        let mut other_modifiers = borrow_cell(&other_stat.modifiers);
        let mut temporary_handles: TinyVec<[StatModifierHandle; M]> =
            TinyVec::with_capacity(other_modifiers.len());

        for modifier in other_modifiers.iter_mut() {
            temporary_handles.push(self.add_modifier_with_order(
                modifier.modifier.clone(),
                highest_order + 1 + modifier.order,
            ));
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

    /// Returns the INPUT base_value (ignores self) with modifiers applied
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

    fn order_modifiers(modifiers: &mut RefMut<TinyVec<[ModifierMeta; M]>>) {
        modifiers.sort_by(|m1, m2| m1.order.cmp(&m2.order));
    }

    fn apply_modifiers_to_value(
        mut modifiers: RefMut<TinyVec<[ModifierMeta; M]>>,
        value: &mut f32,
    ) {
        for modifier_meta in modifiers.iter_mut() {
            if let Some(_key) = modifier_meta.owner_modifier_weak.upgrade() {
                modifier_meta.modifier.apply(value);
            }
        }
    }
}
