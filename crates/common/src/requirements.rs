use {bevy::prelude::*, serde::Deserialize};

#[derive(Debug, Clone, Reflect, Deserialize, PartialEq, Default)]
pub enum Condition {
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

impl Condition {
    pub fn is_satisfied(&self, current_level: u32) -> bool {
        match self {
            Self::Always => true,
            Self::MinLevel(min) => current_level >= *min,
            Self::LevelRange(min, max) => current_level >= *min && current_level <= *max,
            Self::Periodic { interval, offset } => {
                current_level >= *offset && (current_level - offset) % interval == 0
            }
        }
    }
}

#[derive(Debug, Clone, Reflect, Deserialize, PartialEq, Default)]
pub struct Requirement<T> {
    pub condition: Condition,
    pub value: T,
}

impl<T> Requirement<T> {
    pub fn new(condition: Condition, value: T) -> Self {
        Self { condition, value }
    }

    pub fn get(&self, context: u32) -> Option<&T> {
        self.condition.is_satisfied(context).then_some(&self.value)
    }

    pub fn get_mut(&mut self, context: u32) -> Option<&mut T> {
        self.condition
            .is_satisfied(context)
            .then_some(&mut self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_always() {
        let cond = Condition::Always;
        assert!(cond.is_satisfied(0));
        assert!(cond.is_satisfied(10));
    }

    #[test]
    fn test_condition_min_level() {
        let cond = Condition::MinLevel(5);
        assert!(!cond.is_satisfied(0));
        assert!(!cond.is_satisfied(4));
        assert!(cond.is_satisfied(5));
        assert!(cond.is_satisfied(10));
    }

    #[test]
    fn test_requirement_wrap() {
        let req = Requirement::new(Condition::MinLevel(5), 100);
        assert_eq!(req.get(4), None);
        assert_eq!(req.get(5), Some(&100));
    }
}
