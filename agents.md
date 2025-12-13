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
    * **Definitions:** Use `.ron` files for base stats/composition.
    * **Scaling:** Use Rust `GrowthStrategy` logic for incremental scaling.
* **Component Composition:** Use granular components (`Melee`, `AttackRange`, `Damage`) instead of monolithic objects (`Sword`).
* **Observer Pattern:** Use Observers for UI interactions and entity lifecycle hooks.

**Code Quality:**
* **Error Handling:** Use `expect("context")` for unrecoverable errors. Avoid naked `unwrap()`.
* **No Unsafe:** `unsafe` code is strictly forbidden.
* **Intentional Documentation:** Explain the *why*, not just the *how*.

## 4. The Core Game Loop Architecture
Gameplay logic must follow this 4-phase execution flow (enforced via System Sets):

1.  **Phase 1: Decision (Intent)**
    * Entities decide *what* to do (e.g., `npc_decision_system` sets `Intent::Attack`).
2.  **Phase 2: Execution (Resolution)**
    * Systems resolve `Intent` into **Messages** (e.g., `melee_execution_system` emits `DamageMessage`).
3.  **Phase 3: Application (Effects)**
    * Systems consume Messages to mutate state (e.g., `damage_application_system` applies mitigation and reduces `Health`).
4.  **Phase 4: Cleanup & Lifecycle**
    * Handle consequences (e.g., `death_system` handles 0 HP entities, rewards, and despawning).

## 5. Project Structure

The project is organized as a Cargo workspace with the following crates:
* **`common`**: Shared types, `GameState`, and global **Messages** (Events).
* **`game_core`**: Main glue crate, global setup.
* **`enemy`**: Enemy logic and configurations.
* **`player_npcs`**: Player ally logic (Soldiers, Heroes).
* **`portal`**: Core portal mechanics, upgrades, spawning.
* **`items`**: Itemization components (`Melee`, `Ranged`, `Armor`).
* **`ui`**: User Interface systems.
* **`wallet`**: Resource/currency management.
* **`assets`**: Asset loading and management.
* **`src/` (Root)**: Main binary entry point.

# Pre-commit
* Run `cargo +nightly fmt`.
* Run `cargo check` and `cargo clippy`.
* **Architecture Check:** Ensure new systems are added to the correct Phase/SystemSet.
