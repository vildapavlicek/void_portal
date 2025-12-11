#![allow(clippy::type_complexity)]

use {
    bevy::prelude::*,
    common::{Dead, EnemyKilled, Reward},
};

pub struct VoidWalletPlugin;

impl Plugin for VoidWalletPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Wallet>()
            .add_systems(Update, update_wallet_from_enemy_killed);
    }
}

#[derive(Resource, Default, Debug)]
pub struct Wallet {
    pub void_shards: f32,
}

fn update_wallet_from_enemy_killed(
    mut events: MessageReader<EnemyKilled>,
    mut wallet: ResMut<Wallet>,
    reward_query: Query<&Reward, With<Dead>>,
) {
    for event in events.read() {
        if let Ok(reward) = reward_query.get(event.entity) {
            wallet.void_shards += reward.0;
            info!(
                "Wallet updated: +{} void shards. Total: {}",
                reward.0, wallet.void_shards
            );
        } else {
            warn!(
                "EnemyKilled event received for entity {:?} but no Reward/Dead component found",
                event.entity
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_update() {
        let mut app = App::new();
        // Add minimal plugins required for the test
        app.add_plugins(MinimalPlugins)
            .add_plugins(VoidWalletPlugin)
            .add_message::<EnemyKilled>();

        // Check initial state
        assert_eq!(app.world().resource::<Wallet>().void_shards, 0.0);

        // Spawn a dead enemy with reward
        let entity1 = app
            .world_mut()
            .spawn((
                Reward(10.0),
                Dead {
                    despawn_timer: Timer::from_seconds(1.0, TimerMode::Once),
                },
            ))
            .id();

        // Send message
        let mut messages = app.world_mut().resource_mut::<Messages<EnemyKilled>>();
        messages.write(EnemyKilled { entity: entity1 });

        // Run systems
        app.update();

        // Check updated state
        assert_eq!(app.world().resource::<Wallet>().void_shards, 10.0);

        // Spawn another dead enemy
        let entity2 = app
            .world_mut()
            .spawn((
                Reward(5.5),
                Dead {
                    despawn_timer: Timer::from_seconds(1.0, TimerMode::Once),
                },
            ))
            .id();

        // Send another
        let mut messages = app.world_mut().resource_mut::<Messages<EnemyKilled>>();
        messages.write(EnemyKilled { entity: entity2 });

        app.update();

        assert_eq!(app.world().resource::<Wallet>().void_shards, 15.5);
    }
}
