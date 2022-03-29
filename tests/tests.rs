use game_stat::prelude::*;

#[test]
fn base_value() {
    let mut stat: Stat<3> = Stat::new(8f32);
    assert!(stat.value() == 8f32);
}

#[test]
fn flat_modifier() {
    let mut stat: Stat<3> = Stat::new(8f32);
    {
        let _modifier_key = stat.add_modifier(StatModifier::Flat(7f32));
        assert!(stat.value() == 15f32);
    }
    assert!(stat.value() == 8f32);
}

#[test]
fn percent_add_modifier() {
    let mut stat: Stat<3> = Stat::new(10f32);
    {
        let _modifier_key = stat.add_modifier(StatModifier::PercentAdd(0.5f32));
        assert!(stat.value() == 15f32);
    }
    assert!(stat.value() == 10f32);
}

#[test]
fn percent_multiply_modifier() {
    let mut stat: Stat<3> = Stat::new(10f32);
    {
        let _modifier_key = stat.add_modifier(StatModifier::PercentMultiply(0.5f32));
        assert!(stat.value() == 5f32);
    }
    assert!(stat.value() == 10f32);
}

#[test]
fn all_modifiers() {
    let mut stat: Stat<3> = Stat::new(10f32);
    {
        let _modifier_key_flat = stat.add_modifier(StatModifier::Flat(90f32));
        let _modifier_key_percent_add = stat.add_modifier(StatModifier::PercentAdd(-0.5f32));
        let _modifier_key_percent_multiply =
            stat.add_modifier(StatModifier::PercentMultiply(0.5f32));
        assert!(stat.value() == 25f32);
    }
    assert!(stat.value() == 10f32);
}

#[test]
fn all_modifiers_reverse_order() {
    let mut stat: Stat<3> = Stat::new(10f32);
    {
        let _modifier_key_percent_multiply =
            stat.add_modifier_with_order(StatModifier::PercentMultiply(0.5f32), 0);
        let _modifier_key_percent_add =
            stat.add_modifier_with_order(StatModifier::PercentAdd(0.5f32), 1);
        let _modifier_key_flat = stat.add_modifier_with_order(StatModifier::Flat(2.5f32), 2);
        assert!(stat.value() == 10f32);
    }
    assert!(stat.value() == 10f32);
}

#[test]
// test that if we try to add more modifiers than we set it up to support
// in this test, only 2 flat modifiers should be applied
fn to_many_modifiers() {
    let mut stat: Stat<2> = Stat::new(0f32);
    let _modifier_1 = stat.add_modifier(StatModifier::Flat(1.0f32));
    let _modifier_2 = stat.add_modifier_with_order(StatModifier::Flat(1.0f32), 0);
    let modifier_3 = stat.add_modifier_with_order(StatModifier::Flat(1.0f32), 0);
    assert!(stat.value() == 2f32);
    assert!(modifier_3.is_err());
}

#[test]
// ensure that when we remove modifier and add new ones, they will be valid
fn array_cleanup() {
    let mut stat: Stat<1> = Stat::new(0f32);
    {
        let _modifier_1 = stat.add_modifier(StatModifier::Flat(1.0f32));
    }
    let _modifier_1 = stat.add_modifier(StatModifier::Flat(1.0f32));
    assert!(stat.value() == 1f32);
}

#[test]
fn all() {
    let mut stat: Stat<2> = Stat::new(0f32);
    {
        let _modifier_1 = stat.add_modifier(StatModifier::Flat(1.0f32));
        assert!(stat.value() == 1f32);
        let _modifier_2 = stat.add_modifier(StatModifier::PercentMultiply(2.0f32));
        assert!(stat.value() == 2f32);
    }
    assert!(stat.value() == 0f32);
    let _modifier_1 = stat.add_modifier(StatModifier::Flat(5.0f32));
    assert!(stat.value() == 5f32);
    let _modifier_2 = stat.add_modifier(StatModifier::Flat(9.0f32));
    assert!(stat.value() == 14f32);
    let modifier_2_result = stat.add_modifier(StatModifier::Flat(9.0f32));
    assert!(modifier_2_result.is_err());
}

#[test]
// cautionary tale:
// I wanted to highlight that shadowing a modifier does not drop the original value until it goes out of scope
// shadowing makes data unaccesable, but things like reference counted things, still point to valid data
// a shadowed value gets cleared up when leaving the current scope, not the moment it gets shadowed
fn key_shadowing() {
    let mut stat: Stat<2> = Stat::new(0f32);
    {
        let _modifier_1 = stat.add_modifier(StatModifier::Flat(1.0f32));
        let _modifier_1 = stat.add_modifier(StatModifier::Flat(1.0f32));
        assert!(stat.value() == 2f32);

        drop(_modifier_1);
        assert!(stat.value() == 1f32);
    } // the original shadowed _modifier_1 get's dropped here
    assert!(stat.value() == 0f32);
}

#[cfg(feature = "sync")]
#[test]
pub fn multithreaded_environment() {
    use std::sync::{Arc, Mutex};
    use std::thread::*;
    let stat = Arc::new(Mutex::new(Stat::<2>::new(0.0f32)));
    let stat_thread_1 = stat.clone();
    let stat_thread_2 = stat.clone();
    let handle_1 = spawn(move || {
        let _modifier_handle = stat_thread_1
            .lock()
            .unwrap()
            .add_modifier(StatModifier::Flat(1.0));
    });
    let handle_2 = spawn(move || {
        let _modifier_handle = stat_thread_2
            .lock()
            .unwrap()
            .add_modifier(StatModifier::Flat(1.0));
    });
    handle_1.join().unwrap();
    handle_2.join().unwrap();
    assert!(stat.lock().unwrap().value() == 0.0f32);
}
