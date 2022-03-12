use crate::modifier::StatModifier;
use core::mem::MaybeUninit;
use std::rc::{Rc, Weak};

pub struct Stat<const M: usize> {
    pub base_value: f32,
    pub modifiers: [Option<ModifierMeta>; M],
    pub cached_orders: [Option<(usize, i32)>; M],
}

pub struct ModifierMeta {
    modifier: StatModifier,
    order: i32,
    owner_modifier_weak: Weak<ModifierKeyTag>,
}

#[derive(Debug, Clone, Copy)]
pub struct AddModifierError;

impl<const M: usize> Stat<M> {
    pub fn new(base_value: f32) -> Self {
        // DANGER DANGER! WARNING WARNING!
        let mut modifiers: [MaybeUninit<Option<ModifierMeta>>; M] =
            unsafe { MaybeUninit::uninit().assume_init() };
        modifiers[..].iter_mut().for_each(|elem| {
            elem.write(None);
        });
        let modifiers = unsafe {
            modifiers
                .as_ptr()
                .cast::<[Option<ModifierMeta>; M]>()
                .read()
        };
        // hopefully we survived that :D
        Self {
            base_value,
            modifiers,
            cached_orders: [None; M],
        }
    }

    pub fn add_modifier(&mut self, modifier: StatModifier) -> Result<ModifierKey, AddModifierError> {
        match self.modifiers.iter_mut().filter(|m| m.is_none()).next() {
            Some(modifier_option) => {
                let key = Rc::new(ModifierKeyTag);
                *modifier_option = Some(ModifierMeta {
                    order: modifier.default_order(),
                    modifier,
                    owner_modifier_weak: Rc::downgrade(&key),
                });
                Ok(key)
            }
            None => Err(AddModifierError),
        }
    }

    pub fn add_modifier_with_order(
        &mut self,
        modifier: StatModifier,
        order: i32,
    ) -> Result<ModifierKey, AddModifierError> {
        match self.modifiers.iter_mut().filter(|m| m.is_none()).next() {
            Some(modifier_option) => {
                let key = Rc::new(ModifierKeyTag);
                *modifier_option = Some(ModifierMeta {
                    modifier,
                    owner_modifier_weak: Rc::downgrade(&key),
                    order,
                });
                Ok(key)
            }
            None => Err(AddModifierError),
        }
    }

    pub fn value(&mut self) -> f32 {
        let mut value = self.base_value;
        self.cached_orders.iter_mut().for_each(|v| *v = None);
        let mut cache_index = 0;
        self.modifiers.iter().enumerate().for_each(|(i, m)| {
            if let Some(modifier) = m {
                self.cached_orders[cache_index] = Some((i, modifier.order));
                cache_index += 1;
            }
        });
        // order
        self.cached_orders.sort_by(|v1, v2| {
            use std::cmp::Ordering;
            if let Some(v1) = v1 {
                if let Some(v2) = v2 {
                    return v1.1.cmp(&v2.1);
                }
            }
            Ordering::Equal
        });
        for order_option in self.cached_orders.iter() {
            if let Some(order) = order_option {
                let modifier_meta = self.modifiers[order.0].as_ref().unwrap();
                if let Some(_owner) = modifier_meta.owner_modifier_weak.upgrade() {
                    modifier_meta.modifier.apply(&mut value);
                } else {
                    // owner droped the modifier, drop the reference first so we can take() and remove the modifier
                    drop(modifier_meta);
                    self.modifiers[order.0].take();
                }
            } else {
                break;
            }
        }
        value
    }
}

pub type ModifierKey = Rc<ModifierKeyTag>;
pub struct ModifierKeyTag;
