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
* **Reactive ECS (Message-Based):**
    * Use Bevy 0.17+ **Messages** (`#[derive(Message)]`, `app.add_message::<T>()`) for event handling.
    * **Pattern:** System A emits a `Message`; System B consumes it and mutates state. Logic should be "Fire and Forget" where possible.
* **Data-Driven Hybrid Approach:**
    * **Definitions:** Use `.ron` files for base stats/composition (loaded via `bevy_common_assets`).
    * **Scaling:** Use Rust `GrowthStrategy` logic (in `common` crate) for incremental scaling.
* **Component Composition:** Use granular components (`Melee`, `AttackRange`, `Damage`) instead of monolithic objects.
* **Observer Pattern:** Use Bevy Observers (`.observe(on_click)`, `On<Trigger>`) for UI interactions and entity lifecycle hooks.

**Code Quality:**
* **Error Handling:** Use `expect("context")` for unrecoverable errors. Avoid naked `unwrap()`.
* **No Unsafe:** `unsafe` code is strictly forbidden.
* **Intentional Documentation:** Explain the *why*, not just the *how*.

## 4. The Core Game Loop Architecture
Gameplay logic follows a 4-phase execution flow (implemented via system chaining and ordering):

1.  **Phase 1: Decision (Intent)**
    * Systems read state and set an `Intent` component (e.g., `npc_decision_logic` sets `Intent::Attack`).
2.  **Phase 2: Execution (Resolution)**
    * Systems read `Intent` and resolve it into **Messages** (e.g., `melee_attack_emit` emits `DamageMessage`).
3.  **Phase 3: Application (Effects)**
    * Systems consume Messages to mutate state (e.g., `apply_damage_logic` reduces `Health`).
4.  **Phase 4: Cleanup & Lifecycle**
    * Systems handle consequences (e.g., `handle_dying_enemies` manages death transitions).

*Note: While explicit `SystemSet` usage is encouraged, current implementation relies on explicit `.chain()` ordering or Bevy's default parallel execution where dependency is implicitly managed via Message readers/writers.*

## 5. Project Structure

The project is organized as a Cargo workspace with the following crates:
* **`common`**: Shared types, `GameState`, and global **Messages**.
* **`game_core`**: Main glue crate, global setup, plugin orchestration.
* **`enemy`**: Enemy logic, spawning, and configurations.
* **`player_npcs`**: Player ally logic (Soldiers, Heroes).
* **`portal`**: Core portal mechanics, upgrades, spawning logic.
* **`items`**: Itemization components (`Melee`, `Ranged`, `Armor`).
* **`ui`**: User Interface systems (Wallet, Portal Panel).
* **`wallet`**: Resource/currency management (`Void Shards`).
* **`assets`**: Asset loading infrastructure.
* **`src/` (Root)**: Main binary entry point.

# Pre-commit
* Run `cargo +nightly fmt`.
* Run `cargo check` and `cargo clippy`.
* **Architecture Check:** Ensure new systems follow the Decision -> Message -> Effect flow.
