#![allow(clippy::type_complexity)]

use {
    bevy::prelude::*,
    common::{MonsterKilled, MonsterScavenged, Reward, SpawnFloatingText},
};

pub struct VoidWalletPlugin;

impl Plugin for VoidWalletPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Wallet>()
            .add_message::<common::MonsterScavenged>()
            .add_systems(
                Update,
                (
                    update_wallet_from_monster_killed,
                    update_wallet_from_scavenge,
                ),
            );
    }
}

#[derive(Resource, Default, Debug)]
pub struct Wallet {
    pub void_shards: f32,
}

fn update_wallet_from_monster_killed(
    mut events: MessageReader<MonsterKilled>,
    mut wallet: ResMut<Wallet>,
    reward_query: Query<(&Reward, &Transform)>,
    mut vfx_events: MessageWriter<SpawnFloatingText>,
) {
    for event in events.read() {
        if let Ok((reward, transform)) = reward_query.get(event.entity) {
            wallet.void_shards += reward.0;

            vfx_events.write(SpawnFloatingText::void_shards_reward(
                reward.0,
                transform.translation,
            ));

            info!(
                "Wallet updated: +{} void shards. Total: {}",
                reward.0, wallet.void_shards
            );
        } else {
            warn!(
                "MonsterKilled event received for entity {:?} but no Reward/Dead component found",
                event.entity
            );
        }
    }
}

fn update_wallet_from_scavenge(
    mut events: MessageReader<MonsterScavenged>,
    mut wallet: ResMut<Wallet>,
    mut vfx_events: MessageWriter<SpawnFloatingText>,
) {
    for event in events.read() {
        wallet.void_shards += event.amount;

        vfx_events.write(SpawnFloatingText {
            text: format!("+{:.0}", event.amount),
            location: event.location,
            color: Color::srgb(0.7, 0.7, 0.7), // Scavenge color
            size: 18.0,
        });

        info!(
            "Wallet scavenge update: +{}. Total: {}",
            event.amount, wallet.void_shards
        );
    }
}

#[cfg(test)]
mod tests {
    use {super::*, common::Dead};

    #[test]
    fn test_wallet_update() {
        let mut app = App::new();
        // Add minimal plugins required for the test
        app.add_plugins(MinimalPlugins)
            .add_plugins(VoidWalletPlugin)
            .add_message::<MonsterKilled>()
            .add_message::<SpawnFloatingText>();

        // Check initial state
        assert_eq!(app.world().resource::<Wallet>().void_shards, 0.0);

        // Spawn a dead monster with reward
        let entity1 = app
            .world_mut()
            .spawn((
                Reward(10.0),
                Transform::default(),
                Dead {
                    despawn_timer: Timer::from_seconds(1.0, TimerMode::Once),
                },
            ))
            .id();

        // Send message
        let mut messages = app.world_mut().resource_mut::<Messages<MonsterKilled>>();
        messages.write(MonsterKilled { entity: entity1 });

        // Run systems
        app.update();

        // Check updated state
        assert_eq!(app.world().resource::<Wallet>().void_shards, 10.0);

        // Spawn another dead monster
        let entity2 = app
            .world_mut()
            .spawn((
                Reward(5.5),
                Transform::default(),
                Dead {
                    despawn_timer: Timer::from_seconds(1.0, TimerMode::Once),
                },
            ))
            .id();

        // Send another
        let mut messages = app.world_mut().resource_mut::<Messages<MonsterKilled>>();
        messages.write(MonsterKilled { entity: entity2 });

        app.update();

        assert_eq!(app.world().resource::<Wallet>().void_shards, 15.5);
    }

    #[test]
    fn test_wallet_scavenge() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(VoidWalletPlugin);
        // Note: VoidWalletPlugin already adds MonsterScavenged message, but NOT MonsterKilled
        // So when Update runs, `update_wallet_from_monster_killed` panics because `MonsterKilled` is missing.
        // We must add it for the system to not panic, or disable the system for this test.
        app.add_message::<MonsterKilled>()
            .add_message::<SpawnFloatingText>();

        assert_eq!(app.world().resource::<Wallet>().void_shards, 0.0);

        let mut messages = app.world_mut().resource_mut::<Messages<MonsterScavenged>>();
        messages.write(MonsterScavenged {
            amount: 12.5,
            location: Vec3::ZERO,
        });

        app.update();

        assert_eq!(app.world().resource::<Wallet>().void_shards, 12.5);
    }
}
