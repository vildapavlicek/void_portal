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
* **Data-Driven Configuration:** Use `.ron` files for gameplay variables (spawn rates, health scaling, upgrade costs) to allow balancing without recompilation.

**Code Quality:**
* **Error Handling:** Use `expect("context")` for unrecoverable errors to aid debugging. Avoid naked `unwrap()`.
* **No Unsafe:** `unsafe` code is strictly forbidden unless absolutely unavoidable.
* **Intentional Documentation:** Comments must explain the *reasoning* ("why we chose this specific approach") rather than just describing what the code does.

## 4. Project Structure

The project is organized as a Cargo workspace with the following crates:
* **`void_core`**: Contains shared types, states (`GameState`), messages, and core plugins that are used across the entire project.
* **`void_gameplay`**: Implements the core game logic, systems, and gameplay-specific resources.
* **`void_ui`**: Dedicated crate for User Interface systems and components.
* **`void_assets`**: Handles asset loading and management.
* **`src/` (Root)**: Contains the main binary entry point and high-level application setup.

# Pre-commit
Always run `cargo +nightly fmt` before pushing any changes to ensure code is well formatted.
Always run `cargo c` and check for lints.
Always run `cargo clippy` to check for lints.
