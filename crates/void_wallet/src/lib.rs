use {bevy::prelude::*, void_core::events::EnemyKilled};

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
) {
    for event in events.read() {
        wallet.void_shards += event.reward;
        info!(
            "Wallet updated: +{} void shards. Total: {}",
            event.reward, wallet.void_shards
        );
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
            // EnemyKilled is a Message, so we need to add it as a message to the app
            // (though VoidWalletPlugin doesn't add it, void_core might, but here we depend on void_core types)
            // Actually, `update_wallet_from_enemy_killed` reads `MessageReader`.
            // In Bevy 0.17 Messages, we need to register the message type `app.add_message::<T>()`
            // The plugin assumes someone else registers it, OR the plugin should register it if it owns it.
            // EnemyKilled is owned by void_core. So we should probably register it in the test manually
            // if we don't include VoidCorePlugin.
            .add_message::<EnemyKilled>();

        // Check initial state
        assert_eq!(app.world().resource::<Wallet>().void_shards, 0.0);

        // Send message
        let mut messages = app.world_mut().resource_mut::<Messages<EnemyKilled>>();
        messages.write(EnemyKilled { reward: 10.0 });

        // Run systems
        app.update();

        // Check updated state
        assert_eq!(app.world().resource::<Wallet>().void_shards, 10.0);

        // Send another
        let mut messages = app.world_mut().resource_mut::<Messages<EnemyKilled>>();
        messages.write(EnemyKilled { reward: 5.5 });

        app.update();

        assert_eq!(app.world().resource::<Wallet>().void_shards, 15.5);
    }
}
