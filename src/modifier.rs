/// Used to transform the base value of a [`super::Stat`]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StatModifier {
    /// Add or subtract flat value. ```StatModifier::Flat(-1.0)``` means it will **subtract -1.0**
    Flat(f32),
    /// Increase or decrease using procentage. ```StatModifier::PercentAdd(0.4)``` the value will **increase by 40%**
    PercentAdd(f32),
    /// Direct multiplication. StatModifier::```PercentMultiply(0.5)``` the value is **halved**
    PercentMultiply(f32),
}

impl StatModifier {
    /// Modifies the input value based on the StateModifier variant
    pub fn apply(&self, value: &mut f32) {
        match self {
            StatModifier::Flat(v) => *value += v,
            StatModifier::PercentAdd(v) => *value *= 1.0f32 + v,
            StatModifier::PercentMultiply(v) => *value *= v,
        }
    }

    /// Returns the default order based on the variant
    pub fn default_order(&self) -> i32 {
        match self {
            StatModifier::Flat(_) => 0,
            StatModifier::PercentAdd(_) => 1,
            StatModifier::PercentMultiply(_) => 2,
        }
    }
}
