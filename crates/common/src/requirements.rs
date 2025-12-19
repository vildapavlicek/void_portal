use {bevy::prelude::*, serde::Deserialize};

#[derive(Debug, Clone, Reflect, Deserialize, PartialEq, Default)]
pub enum Requirement {
    /// The requirement is always satisfied, regardless of the level.
    #[default]
    Always,
    /// Satisfied if the current level is greater than or equal to the specified minimum.
    MinLevel(u32),
    /// Satisfied if the current level is within the inclusive range [min, max].
    LevelRange(u32, u32),
    /// Satisfied periodically. The level must be at least the `offset`,
    /// and `(current_level - offset) % interval == 0`.
    Periodic {
        /// The level interval at which the requirement is satisfied.
        interval: u32,
        /// The starting level offset for the periodic requirement.
        offset: u32,
    },
}

impl Requirement {
    pub fn is_satisfied(&self, current_level: u32) -> bool {
        match self {
            Self::Always => true,
            Self::MinLevel(min) => current_level >= *min,
            Self::LevelRange(min, max) => current_level >= *min && current_level <= *max,
            Self::Periodic { interval, offset } => {
                current_level >= *offset && (current_level - offset).is_multiple_of(*interval)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requirement_always() {
        let req = Requirement::Always;
        assert!(req.is_satisfied(0));
        assert!(req.is_satisfied(10));
    }

    #[test]
    fn test_requirement_min_level() {
        let req = Requirement::MinLevel(5);
        assert!(!req.is_satisfied(0));
        assert!(!req.is_satisfied(4));
        assert!(req.is_satisfied(5));
        assert!(req.is_satisfied(10));
    }

    #[test]
    fn test_requirement_level_range() {
        let req = Requirement::LevelRange(5, 10);
        assert!(!req.is_satisfied(4));
        assert!(req.is_satisfied(5));
        assert!(req.is_satisfied(10));
        assert!(!req.is_satisfied(11));
    }

    #[test]
    fn test_requirement_periodic() {
        let req = Requirement::Periodic {
            interval: 5,
            offset: 2,
        };
        assert!(!req.is_satisfied(0));
        assert!(!req.is_satisfied(1));
        assert!(req.is_satisfied(2)); // 2
        assert!(!req.is_satisfied(3));
        assert!(req.is_satisfied(7)); // 2 + 5
        assert!(req.is_satisfied(12)); // 2 + 10
    }
}
