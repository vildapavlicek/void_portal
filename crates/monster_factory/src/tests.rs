#[cfg(test)]
use {
    crate::*,
    bevy::prelude::{App, MinimalPlugins, Transform, Update, Vec2, Visibility},
    common::{
        components::{MonsterScaling, PortalLevel, PortalRoot, PortalUpgrades, ScavengerPenalty},
        GrowthStrategy, Reward, ScavengeModifier,
    },
    monsters::{Health, Lifetime, Monster, Speed},
    std::collections::HashMap,
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
        .register_type::<MonsterBuilder>();

    // Register components for hydration target
    app.register_type::<Health>()
        .register_type::<Speed>()
        .register_type::<Reward>()
        .register_type::<Lifetime>()
        .register_type::<ScavengeModifier>()
        .register_type::<Monster>();

    // Register Portal components
    app.register_type::<PortalLevel>()
        .register_type::<MonsterScaling>()
        .register_type::<PortalRoot>()
        .register_type::<PortalUpgrades>()
        .register_type::<ScavengerPenalty>();

    app.add_systems(Update, systems::hydrate_monster_stats);

    // 1. Spawn Mock Portal
    let portal_entity = app
        .world_mut()
        .spawn((
            PortalRoot,
            PortalLevel {
                active: 1,
                max_unlocked: 1,
            },
            MonsterScaling {
                health_strategy: GrowthStrategy::Linear {
                    base: 100.0,
                    coefficient: 0.0,
                },
                speed_strategy: GrowthStrategy::Linear {
                    base: 10.0,
                    coefficient: 0.0,
                },
                reward_strategy: GrowthStrategy::Linear {
                    base: 50.0,
                    coefficient: 0.0,
                },
                lifetime_strategy: GrowthStrategy::Linear {
                    base: 20.0,
                    coefficient: 0.0,
                },
            },
            ScavengerPenalty(0.5),
            PortalUpgrades(HashMap::new()), // No upgrades for simplicity
        ))
        .id();

    // 2. Spawn Monster with Builder pointing to Portal
    let target_pos = Vec2::new(10.0, 10.0);
    let builder = MonsterBuilder {
        portal_entity,
        spawn_index: 1,
        target_position: target_pos,
    };

    let entity = app
        .world_mut()
        .spawn((
            builder,
            HpCoef { val: 1.5 },
            SpeedCoef { val: 2.0 },
            RewardCoef { val: 0.5 },
            LifetimeCoef { val: 1.0 },
            Transform::default(),
            Visibility::default(),
        ))
        .id();

    app.update();

    // 3. Check if hydration worked
    let health = app.world().get::<Health>(entity);
    assert!(health.is_some(), "Health component missing");
    assert_eq!(health.unwrap().max, 150.0); // 100 * 1.5

    let speed = app.world().get::<Speed>(entity);
    assert!(speed.is_some(), "Speed component missing");
    assert_eq!(speed.unwrap().0, 20.0); // 10 * 2.0

    let reward = app.world().get::<Reward>(entity);
    assert!(reward.is_some(), "Reward component missing");
    assert_eq!(reward.unwrap().0, 25.0); // 50 * 0.5

    let lifetime = app.world().get::<Lifetime>(entity);
    assert!(lifetime.is_some(), "Lifetime component missing");
    // Lifetime = (20 * 1.0) + 0 (bonus) = 20
    assert_eq!(lifetime.unwrap().timer.duration().as_secs_f32(), 20.0);

    let scav = app.world().get::<ScavengeModifier>(entity);
    assert!(scav.is_some(), "ScavengeModifier component missing");
    assert_eq!(scav.unwrap().0, 0.5);

    let enemy = app.world().get::<Monster>(entity);
    assert!(enemy.is_some(), "Enemy component missing");
    assert_eq!(enemy.unwrap().target_position, target_pos);

    // Ensure builder and coefs are removed
    assert!(
        app.world().get::<MonsterBuilder>(entity).is_none(),
        "MonsterBuilder should be removed"
    );
    assert!(
        app.world().get::<HpCoef>(entity).is_none(),
        "HpCoef should be removed"
    );
}
