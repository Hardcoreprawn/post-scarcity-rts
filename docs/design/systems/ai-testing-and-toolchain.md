# AI-Driven QA, UX Feedback & Asset Toolchain

## 1. AI Gameplay Evaluation & UX Feedback Loop

To ensure our game remains balanced and accessible as we add complexity, we will implement an automated "AI Evaluator" loop. This goes beyond simple unit tests by having AI agents play full matches to actionable metrics.

### 1.1 The "Gym" Environment

We will leverage the **Headless Simulation Runner** (Planned Phase 3) to create a training gym.

- **Run Mode**: `rts_runner --gym --speed=100x`
- **Scenarios**:
  - `Macrotest`: AI builds economy to X supply as fast as possible.
  - `Microtest`: AI manages a specific combat skirmish for minimal losses.
  - `GameCheck`: Full match vs another AI.

### 1.2 Feedback Metrics (The "Report Card")

Every run generates a JSON report containing:

- **Efficiency**: Resources gathered vs. Resources spent (Resource Float).
- **Responsiveness**: "Input friction" metrics. *Example: If AI issues a Build command 10 times but it fails 9 times due to "Invalid Placement", this flags a UX/Level Design issue.*
- **Pathing Friction**: Time spent waiting for path verification. High friction suggests map readability issues (UX).
- **APM Saturation**: If the Standard AI requires 300+ APM to be effective, the mechanics are too click-intensive for players.

### 1.3 UX Evolution via AI

How AI feedback changes the UI/UX:

1. **Accessibility Validation**: The AI interfaces with the game via the *Command API*, which mirrors the Player's UI commands. If the AI cannot "find" a command (e.g., missing from the tech tree logic), a player wouldn't utilize it either.
2. **Readability Proxy**: We can simulate "Fog of War" uncertainty in the AI. If the AI consistently loses units because it couldn't "see" a threat that was technically visible, visual indicators (UX) need boosting.

---

## 2. Graphics & Asset Toolchain (VS Code)

To support rapid iteration of unit models and sprites, we will evolve our toolchain from procedural Python scripts to a professional workflow integrated into VS Code.

### 2.1 Current State: Procedural Generation

- **Source**: `tools/generate_sprites.py` (Python/PIL)
- **Output**: `crates/rts_game/assets/textures/sprites/*.png`
- **Workflow**: Edit Code -> Run Script -> Check Game.

### 2.2 Future State: Data-Driven Pipeline

We will transition to a **Hot-Reloading** workflow.

#### VS Code Configuration

We will add `tasks.json` entries to automate the asset loop:

```json
{
    "label": "Watch Assets",
    "type": "shell",
    "command": "./tools/watch_assets.sh",
    "isBackground": true,
    "problemMatcher": []
}
```

#### Recommended Extensions

To edit assets directly in the IDE:

- **Luna Paint** (glenn2223.luna-paint): Edit `.png` sprites directly in VS Code tabs.
- **glTF Tools** (Cesium): Preview 3D models (if we move to 3D).

#### The "Sprite Definition" Format

Instead of hardcoding drawing commands, we will define units in `.ron` (Rusty Object Notation) files that reference raw assets:

```rust
// assets/data/units/infantry.ron
(
    name: "Infantry",
    base_sprite: "textures/raw/humanoid_base.png",
    weapon_sprite: "textures/raw/rifle_mk1.png",
    tint_mask: "textures/raw/humanoid_mask.png", // Areas to color with faction color
    animation_frames: {
        "idle": [0, 1], // grid coordinates
        "walk": [2, 3, 4, 5]
    }
)
```

### 2.3 Implementation Plan

1. **Asset Watcher**: A simple Rust tool (in `rts_tools`) that watches the `assets` folder. When a `.png` or `.ron` changes, it hot-reloads the texture in the running game.
2. **Sprite Baker**: A build-time tool that takes the `.ron` definitions and "bakes" them into the final texture atlases, replacing the runtime python script.

---

## 3. Summary of Work Items

| Phase | Feature | Description |
| :--- | :--- | :--- |
| **3 (Vertical Slice)** | **Asset Watcher** | Hot-reload textures in `rts_game` via `rts_tools`. |
| **4 (Rollout)** | **Metric Collector** | AI records resource float & error rates. |
| **5 (Advanced)** | **UX Heatmaps** | Visualize where AI (and players) click most often. |
