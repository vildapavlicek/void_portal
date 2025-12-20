use {crate::requirements::Requirement, bevy::prelude::*, serde::Deserialize};

#[derive(Debug, Clone, Reflect, Deserialize, PartialEq)]
pub enum GrowthStrategy {
    /// Returns the same value regardless of level.
    Static(f32),
    /// Calculation: base + (level * coefficient)
    Linear { base: f32, coefficient: f32 },
    /// Calculation: base * (factor ^ level)
    Exponential { base: f32, factor: f32 },
    /// Calculation: base + (level * step)
    Incremental { base: f32, step: f32 },
    /// Calculation: base + floor(level / interval) * step
    Chunked {
        base: f32,
        interval: f32,
        step: f32,
    },
}

impl Default for GrowthStrategy {
    fn default() -> Self {
        Self::Static(0.0)
    }
}

impl GrowthStrategy {
    pub fn calculate(&self, level: f32) -> f32 {
        match self {
            Self::Static(val) => *val,
            Self::Linear { base, coefficient } => base + (level * coefficient),
            Self::Exponential { base, factor } => base * factor.powf(level),
            Self::Incremental { base, step } => base + (level * step),
            Self::Chunked {
                base,
                interval,
                step,
            } => base + (level / interval).floor() * step,
        }
    }
}

pub type ConditionalUpgrade = Requirement<GrowthStrategy>;

impl ConditionalUpgrade {
    pub fn calculate(&self, level: u32) -> Option<f32> {
        self.get(level).map(|strategy| strategy.calculate(level as f32))
    }
}

#[derive(Debug, Clone, Reflect, Deserialize, Component)]
#[reflect(Component)]
pub struct UpgradeableStat {
    pub level: f32,

    // Cached current state
    pub value: f32,
    pub price: f32,

    // Logic containers
    pub value_strategy: GrowthStrategy,
    pub price_strategy: GrowthStrategy,
}

impl Default for UpgradeableStat {
    fn default() -> Self {
        Self {
            level: 0.0,
            value: 0.0,
            price: 0.0,
            value_strategy: GrowthStrategy::default(),
            price_strategy: GrowthStrategy::default(),
        }
    }
}

impl UpgradeableStat {
    pub fn new(value_strategy: GrowthStrategy, price_strategy: GrowthStrategy) -> Self {
        let mut stat = Self {
            level: 0.0,
            value: 0.0,
            price: 0.0,
            value_strategy,
            price_strategy,
        };
        stat.recalculate();
        stat
    }

    pub fn upgrade(&mut self) {
        self.level += 1.0;
        self.recalculate();
    }

    pub fn set_level(&mut self, level: f32) {
        self.level = level;
        self.recalculate();
    }

    pub fn recalculate(&mut self) {
        self.value = self.value_strategy.calculate(self.level);
        self.price = self.price_strategy.calculate(self.level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::requirements::Condition;

    #[test]
    fn test_static_strategy() {
        let strategy = GrowthStrategy::Static(10.0);
        assert_eq!(strategy.calculate(0.0), 10.0);
        assert_eq!(strategy.calculate(100.0), 10.0);
    }

    #[test]
    fn test_linear_strategy() {
        let strategy = GrowthStrategy::Linear {
            base: 10.0,
            coefficient: 2.0,
        };
        assert_eq!(strategy.calculate(0.0), 10.0);
        assert_eq!(strategy.calculate(1.0), 12.0);
        assert_eq!(strategy.calculate(2.0), 14.0);
    }

    #[test]
    fn test_exponential_strategy() {
        let strategy = GrowthStrategy::Exponential {
            base: 10.0,
            factor: 2.0,
        };
        assert_eq!(strategy.calculate(0.0), 10.0);
        assert_eq!(strategy.calculate(1.0), 20.0);
        assert_eq!(strategy.calculate(2.0), 40.0);
    }

    #[test]
    fn test_incremental_strategy() {
        let strategy = GrowthStrategy::Incremental {
            base: 5.0,
            step: 1.0,
        };
        assert_eq!(strategy.calculate(0.0), 5.0);
        assert_eq!(strategy.calculate(1.0), 6.0);
        assert_eq!(strategy.calculate(2.0), 7.0);
    }

    #[test]
    fn test_upgradeable_stat() {
        let mut stat = UpgradeableStat::new(
            GrowthStrategy::Incremental {
                base: 10.0,
                step: 1.0,
            },
            GrowthStrategy::Linear {
                base: 100.0,
                coefficient: 50.0,
            },
        );

        // Level 0
        assert_eq!(stat.level, 0.0);
        assert_eq!(stat.value, 10.0);
        assert_eq!(stat.price, 100.0);

        // Upgrade to Level 1
        stat.upgrade();
        assert_eq!(stat.level, 1.0);
        assert_eq!(stat.value, 11.0);
        assert_eq!(stat.price, 150.0);
    }

    #[test]
    fn test_conditional_upgrade() {
        let conditional = ConditionalUpgrade::new(
            Condition::MinLevel(2),
            GrowthStrategy::Static(10.0),
        );

        assert_eq!(conditional.calculate(0), None);
        assert_eq!(conditional.calculate(1), None);
        assert_eq!(conditional.calculate(2), Some(10.0));
        assert_eq!(conditional.calculate(3), Some(10.0));
    }

    #[test]
    fn test_conditional_upgrade_default() {
        let conditional = ConditionalUpgrade::default();
        // Default requirement is Always, default strategy is Static(0.0)
        assert_eq!(conditional.calculate(0), Some(0.0));
        assert_eq!(conditional.calculate(10), Some(0.0));
    }

    #[test]
    fn test_chunked_strategy() {
        let strategy = GrowthStrategy::Chunked {
            base: 0.0,
            interval: 10.0,
            step: 5.0,
        };
        // 0-9 -> 0
        assert_eq!(strategy.calculate(0.0), 0.0);
        assert_eq!(strategy.calculate(5.0), 0.0);
        assert_eq!(strategy.calculate(9.0), 0.0);
        // 10-19 -> 5
        assert_eq!(strategy.calculate(10.0), 5.0);
        assert_eq!(strategy.calculate(15.0), 5.0);
        assert_eq!(strategy.calculate(19.0), 5.0);
        // 20-29 -> 10
        assert_eq!(strategy.calculate(20.0), 10.0);
        assert_eq!(strategy.calculate(25.0), 10.0);
        assert_eq!(strategy.calculate(29.0), 10.0);
        // 30 -> 15
        assert_eq!(strategy.calculate(30.0), 15.0);
    }
}
