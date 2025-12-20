#![allow(clippy::type_complexity)]

use {
    bevy::{ecs::relationship::Relationship, prelude::*, scene::DynamicScene},
    common::{
        DamageMessage, GameState, MarkedForCleanUp, MeleeDamageContext, MeleeHitMessage,
        ProjectileCollisionMessage, ProjectileDamageContext, VoidGameStage,
    },
    items::{
        AttackRange as ItemAttackRange, BaseDamage, Melee, ProjectileStats as ItemProjectileStats,
        Ranged,
    },
    monsters::{Monster, SpawnIndex},
    portal::PortalSpawnTracker,
    std::time::Duration,
};

pub struct PlayerNpcsPlugin;

impl Plugin for PlayerNpcsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageMessage>();
        app.register_type::<PlayerNpc>()
            .register_type::<MovementSpeed>()
            .register_type::<Target>()
            .register_type::<Weapon>()
            .register_type::<WeaponCooldown>()
            .register_type::<Projectile>()
            .register_type::<MasteryTrack>()
            .register_type::<WeaponExpertise>()
            .register_type::<WeaponExpertiseXp>()
            .register_type::<CooldownText>();

        app.add_systems(OnEnter(GameState::Playing), spawn_player_npc);

        app.add_systems(
            Update,
            (
                tick_weapon_cooldowns.in_set(VoidGameStage::FrameStart),
                player_npc_decision_logic.in_set(VoidGameStage::ResolveIntent),
                (
                    player_npc_movement_logic,
                    melee_attack_emit,
                    ranged_attack_logic,
                    (move_projectiles, projectile_collision).chain(),
                )
                    .in_set(VoidGameStage::Actions),
                (
                    resolve_melee_base_damage.pipe(apply_melee_damage),
                    resolve_projectile_base_damage.pipe(apply_projectile_damage),
                )
                    .in_set(VoidGameStage::Effect),
                update_cooldown_text.in_set(VoidGameStage::FrameEnd),
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

pub fn tick_weapon_cooldowns(time: Res<Time>, mut weapon_query: Query<&mut WeaponCooldown>) {
    for mut cooldown in weapon_query.iter_mut() {
        cooldown.timer.tick(time.delta());
    }
}

// Components

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PlayerNpc;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub enum Intent {
    #[default]
    Idle,
    MoveTo(Vec3),
    Attack(Entity),
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct Target(pub Option<Entity>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Weapon;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct WeaponCooldown {
    pub timer: Timer,
}

#[derive(Component, Reflect)]
#[reflect(Component, Default)]
pub struct WeaponExpertiseXp(pub f32);

impl Default for WeaponExpertiseXp {
    fn default() -> Self {
        Self(10.0)
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Projectile {
    pub velocity: Vec3,
    pub lifetime: Timer,
    pub source: Entity,
    pub weapon: Entity,
}

#[derive(Debug, Clone, Reflect, Default)]
#[reflect(Default)]
pub struct MasteryTrack {
    pub level: u32,
    pub current_xp: f32,
}

impl MasteryTrack {
    /// Returns the damage multiplier (e.g., Level 5 = 1.5x damage)
    pub fn get_damage_bonus(&self) -> f32 {
        1.0 + (self.level as f32 * 0.10)
    }

    /// Returns the XP needed for the NEXT level
    pub fn xp_for_next_level(&self) -> f32 {
        100.0 * (self.level as f32 + 1.0)
    }

    /// Adds XP and returns true if leveled up
    pub fn add_xp(&mut self, amount: f32) -> bool {
        self.current_xp += amount;

        // Curve: Level 1 needs 100, Level 2 needs 200, etc.
        let xp_needed = self.xp_for_next_level();

        if self.current_xp >= xp_needed {
            self.current_xp -= xp_needed;
            self.level += 1;
            return true;
        }
        false
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct WeaponExpertise {
    pub melee: MasteryTrack,
    pub ranged: MasteryTrack,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CooldownText;

// Systems

pub fn spawn_player_npc(
    player_npc_query: Query<Entity, With<PlayerNpc>>,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    // Only spawn if not already present (simple logic for now, or maybe we want multiple?)
    // The previous logic checked `player_npc_query.is_empty()`. I'll keep that behavior.
    if !player_npc_query.is_empty() {
        return;
    }

    let soldier_handle = asset_server.load::<DynamicScene>("prefabs/player_npcs/soldier.scn.ron");
    scene_spawner.spawn_dynamic(soldier_handle);

    // let soldier_handle = asset_server.load::<DynamicScene>("prefabs/player_npcs/ranged.scn.ron");
    // scene_spawner.spawn_dynamic(soldier_handle);
}

pub fn player_npc_decision_logic(
    mut player_npc_query: Query<
        (
            Entity,
            &mut Intent,
            &mut Target,
            &Transform,
            Option<&Children>,
        ),
        With<PlayerNpc>,
    >,
    weapon_query: Query<&ItemAttackRange, With<Weapon>>,
    monster_query: Query<(Entity, &SpawnIndex, &Transform), With<Monster>>,
    portal_tracker: Res<PortalSpawnTracker>,
) {
    let current_spawn_count = portal_tracker.0;

    for (_npc_entity, mut intent, mut target_comp, npc_transform, children) in
        player_npc_query.iter_mut()
    {
        // 1. Calculate Effective Range
        let mut max_range = 0.0;
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(range) = weapon_query.get(child) {
                    if range.0 > max_range {
                        max_range = range.0;
                    }
                }
            }
        }

        let mut target_valid = false;
        if let Some(target) = target_comp.0 {
            if monster_query.get(target).is_ok() {
                target_valid = true;
            }
        }

        if !target_valid {
            target_comp.0 = monster_query
                .iter()
                .max_by_key(|(_, index, _)| current_spawn_count.wrapping_sub(index.0))
                .map(|(e, _, _)| e);
        }

        // Decision logic based on target
        let Some(target_entity) = target_comp.0 else {
            *intent = Intent::Idle;
            continue;
        };

        if let Ok((_, _, target_transform)) = monster_query.get(target_entity) {
            let distance = npc_transform
                .translation
                .distance(target_transform.translation);

            if distance <= max_range {
                *intent = Intent::Attack(target_entity);
            } else {
                *intent = Intent::MoveTo(target_transform.translation);
            }
        } else {
            // Target dead or gone
            info!("setting intent to idle");
            *intent = Intent::Idle;
        }
    }
}

pub fn player_npc_movement_logic(
    time: Res<Time>,
    mut player_npc_query: Query<(&mut Transform, &Intent, &MovementSpeed), With<PlayerNpc>>,
) {
    for (mut transform, intent, speed) in player_npc_query.iter_mut() {
        if let Intent::MoveTo(target_pos) = intent {
            let dir = (*target_pos - transform.translation).normalize_or_zero();
            transform.translation += dir * speed.0 * time.delta_secs();
        }
    }
}

pub fn melee_attack_emit(
    mut player_npc_query: Query<
        (Entity, &Intent, &Children, &mut WeaponExpertise),
        With<PlayerNpc>,
    >,
    mut weapon_query: Query<
        (
            &mut WeaponCooldown,
            &ItemAttackRange,
            &WeaponExpertiseXp,
        ),
        (With<Weapon>, With<Melee>),
    >,
    mut melee_hit_events: MessageWriter<MeleeHitMessage>,
) {
    for (_npc_entity, intent, children, mut proficiency) in player_npc_query.iter_mut() {
        let Intent::Attack(target_entity) = intent else {
            continue;
        };

        for child in children.iter() {
            let Ok((mut cooldown, _range, xp_reward)) = weapon_query.get_mut(child) else {
                continue;
            };

            if !cooldown.timer.is_finished() {
                continue;
            }

            melee_hit_events.write(MeleeHitMessage {
                attacker: child, // Pass the weapon entity
                target: *target_entity,
            });

            // add XP for weapon proficiency
            proficiency.melee.add_xp(xp_reward.0);

            cooldown.timer.reset();
        }
    }
}

pub fn ranged_attack_logic(
    mut commands: Commands,
    // time: Res<Time>, // Removed
    mut player_npc_query: Query<
        (
            Entity,
            &Transform,
            &Intent,
            &Children,
            &mut WeaponExpertise,
        ),
        With<PlayerNpc>,
    >,
    mut weapon_query: Query<
        (
            &mut WeaponCooldown,
            &ItemAttackRange,
            &ItemProjectileStats,
            &WeaponExpertiseXp,
        ),
        (With<Weapon>, With<Ranged>),
    >,
    monster_query: Query<&Transform, With<Monster>>,
) {
    for (npc_entity, npc_tf, intent, children, mut proficiency) in player_npc_query.iter_mut() {
        let Intent::Attack(target_entity) = intent else {
            continue;
        };

        let Ok(target_tf) = monster_query.get(*target_entity) else {
            continue;
        };

        for child in children.iter() {
            if let Ok((mut cooldown, _range, proj_stats, xp_reward)) = weapon_query.get_mut(child) {
                if cooldown.timer.is_finished() {
                    let direction =
                        (target_tf.translation - npc_tf.translation).normalize_or_zero();

                    // 1. Add XP
                    proficiency.ranged.add_xp(xp_reward.0);

                    // Spawn Projectile
                    commands.spawn((
                        Sprite {
                            color: Color::srgb(1.0, 1.0, 0.0), // Yellow
                            custom_size: Some(Vec2::new(8.0, 8.0)),
                            ..default()
                        },
                        Transform::from_translation(npc_tf.translation),
                        Projectile {
                            velocity: direction * proj_stats.speed,
                            lifetime: Timer::from_seconds(proj_stats.lifetime, TimerMode::Once),
                            source: npc_entity,
                            weapon: child,
                        },
                    ));

                    cooldown.timer.reset();
                }
            }
        }
    }
}

pub fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectile_query: Query<(Entity, &mut Transform, &mut Projectile)>,
) {
    for (entity, mut transform, mut projectile) in projectile_query.iter_mut() {
        projectile.lifetime.tick(time.delta());
        if projectile.lifetime.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation += projectile.velocity * time.delta_secs();
    }
}

pub fn projectile_collision(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &Projectile)>,
    monster_query: Query<(Entity, &Transform), With<Monster>>,
    mut collision_events: MessageWriter<ProjectileCollisionMessage>,
) {
    for (proj_entity, proj_transform, projectile) in projectile_query.iter() {
        let mut hit = false;
        for (monster_entity, monster_transform) in monster_query.iter() {
            let distance = proj_transform
                .translation
                .distance(monster_transform.translation);
            // Monster size is 24, Projectile 8. Radius approx 12 + 4 = 16. Use 20 for buffer.
            if distance < 20.0 {
                collision_events.write(ProjectileCollisionMessage {
                    projectile: proj_entity,
                    source: projectile.source,
                    target: monster_entity,
                });
                hit = true;
                break;
            }
        }

        if hit {
            commands
                .entity(proj_entity)
                .remove::<Transform>()
                .remove::<Sprite>()
                .insert(MarkedForCleanUp {
                    despawn_timer: Timer::new(Duration::from_secs(60), TimerMode::Once),
                });
        }
    }
}

pub fn resolve_melee_base_damage(
    mut messages: MessageReader<MeleeHitMessage>,
    weapon_query: Query<(&BaseDamage, &ChildOf), With<Weapon>>,
) -> Vec<MeleeDamageContext> {
    let mut contexts = Vec::new();
    for msg in messages.read() {
        if let Ok((damage, parent)) = weapon_query.get(msg.attacker) {
            contexts.push(MeleeDamageContext {
                source: parent.get(),
                target: msg.target,
                current_value: damage.0,
            });
        }
    }
    contexts
}

pub fn apply_melee_damage(
    In(contexts): In<Vec<MeleeDamageContext>>,
    mut monster_query: Query<&mut monsters::Health, With<Monster>>,
) {
    for ctx in contexts {
        if let Ok(mut health) = monster_query.get_mut(ctx.target) {
            health.current -= ctx.current_value.round();
            debug!(
                "Unit {:?} took {} damage from melee source {:?}",
                ctx.target, ctx.current_value, ctx.source
            );
        }
    }
}

pub fn resolve_projectile_base_damage(
    mut messages: MessageReader<ProjectileCollisionMessage>,
    projectile_query: Query<&Projectile>,
    base_damage: Query<&BaseDamage, With<Ranged>>,
) -> Vec<ProjectileDamageContext> {
    let mut contexts = Vec::new();
    for msg in messages.read() {
        if let Ok(projectile) = projectile_query.get(msg.projectile) {
            let base_damage = base_damage
                .get(projectile.weapon)
                .expect("projectile must be paired to weapon");
            contexts.push(ProjectileDamageContext {
                weapons: projectile.weapon,
                source: msg.projectile,
                target: msg.target,
                current_value: base_damage.0,
            });
        }
    }
    contexts
}

pub fn apply_projectile_damage(
    In(contexts): In<Vec<ProjectileDamageContext>>,
    mut monster_query: Query<&mut monsters::Health, With<Monster>>,
) {
    for ctx in contexts {
        if let Ok(mut health) = monster_query.get_mut(ctx.target) {
            health.current -= ctx.current_value.round();
            debug!(
                "Unit {:?} took {} damage from projectile {:?} fired by {:?}",
                ctx.target, ctx.current_value, ctx.source, ctx.weapons
            );
        }
    }
}

pub fn update_cooldown_text(
    player_npc_query: Query<(Entity, &Children), With<PlayerNpc>>,
    weapon_query: Query<&WeaponCooldown, With<Weapon>>,
    mut text_query: Query<&mut Text2d, With<CooldownText>>,
) {
    for (_npc, children) in player_npc_query.iter() {
        // Find WeaponCooldown
        let Some(rem_secs) = children.iter().find_map(|child| {
            weapon_query
                .get(child)
                .ok()
                .map(|cooldown| cooldown.timer.remaining_secs())
        }) else {
            continue;
        };

        // Find CooldownText and update it
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = format!("{:.1}", rem_secs);
            }
        }
    }
}

#[cfg(test)]
mod tests_logic;
