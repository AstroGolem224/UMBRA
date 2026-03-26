# High-End Editorial: The Mission Orchestrator Design System

## 1. Overview & Creative North Star: "The Digital Astral"
This design system is not a collection of boxes; it is a sophisticated, layered lens into complex data. Our Creative North Star is **"The Digital Astral"**—a concept that treats the dashboard as a high-fidelity instrument cluster floating in a deep-space vacuum. 

To move beyond the "SaaS template" look, we leverage **intentional asymmetry** and **tonal depth**. Large-scale data visualizations should bleed to the edges, while control modules utilize glassmorphism to feel like physical overlays. We break the rigid grid by using oversized display typography against micro-scale technical data, creating a rhythmic tension that signals both authority and precision.

---

## 2. Colors: Tonal Depth & Luminous Accents

The palette is rooted in the "Deep Charcoal" void, using light not as a fill, but as a medium.

### Core Palette
- **Background (`surface-dim`):** `#111417` — The foundational vacuum.
- **Brand Primary (`primary`):** `#00cdff` (Sky Blue) — Used for critical paths and active states.
- **Brand Accent (`secondary`):** `#bdc2ff` (Indigo) — Reserved for secondary telemetry and data categorization.
- **Brand Tertiary (`tertiary`):** `#ff6600` (Orange) — An additional accent for highlights, badges, or decorative elements.
- **Surface Elevated:** `rgba(255, 255, 255, 0.05)` with `backdrop-filter: blur(12px)`.

### The "No-Line" Rule
Standard 1px solid borders for sectioning are strictly prohibited. Layout boundaries must be defined through **Background Color Shifts**. To separate a sidebar from a main feed, transition from `surface-container-lowest` to `surface-container-low`. The eye should perceive the change in depth, not a drawn line.

### Surface Hierarchy & Nesting
Treat the UI as a series of nested physical layers:
1.  **Level 0 (Base):** `surface-dim` (#111417).
2.  **Level 1 (Sections):** `surface-container-low`.
3.  **Level 2 (Cards/Modules):** `surface-container` or `surface-variant`.
4.  **Level 3 (Popovers/Modals):** Glassmorphic surfaces with `backdrop-blur`.

### The "Glass & Gradient" Rule
For primary CTAs or critical status headers, use a subtle linear gradient: `primary` (#00cdff) to `primary_container` (#21bcff). This adds "soul" and a sense of internal illumination that flat hex codes cannot achieve.

---

## 3. Typography: The Editorial Tech-Stack

We use a dual-typeface system to distinguish between human-readable intent and machine-generated precision.

*   **Inter (Sans-Serif):** The UI Voice. Used for navigation, headers, and instructional text. It provides a clean, neutral "Editorial" feel.
*   **JetBrains Mono (Monospace):** The Technical Voice. Used for all data points, timestamps, and coordinates. This creates a clear mental model: *If it's monospaced, it’s a live value.*

### Key Scales
- **Display LG (Inter):** 3.5rem / Tracking -0.02em. Use for hero metrics (e.g., "99.9% Uptime").
- **Label MD (Space Grotesk/Mono):** 0.75rem / All Caps / Tracking 0.05em. Use for technical metadata and "over-the-shoulder" technical labels.
- **Body MD (Inter):** 0.875rem. The workhorse for all descriptions.

---

## 4. Elevation & Depth: Tonal Layering

We reject traditional drop shadows in favor of **Ambient Luminosity**.

- **The Layering Principle:** Depth is achieved by stacking. A `surface-container-highest` card should sit atop a `surface-container-low` background. The contrast in value creates the lift.
- **Ambient Shadows:** For floating glass panels, use a shadow with a massive blur (40px+) and ultra-low opacity (4-6%). The shadow color should be a tinted blue-black, never pure black.
- **The Ghost Border:** For accessibility on interactive elements, use `outline_variant` at **15% opacity**. This creates a "glint" on the edge of the glass rather than a heavy container wall.
- **Glassmorphism:** Use `surface-variant` with 40% opacity and a `12px` blur for any element that "floats" over live data streams.

---

## 5. Components: The Instrument Cluster

### Buttons
- **Primary:** Gradient fill (`primary` to `primary_container`). White text. No border.
- **Secondary:** Ghost style. `outline_variant` (20% opacity) border. Inter Medium.
- **Tertiary:** Text only. `primary` color. Used for low-priority actions in dense data tables.

### Input Fields
- **Base State:** `surface-container-highest` background. No border.
- **Focus State:** 1px "Ghost Border" using `primary`. Subtle inner glow (`box-shadow: 0 0 8px rgba(56, 189, 248, 0.2)`).

### Data Chips
- Small, 4px radius. Use `secondary_container` for background and `on_secondary_container` for text. These should feel like small "LED indicators" on the dashboard.

### Cards & Lists
- **The Divider Ban:** Never use a horizontal rule `<hr>`. Use a `1.75rem (spacing.8)` vertical gap or a slight shift from `surface-container` to `surface-container-low`.

### Unique Component: The "Telemetric Gauge"
A custom component for mission control. Use `JetBrains Mono` for the value and a thin, 2px stroke circular progress bar using the `primary` to `secondary` gradient.

---

## 6. Do’s and Don’ts

### Do
*   **Do** embrace negative space. In a mission control environment, white space is "processing time" for the user's brain.
*   **Do** use `JetBrains Mono` for any number that changes dynamically.
*   **Do** use high-contrast semantic colors (`Success`, `Warning`, `Error`) sparingly. They should act as "alarms" in a sea of monochrome charcoal.

### Don't
*   **Don't** use 100% opaque white borders. They shatter the "Digital Astral" immersion.
*   **Don't** use standard "drop shadows" with 0 blur. 
*   **Don't** use rounded corners larger than `1.5rem`. This is a professional orchestrator, not a consumer social app. Keep it "Sophisticated Technical."
*   **Don't** mix the typefaces. Inter is for the human; Mono is for the machine.