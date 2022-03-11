use modular_stats::prelude::*;

#[test]
fn base_value() {
    let mut stat: Stat<3> = Stat::new(8f32);
    assert!(stat.value() == 8f32);
}

#[test]
fn flat_modifier() {
    let mut stat: Stat<3> = Stat::new(8f32);
    {
        let _modifier_key = stat.add_modifier(Modifier::Flat(7f32));
        assert!(stat.value() == 15f32);
    }
    assert!(stat.value() == 8f32);
}

#[test]
fn percent_add_modifier() {
    let mut stat: Stat<3> = Stat::new(10f32);
    {
        let _modifier_key = stat.add_modifier(Modifier::PercentAdd(0.5f32));
        assert!(stat.value() == 15f32);
    }
    assert!(stat.value() == 10f32);
}

#[test]
fn percent_multiply_modifier() {
    let mut stat: Stat<3> = Stat::new(10f32);
    {
        let _modifier_key = stat.add_modifier(Modifier::PercentMultiply(0.5f32));
        assert!(stat.value() == 5f32);
    }
    assert!(stat.value() == 10f32);
}

#[test]
fn all_modifiers() {
    let mut stat: Stat<3> = Stat::new(10f32);
    {
        let _modifier_key_flat = stat.add_modifier(Modifier::Flat(90f32));
        let _modifier_key_percent_add = stat.add_modifier(Modifier::PercentAdd(-0.5f32));
        let _modifier_key_percent_multiply = stat.add_modifier(Modifier::PercentMultiply(0.5f32));
        assert!(stat.value() == 25f32);
    }
    assert!(stat.value() == 10f32);
}

#[test]
fn all_modifiers_reverse_order() {
    let mut stat: Stat<3> = Stat::new(10f32);
    {
        let _modifier_key_percent_multiply =
            stat.add_modifier_with_order(Modifier::PercentMultiply(0.5f32), 0);
        let _modifier_key_percent_add =
            stat.add_modifier_with_order(Modifier::PercentAdd(0.5f32), 1);
        let _modifier_key_flat = stat.add_modifier_with_order(Modifier::Flat(2.5f32), 2);
        assert!(stat.value() == 10f32);
    }
    assert!(stat.value() == 10f32);
}

#[test]
// test that if we try to add more modifiers than we set it up to support
// in this test, only 2 flat modifiers should be applied
fn to_many_modifiers() {
    let mut stat: Stat<2> = Stat::new(0f32);
    let _modifier_1 = stat.add_modifier(Modifier::Flat(1.0f32));
    let _modifier_2 = stat.add_modifier_with_order(Modifier::Flat(1.0f32), 0);
    let modifier_3 = stat.add_modifier_with_order(Modifier::Flat(1.0f32), 0);
    assert!(stat.value() == 2f32);
    assert!(modifier_3.is_err());
}
