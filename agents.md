# Agent Configuration: Lead Developer

## 1. Game Design Overview
**Title/Concept:** Sci-Fi Fantasy Incremental Idle Game (2D)

**Core Mechanics:**
* **The Loop:** A central Portal spawns waves of monsters. Heroes automatically engage and battle these monsters.
* **Player Interaction:** The player acts as the manager—upgrading the Portal (to increase enemy difficulty/rewards) and managing/upgrading Heroes.
* **Progression:** There is no "Game Over" or "Win" state. The goal is infinite growth and optimization of numbers and efficiency.

## 2. Identity & Objectives
**Role:** Lead Game Developer
**Primary Goal:** Build the game in Bevy 0.17 and Rust, managing the entire lifecycle from setup to polish.

**Key Responsibilities:**
* **Full-Stack Engine Work:** Implement gameplay mechanics, UI, physics, and asset management.
* **Incremental Development:** Focus on specific, small, testable iterations (e.g., "Setup Project" -> "Core Loop" -> "Movement").
* **Architecture Ownership:** Maintain a clean, modular structure as the project grows.

## 3. Technical Constraints & Style Guide

**Architecture & Patterns:**
* **Reactive ECS (Message-Based):** Avoid direct mutation across domains. Logic should be "Fire and Forget" where possible.
    * *Pattern:* System A emits a `Message`; System B consumes it and mutates state.
* **Data-Driven Hybrid Approach:**
    * **Definitions:** Use `.ron` files for base stats/composition (e.g., `portal.scn.ron`, `soldier.scn.ron`).
    * **Scaling:** Use Rust `GrowthStrategy` logic for incremental scaling.
* **Component Composition:** Use granular components (`Melee`, `AttackRange`, `Damage`) instead of monolithic objects (`Sword`).
* **Observer Pattern:** Use Observers for UI interactions (e.g., `.observe(on_click)`) and entity lifecycle hooks.

**Code Quality:**
* **Error Handling:** Use `expect("context")` for unrecoverable errors. Avoid naked `unwrap()`.
* **No Unsafe:** `unsafe` code is strictly forbidden.
* **Intentional Documentation:** Explain the *why*, not just the *how*.

## 4. The Core Game Loop Architecture
Gameplay logic must follow this 4-phase execution flow (enforced via System Sets):

1.  **Phase 1: Decision (Intent)**
    * Entities decide *what* to do (e.g., `player_npc_decision_logic` sets `Intent::Attack`).
2.  **Phase 2: Execution (Resolution)**
    * Systems resolve `Intent` into **Messages** (e.g., `melee_attack_emit` emits `DamageMessage`).
3.  **Phase 3: Application (Effects)**
    * Systems consume Messages to mutate state (e.g., `apply_damage_logic` reads `DamageMessage` to reduce `Health`).
4.  **Phase 4: Cleanup & Lifecycle**
    * Handle consequences (e.g., `manage_enemy_lifecycle` handles 0 HP entities, rewards, and despawning).

## 5. Project Structure

The project is organized as a Cargo workspace with the following crates:
* **`common`**: Shared types, `GameState`, and global **Messages** (e.g., `DamageMessage`, `EnemyKilled`).
* **`game_core`**: Main glue crate, global setup, and plugin aggregation.
* **`enemy`**: Enemy logic, `EnemyConfig`, and lifecycle management.
* **`player_npcs`**: Player ally logic (Soldiers, Heroes, Turrets), including AI and combat.
* **`player_npcs_ui`**: UI systems for Soldier/Hero interactions.
* **`portal`**: Core portal mechanics, upgrade logic, and spawn timers.
* **`monster_factory`**: Enemy spawning infrastructure and stat hydration.
* **`items`**: Itemization components (`Melee`, `Ranged`, `Armor`, `ProjectileStats`).
* **`ui`**: General User Interface systems (e.g., Portal Panel).
* **`wallet`**: Resource/currency management (`void_shards`).
* **`assets`**: Asset loading and management.
* **`src/` (Root)**: Main binary entry point.

# Pre-commit
* Run `cargo +nightly fmt`.
* Run `cargo check` and `cargo clippy`.
* **Architecture Check:** Ensure new systems are added to the correct Phase/SystemSet.
