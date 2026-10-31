---
name: Realme Link Desktop
description: A calm, exact control panel for realme earbuds, with one yellow signal and soft glass surfaces.
colors:
  ink: "oklch(0.165 0.006 270)"
  ink-raised: "oklch(0.205 0.007 270)"
  ink-track: "oklch(0.235 0.007 270)"
  bone: "oklch(0.955 0.011 85)"
  bone-raised: "oklch(0.975 0.008 85)"
  bone-track: "oklch(0.925 0.012 85)"
  text-on-ink: "oklch(0.935 0.012 85)"
  text-on-bone: "oklch(0.2 0.01 270)"
  text-muted-on-ink: "oklch(0.7 0.012 270)"
  text-muted-on-bone: "oklch(0.5 0.012 270)"
  signal: "oklch(0.88 0.175 94)"
  signal-on-bone: "oklch(0.87 0.175 94)"
  signal-text: "oklch(0.2 0.04 90)"
  transparency: "oklch(0.8 0.1 200)"
  transparency-on-bone: "oklch(0.6 0.1 205)"
  low: "oklch(0.72 0.19 28)"
  low-on-bone: "oklch(0.6 0.2 28)"
typography:
  display:
    fontFamily: "Bricolage Grotesque Variable, Manrope Variable, sans-serif"
    fontSize: "2.25rem"
    fontWeight: 600
    lineHeight: 1
    letterSpacing: "-0.025em"
  headline:
    fontFamily: "Bricolage Grotesque Variable, Manrope Variable, sans-serif"
    fontSize: "1.25rem"
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: "-0.025em"
  numeral:
    fontFamily: "Bricolage Grotesque Variable, Manrope Variable, sans-serif"
    fontSize: "2.25rem"
    fontWeight: 500
    lineHeight: 1
    letterSpacing: "-0.025em"
  body:
    fontFamily: "Manrope Variable, system-ui, sans-serif"
    fontSize: "0.875rem"
    fontWeight: 400
    lineHeight: 1.45
  label:
    fontFamily: "Manrope Variable, system-ui, sans-serif"
    fontSize: "0.75rem"
    fontWeight: 500
    lineHeight: 1.3
rounded:
  sm: "0.6rem"
  md: "0.8rem"
  lg: "1rem"
  xl: "1.4rem"
  "2xl": "1.8rem"
  "3xl": "2.4rem"
spacing:
  xs: "4px"
  sm: "8px"
  md: "16px"
  lg: "24px"
  xl: "28px"
components:
  segmented-track:
    backgroundColor: "{colors.ink-raised}"
    rounded: "{rounded.2xl}"
    padding: "4px"
  segmented-pill-active:
    backgroundColor: "{colors.signal}"
    textColor: "{colors.signal-text}"
    rounded: "{rounded.xl}"
    height: "44px"
  switch-on:
    backgroundColor: "{colors.signal}"
    rounded: "{rounded.3xl}"
  button-primary:
    backgroundColor: "{colors.signal}"
    textColor: "{colors.signal-text}"
    rounded: "{rounded.3xl}"
    padding: "6px 14px"
  stage:
    backgroundColor: "{colors.ink}"
    rounded: "{rounded.3xl}"
---

# Design System: Realme Link Desktop

## Overview

**Creative North Star: "The Instrument Panel"**

A control surface for something you wear: precise, quiet, and legible at a glance. The 3D earbuds sit on a stage of faint concentric rings like a dial face; everything you can change lives beside it on a continuous console divided by hairlines, not stacked into identical cards. One thing is active at a time and it is the only thing that is yellow.

Components are soft and glassy rather than hard-edged: generous radii, frosted tracks, and a selected pill that lifts off its track with a real offset shadow. Motion answers actions (the pill slides, numbers settle, the EQ curve morphs) and the single authored moment is the buds assembling on connect.

**Key Characteristics:**
- One accent, used only for the active state; colour elsewhere carries meaning (teal = transparency, red = low battery).
- Ink (dark) is the default; Bone (light) follows the system. Both are tinted neutrals, never pure black or white.
- Numerals are display-weight and tabular; they read like a meter.
- Structure comes from hairline dividers and spacing, not nested containers.

## Colors

Warm-tinted neutrals with a single saturated signal. Values are OKLCH and live as CSS variables in `src/index.css`; the `.dark` class selects Ink.

### Primary
- **Signal Yellow** (oklch(0.88 0.175 94)): the selected segment, an ON switch, the primary button. Never decorative.

### Secondary
- **Transparency Teal** (oklch(0.8 0.1 200) on Ink, oklch(0.6 0.1 205) on Bone): the Transparency noise mode only.
- **Low Red** (oklch(0.72 0.19 28) on Ink, oklch(0.6 0.2 28) on Bone): battery at or below 20%.

### Neutral
- **Ink / Ink Raised / Ink Track** (oklch(0.165 0.006 270) / 0.205 / 0.235): page, raised surfaces, recessed tracks in dark mode.
- **Bone / Bone Raised / Bone Track** (oklch(0.955 0.011 85) / 0.975 / 0.925): the same three layers in light mode.
- **Bone Text** (oklch(0.935 0.012 85)) on Ink and **Ink Text** (oklch(0.2 0.01 270)) on Bone; muted text steps down to 0.7 / 0.5 lightness.

### Named Rules
**The One Signal Rule.** Yellow marks the single active choice in a group. If two yellow things are visible in one group, one is wrong.
**The Meaning-Not-Decoration Rule.** Teal and red appear only when they state something (a mode, a low battery). They are never used to add colour.

## Typography

**Display Font:** Bricolage Grotesque Variable (fallback Manrope Variable)
**Body Font:** Manrope Variable (fallback system-ui)

**Character:** Bricolage gives the device name, section titles and battery numerals a confident, slightly idiosyncratic voice; Manrope keeps everything functional and quiet.

### Hierarchy
- **Display** (600, 2.25rem, 1.0): the device name on the stage.
- **Headline** (600, 1.25rem, 1.2): section titles (Noise control, Sound, Features, Touch controls). Sentence case, never uppercase, never preceded by a label.
- **Numeral** (500, 2.25rem desktop / 1.875rem mobile): battery levels, tabular.
- **Body** (400, 0.875rem, 1.45): descriptions and hints, max about 46ch.
- **Label** (500, 0.75rem): gesture names and small captions.

### Named Rules
**The Sentence-Case Rule.** No tracked all-caps labels anywhere; hierarchy comes from size and weight.

## Layout

Two columns from `lg`: a stage (about 1.1fr) beside a scrolling console (1fr), 24px gutters, 28px stage padding. Below `lg` it is one column and the whole page scrolls; the stage keeps a 38rem minimum so the model is never cropped. Console sections are separated by a hairline with 28px above and below; within a section the title sits 16px above its control. The connect screen is a single 36rem column, left-aligned, vertically centred.

## Elevation & Depth

Tonal layering with a few real shadows. Surfaces are flat at rest; depth is reserved for what you can act on. Frosted glass (`.glass`: 62% card colour, 18px blur, a 1px inner top highlight) is used for exactly two things: control tracks and the battery strip that overlays the stage rings.

### Shadow Vocabulary
- **Selected pill lift** (`box-shadow: 0 8px 18px -8px color-mix(in oklch, var(--foreground) 45%, transparent)`): the active segment, so it reads as floating above its track.

### Named Rules
**The Glass-Has-A-Job Rule.** Blur is allowed only where something sits over the stage rings or a track needs to read as recessed. It is never ambient decoration.

## Shapes

Soft and rounded, with the radius scaling with the element: 1rem base; tracks 1.8rem; selected pills 1.4rem; the stage 2.4rem; switches and primary buttons fully round. The only non-rounded forms are 1px hairlines and the 4px battery bar (rounded ends).

## Components

### Segmented control
The signature control: a frosted track holding 2-4 options, one pill that slides between them (Framer Motion shared layout, spring 300/32). Noise modes tint the pill by meaning (off: neutral, transparency: teal, cancelling: signal). A second, smaller track for cancelling strength expands beneath only when cancelling is on.

### Switches
shadcn `Switch`; ON is Signal Yellow with an ink thumb. Each row is label + one-line hint, divided by hairlines.

### Battery level
Display-weight numeral that springs to its value, a 4px bar below (foreground normally, Low Red at 20% or less), then the part name and where it is (in ear, in the case; for the case, lid open or closed). Charging state is deliberately not shown.

### Stage
A rounded 2.4rem surface with a faint concentric-ring backdrop. Holds the device name, the interactive 3D model (drag to spin, slow idle drift) and the battery strip.

### Equaliser curve
A single Signal line with ringed points per frequency; the path and points morph when the preset changes.

### Select (touch controls)
shadcn `Select`, full width in its column, one per gesture, labelled by side and gesture.

## Do's and Don'ts

### Do:
- **Do** let exactly one option per group carry Signal Yellow.
- **Do** keep controls reachable by keyboard with the themed focus ring (2px, `--ring`, 2px offset).
- **Do** animate only in response to an action, plus the one 3D assembly on connect; respect `prefers-reduced-motion`.
- **Do** write controls as the action or state they set ("Noise cancelling", "Disconnect"), in sentence case.
- **Do** use `font-variant-numeric: tabular-nums` for every number.

### Don't:
- **Don't** wrap content in identical rounded cards; divide with hairlines and space.
- **Don't** use tracked uppercase labels or eyebrows above headings.
- **Don't** use blur as ambient decoration or put a yellow glow behind things.
- **Don't** use pure black or white; stay on the tinted Ink and Bone neutrals.
- **Don't** show a setting the model or the device's capability bitmap doesn't expose.
