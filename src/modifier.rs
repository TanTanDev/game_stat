#[derive(Copy, Clone)]
pub enum Modifier {
    // add or subtract flat value
    Flat(f32),
    PercentAdd(f32),
    PercentMultiply(f32),
}

impl Modifier {
    pub fn apply(&self, value: &mut f32) {
        match self {
            Modifier::Flat(v) => *value += v,
            Modifier::PercentAdd(v) => *value *= 1.0f32 + v,
            Modifier::PercentMultiply(v) => *value *= v,
        }
    }

    // if no order is provided, we use this default that seems sensible
    pub fn default_order(&self) -> i32 {
        match self {
            Modifier::Flat(_) => 0,
            Modifier::PercentAdd(_) => 1,
            Modifier::PercentMultiply(_) => 2,
        }
    }
}
