#[cfg(test)]
mod tests {
    use {
        crate::*,
        bevy::prelude::*,
        common::{Reward, ScavengeModifier},
        enemy::{Enemy, Health, Lifetime, Speed},
    };

    #[test]
    fn test_hydrate_monster_stats() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Register types used in components
        app.register_type::<HpCoef>()
            .register_type::<SpeedCoef>()
            .register_type::<RewardCoef>()
            .register_type::<LifetimeCoef>()
            .register_type::<MonsterSpawnContext>();

        // Register components for hydration target
        app.register_type::<Health>()
            .register_type::<Speed>()
            .register_type::<Reward>()
            .register_type::<Lifetime>()
            .register_type::<ScavengeModifier>()
            .register_type::<Enemy>();

        app.add_systems(Update, systems::hydrate_monster_stats);

        // Spawn a monster with context and coeffs
        let context = MonsterSpawnContext {
            base_health: 100.0,
            base_speed: 10.0,
            base_reward: 50.0,
            base_lifetime: 20.0,
            bonus_lifetime: 5.0,
            scavenger_penalty: 0.5,
            spawn_index: 1,
            portal_level: 1,
            target_position: Vec2::new(10.0, 10.0),
            spawn_position: Vec2::new(0.0, 0.0),
        };

        let entity = app
            .world_mut()
            .spawn((
                context,
                HpCoef { val: 1.5 },
                SpeedCoef { val: 2.0 },
                RewardCoef { val: 0.5 },
                LifetimeCoef { val: 1.0 },
                // Enemy marker is usually added by hydration if missing or updated?
                // In my logic: hydration ADDS Enemy component.
            ))
            .id();

        app.update();

        // Check if hydration worked
        let health = app.world().get::<Health>(entity);
        assert!(health.is_some());
        assert_eq!(health.unwrap().max, 150.0); // 100 * 1.5

        let speed = app.world().get::<Speed>(entity);
        assert!(speed.is_some());
        assert_eq!(speed.unwrap().0, 20.0); // 10 * 2.0

        let reward = app.world().get::<Reward>(entity);
        assert!(reward.is_some());
        assert_eq!(reward.unwrap().0, 25.0); // 50 * 0.5

        let lifetime = app.world().get::<Lifetime>(entity);
        assert!(lifetime.is_some());
        // Lifetime = (20 * 1.0) + 5 = 25
        assert_eq!(lifetime.unwrap().timer.duration().as_secs_f32(), 25.0);

        let scav = app.world().get::<ScavengeModifier>(entity);
        assert!(scav.is_some());
        assert_eq!(scav.unwrap().0, 0.5);

        let enemy = app.world().get::<Enemy>(entity);
        assert!(enemy.is_some());
        assert_eq!(enemy.unwrap().target_position, Vec2::new(10.0, 10.0));

        // Ensure context and coefs are removed
        assert!(app.world().get::<MonsterSpawnContext>(entity).is_none());
        assert!(app.world().get::<HpCoef>(entity).is_none());
    }
}
