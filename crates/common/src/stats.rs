use bevy::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Reflect, Deserialize, PartialEq, Default)]
pub enum GrowthStrategy {
    #[default]
    /// Value = Base + (Level * Factor)
    Linear,
    /// Value = Base * (Factor ^ Level)
    Exponential,
}

#[derive(Debug, Clone, Reflect, Deserialize)]
pub struct UpgradeableStat {
    /// Current level of the stat
    pub level: f32,

    /// Current calculated value
    pub value: f32,
    /// Current calculated price to upgrade
    pub price: f32,

    /// Base value used for recalculation
    pub base_value: f32,
    /// Base price used for recalculation
    pub base_price: f32,

    /// Factor used for value growth
    pub value_growth_factor: f32,
    /// Strategy used for value growth
    pub value_growth_type: GrowthStrategy,

    /// Factor used for price growth
    pub price_growth_factor: f32,
    /// Strategy used for price growth
    pub price_growth_type: GrowthStrategy,
}

impl Default for UpgradeableStat {
    fn default() -> Self {
        Self {
            level: 0.0,
            value: 0.0,
            price: 0.0,
            base_value: 0.0,
            base_price: 0.0,
            value_growth_factor: 0.0,
            value_growth_type: GrowthStrategy::Linear,
            price_growth_factor: 0.0,
            price_growth_type: GrowthStrategy::Linear,
        }
    }
}

impl UpgradeableStat {
    pub fn new(
        base_value: f32,
        base_price: f32,
        value_growth_factor: f32,
        value_growth_type: GrowthStrategy,
        price_growth_factor: f32,
        price_growth_type: GrowthStrategy,
    ) -> Self {
        let mut stat = Self {
            level: 0.0,
            value: base_value,
            price: base_price,
            base_value,
            base_price,
            value_growth_factor,
            value_growth_type,
            price_growth_factor,
            price_growth_type,
        };
        stat.recalculate();
        stat
    }

    /// Increments the level by 1.0 and recalculates stats
    pub fn upgrade(&mut self) {
        self.level += 1.0;
        self.recalculate();
    }

    /// Sets the level explicitly and recalculates stats
    pub fn set_level(&mut self, level: f32) {
        self.level = level;
        self.recalculate();
    }

    fn calculate_value(base: f32, level: f32, factor: f32, strategy: GrowthStrategy) -> f32 {
        match strategy {
            GrowthStrategy::Linear => base + (level * factor),
            GrowthStrategy::Exponential => base * factor.powf(level),
        }
    }

    /// Recalculates value and price based on current level and base stats
    pub fn recalculate(&mut self) {
        self.value = Self::calculate_value(
            self.base_value,
            self.level,
            self.value_growth_factor,
            self.value_growth_type,
        );

        self.price = Self::calculate_value(
            self.base_price,
            self.level,
            self.price_growth_factor,
            self.price_growth_type,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_growth() {
        // Base: 10, Factor: 2 => L0=10, L1=12, L2=14
        let mut stat = UpgradeableStat::new(
            10.0, 100.0,
            2.0, GrowthStrategy::Linear,
            0.0, GrowthStrategy::Linear,
        );

        assert_eq!(stat.value, 10.0);

        stat.upgrade();
        assert_eq!(stat.level, 1.0);
        assert_eq!(stat.value, 12.0); // 10 + 1*2

        stat.upgrade();
        assert_eq!(stat.level, 2.0);
        assert_eq!(stat.value, 14.0); // 10 + 2*2
    }

    #[test]
    fn test_exponential_growth() {
        // Base: 10, Factor: 2 => L0=10, L1=20, L2=40
        let mut stat = UpgradeableStat::new(
            10.0, 100.0,
            2.0, GrowthStrategy::Exponential,
            0.0, GrowthStrategy::Linear,
        );

        assert_eq!(stat.value, 10.0); // 10 * 2^0

        stat.upgrade();
        assert_eq!(stat.level, 1.0);
        assert_eq!(stat.value, 20.0); // 10 * 2^1

        stat.upgrade();
        assert_eq!(stat.level, 2.0);
        assert_eq!(stat.value, 40.0); // 10 * 2^2
    }

    #[test]
    fn test_mixed_growth() {
        // Value: Linear (+10), Price: Exponential (x1.1)
        let mut stat = UpgradeableStat::new(
            100.0, 10.0,
            10.0, GrowthStrategy::Linear,
            1.1, GrowthStrategy::Exponential,
        );

        stat.set_level(5.0);

        // Value = 100 + 5 * 10 = 150
        assert_eq!(stat.value, 150.0);

        // Price = 10 * 1.1^5 = 10 * 1.61051 = 16.1051
        assert!((stat.price - 16.1051).abs() < 0.001);
    }

    #[test]
    fn test_f32_level_scaling() {
         let mut stat = UpgradeableStat::new(
            10.0, 100.0,
            2.0, GrowthStrategy::Linear,
            0.0, GrowthStrategy::Linear,
        );

        stat.set_level(0.5);
        assert_eq!(stat.value, 11.0); // 10 + 0.5 * 2
    }
}
