#[derive(Copy, Clone)]
pub enum StatModifier {
    // add or subtract flat value
    Flat(f32),
    PercentAdd(f32),
    PercentMultiply(f32),
}

impl StatModifier {
    pub fn apply(&self, value: &mut f32) {
        match self {
            StatModifier::Flat(v) => *value += v,
            StatModifier::PercentAdd(v) => *value *= 1.0f32 + v,
            StatModifier::PercentMultiply(v) => *value *= v,
        }
    }

    // if no order is provided, we use this default that seems sensible
    pub fn default_order(&self) -> i32 {
        match self {
            StatModifier::Flat(_) => 0,
            StatModifier::PercentAdd(_) => 1,
            StatModifier::PercentMultiply(_) => 2,
        }
    }
}
