# Visual Quality Bar (2026)

**Purpose:** This document sets the **minimum acceptable** visual quality, readability, and performance targets for the game. These are **non-negotiable gates** for any public demo, vertical slice, or external review.

The bar assumes **high hardware costs in 2026** and a player base that includes mid‑range and older GPUs. The art direction must remain **cohesive and readable** while staying **performant**.

---

## 1) Readability (Must Pass)

### Strategic Zoom Readability

- Unit **class**, **ownership**, and **health** must be clear within **1–2 seconds** at strategic zoom.
- Distinct silhouettes are required for **infantry, vehicles, mechs, air**, and **buildings**.
- Faction color identity must remain readable against terrain and VFX.

### Visual Hierarchy

- **Primary:** Units, projectiles, attack direction
- **Secondary:** Health/selection, status effects
- **Tertiary:** Ambient effects, environment detail

If primary elements are ever obscured by secondary/tertiary elements, the build **fails** the bar.

### Mandatory Feedback Elements

- Health bars (units/buildings)
- Selection rings/highlights
- Damage feedback (hit flash or impact VFX)
- Command confirmation (sound + small UI ping)

---

## 2) Cohesive Visual Identity (Must Pass)

### Style Guide Requirements

- A single page defining:
  - **Faction palettes** (primary + secondary + outline + VFX accent)
  - **Silhouette rules** by unit class
  - **Scale rules** (infantry vs vehicle vs mech vs building)
  - **VFX language** (damage types and ability categories)

### Faction Differentiation

- At strategic zoom, factions must be distinguishable **without reading UI**.
- Each faction must have at least **one iconic silhouette** visible at mid‑game scale.

---

## 3) Performance Targets (2026 Reality Check)

### Baseline Target

- **60 FPS** at 1080p on mid‑range 2020–2022 GPUs.
- **Stable 30 FPS** on older integrated or entry‑level discrete GPUs.

### Visual Budget Priorities

- **Unit readability > VFX density > terrain detail**
- VFX must degrade gracefully on low settings.
- UI and health bars must never be tied to expensive post‑processing.

### Scaling Requirements

- Texture resolution tiers (low/med/high)
- VFX density tiers (low/med/high)
- Optional shadows and post‑processing

---

## 4) Art Production Minimums

### Units

- Minimum animation set:
  - Idle, Move, Attack, Hit, Death
- Each unit must read clearly at strategic zoom.

### Buildings

- Must indicate production role via silhouette and iconography.
- Construction state must be visually distinct from completed state.

### Terrain

- Terrain should never visually overpower units.
- High‑contrast terrain motifs must be avoided in combat zones.

---

## 5) UX/Visual Acceptance Tests

A build **fails** if any of the following are true:

- A new player cannot identify **who is winning a fight** within 5 seconds.
- Unit classes are confused at strategic zoom.
- Health/selection are lost in VFX or terrain contrast.
- VFX obscure target clarity or create false reads.

---

## 6) Delivery Gates

### Gate A — Visual Readability (Phase 2.7)

- All mandatory feedback elements present.
- Strategic zoom readability proven in 2–3 test matches.

### Gate B — Visual Identity (Vertical Slice)

- One faction meets full style guide requirements.
- Distinct silhouettes + faction palette implemented across all units/buildings.

---

## Summary

The project succeeds visually only if **readability and cohesion** come before feature expansion. This bar is designed to keep the game playable and marketable under 2026 hardware constraints.
