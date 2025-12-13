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
* **Strict ECS Separation:** Data lives in Components/Resources. Logic lives in Systems. Do not bind game logic to struct methods (keep the approach functional/data-oriented).
* **Modular Design:** Every major feature (Portal Logic, Hero AI, UI, Stats) must be organized into its own `Plugin`.
* **Data-Driven Configuration:** Use `.ron` files for gameplay variables to allow balancing without recompilation. Specific extensions help the asset loader distinguish types: `.portal.ron` (mechanics), `.enemy.ron` (enemy stats), and `.soldier.ron` (NPC stats).
* **Observer Pattern:** Leverage Bevy 0.17+ Observers for event-driven logic (e.g., UI interactions, entity spawning/despawning hooks) to decouple systems.

**Code Quality:**
* **Error Handling:** Use `expect("context")` for unrecoverable errors to aid debugging. Avoid naked `unwrap()`.
* **No Unsafe:** `unsafe` code is strictly forbidden unless absolutely unavoidable.
* **Intentional Documentation:** Comments must explain the *reasoning* ("why we chose this specific approach") rather than just describing what the code does.

## 4. Project Structure

The project is organized as a Cargo workspace with the following crates:
* **`common`**: Contains shared types, states (`GameState`), and messages used across the entire project.
* **`game_core`**: Acts as the main glue crate, aggregating other crates and handling global setup.
* **`enemy`**: Handles enemy logic, systems, and configurations.
* **`player_npcs`**: Manages player allies (Soldiers, Heroes, etc.) and their logic.
* **`portal`**: Implements the core portal mechanics, upgrades, and stats.
* **`items`**: Manages itemization components and equipment stats (e.g. `Melee`, `Ranged`, `Armor`).
* **`ui`**: Dedicated crate for User Interface systems.
* **`wallet`**: Manages player resources/currency (Void Shards).
* **`assets`**: Handles asset loading, configurations, and management.
* **`src/` (Root)**: Contains the main binary entry point.

# Pre-commit
Always run `cargo +nightly fmt` before pushing any changes to ensure code is well formatted.
Always run `cargo check` and check for lints.
Always run `cargo clippy` to check for lints.
