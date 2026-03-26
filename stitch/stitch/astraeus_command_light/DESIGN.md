# Design System Specification: The Ethereal Orchestrator

## 1. Overview & Creative North Star
**The Creative North Star: "The Digital Curator"**

This design system moves away from the dense, cluttered aesthetics of traditional command interfaces toward an "Editorial Aerospace" feel. The goal is to present complex mission data with the clarity of a high-end architectural magazine. By utilizing vast amounts of white space, intentional asymmetry, and glassmorphic layering, we transform "data monitoring" into "curated observation."

### Breaking the Template
To avoid a "generic SaaS" look, this system rejects standard grid-row containers. Instead, use:
- **Intentional Asymmetry:** Align primary technical data to a strict left margin, while allowing secondary editorial insights to float with more generous, irregular right-side padding.
- **Overlapping Elements:** Allow glassmorphic cards to slightly overlap section headers to create a sense of physical depth and "stacked" intelligence.
- **High-Contrast Typography Scales:** Use dramatic shifts between massive `display-lg` technical metrics and whisper-quiet `label-sm` metadata.

---

## 2. Color & Surface Architecture

The palette is rooted in high-luminosity neutrals, punctuated by a singular, high-energy "Sky Blue" to denote primary actions and mission-critical status.

### The "No-Line" Rule
**Explicit Instruction:** Do not use 1px solid borders to section off content. Physical boundaries are an antiquated UI relic. Instead, define zones using:
- **Background Shifts:** A `surface-container-low` section sitting against a `background` base.
- **Tonal Transitions:** Using the subtle difference between `surface-container` and `surface-container-highest` to signal a change in context.

### Surface Hierarchy & Nesting
Treat the UI as a series of stacked sheets of fine vellum.
- **Base Layer:** `background` (#f7f9fb)
- **Content Zones:** `surface-container-low` (#f2f4f6)
- **Interactive Cards:** `surface-container-lowest` (#ffffff)
- **Global Navigation/Overlays:** Glassmorphic (`rgba(255, 255, 255, 0.7)`) with a `20px` backdrop-blur.

### The "Glass & Gradient" Rule
To inject "soul" into the system, primary CTAs should not be flat. Use a subtle linear gradient:
- **Primary Gradient:** `primary_container` (#21bcff) to `primary` (#00658d) at a 135-degree angle. This creates a "gemstone" effect that feels premium and tactile.

---

## 3. Typography: Editorial Authority

The system pairs the Swiss-inspired precision of **Inter** with the utilitarian, technical soul of **JetBrains Mono**.

*   **Inter (UI & Narrative):** Used for all headings and body copy to maintain a sophisticated, approachable tone.
*   **JetBrains Mono (Technical Data):** Reserved exclusively for coordinates, timestamps, telemetry, and code-based outputs.

### Typographic Hierarchy
- **Display (Large/Medium):** Use for "Hero Metrics" (e.g., Orbital Velocity). Tracking: -0.02em.
- **Headlines:** Use for section titles. Ensure `on-surface` (#191c1e) color is used for maximum legibility against the light background.
- **Labels (Space Grotesk/JetBrains Mono):** All caps, +0.05em tracking. Use these for technical metadata to provide a "Mission Control" aesthetic.

---

## 4. Elevation & Depth: Tonal Layering

We achieve depth through light physics rather than drop shadows.

### The Layering Principle
Instead of a shadow, place a `surface-container-lowest` (#ffffff) card on top of a `surface-container-low` (#f2f4f6) background. This "High-Key" contrast provides a cleaner, more modern lift.

### Ambient Shadows
Where floating elements (Modals/Popovers) are required:
- **Color:** Use a tinted shadow: `rgba(0, 101, 141, 0.08)` (a 8% opacity version of the `primary` color).
- **Blur:** Minimum `40px` spread. Shadows should feel like a soft atmospheric glow, not a dark smudge.

### The "Ghost Border" Fallback
If a border is legally or functionally required:
- **Token:** `outline-variant` (#bdc8d1).
- **Opacity:** Apply at **15% opacity**. It should be felt, not seen.

---

## 5. Components & Interaction Patterns

### Buttons
- **Primary:** Gradient fill (`primary_container` to `primary`). White text. `0.375rem` (md) corner radius.
- **Secondary:** Surface-tinted. No border. `surface-container-high` background with `primary` text.
- **Tertiary:** Text only. Use `label-md` styling with an underline that appears only on hover.

### Input Fields
- **Styling:** Forgo the box. Use a "Soft Inset" look: `surface-container-highest` background with a subtle bottom-weighted shadow.
- **Focus State:** Transition the background to `surface-container-lowest` and apply a `2px` `primary` glow.

### Cards & Lists (The "No-Divider" Mandate)
- **Forbid:** Do not use horizontal rules (`<hr>`) or 1px dividers.
- **The Solution:** Use `spacing-6` (2rem) of vertical white space or a subtle toggle between `surface-container-low` and `background` to separate list items.

### Special Component: The Telemetry Strip
A custom component for this system. A narrow, full-width glassmorphic bar (`surface-elevated`) that sits at the very bottom or top of the viewport, containing scrolling `JetBrains Mono` data in `label-sm`. This reinforces the "Mission Orchestrator" feel.

---

## 6. Do's and Don'ts

### Do:
- **Embrace White Space:** If a layout feels "empty," add 20% more padding. This system thrives on breathability.
- **Use Subtlety:** Use `surface-dim` for inactive or disabled states to maintain the airy aesthetic.
- **Align to Data:** Use the spacing scale (e.g., `3.5rem`, `7rem`) to create rigid vertical rhythms for technical data.

### Don't:
- **No Pure Black:** Never use #000000. Use `on-surface` (#191c1e) for text to keep the contrast high but the "vibe" sophisticated.
- **No Heavy Borders:** 1px borders are strictly prohibited for layout containment. 
- **No Standard Shadows:** Avoid the default "CSS Drop Shadow" look. If it doesn't look like diffused light hitting glass, it doesn't belong.

---

## 7. Design Tokens Summary

| Role | Token | Value |
| :--- | :--- | :--- |
| **Primary Base** | `primary` | #00658d |
| **Accent Glow** | `primary_container` | #21bcff |
| **Base Canvas** | `background` | #f7f9fb |
| **Elevated Surface** | `surface_container_lowest`| #ffffff |
| **Glass Layer** | `surface_elevated` | rgba(255, 255, 255, 0.7) |
| **Primary Text** | `on_surface` | #191c1e |
| **Secondary Text** | `on_surface_variant` | #3d4850 |
| **Corner Radius** | `md` | 0.375rem |
| **Soft Spacing** | `6` | 2rem |