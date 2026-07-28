# Source-crafter — Roadmap

The roadmap is organized as four progressive stages: **POC**, **MVP**, **Enhancements**, **Moderation**. Each stage carries a top-level goal and a list of checkpoints. Checkpoints are stated as **goals achieved**, not tasks to do — they describe the outcome we expect to see, not the work that produces it.

---

## 1. POC — Proof of Concept

**Goal.** Prove that a YAML file can produce a visible, interactive UI on a real platform.

**Checkpoints:**

- [ ] YAML files can be loaded, parsed, and validated without runtime crashes.
- [ ] The browser adapter renders a YAML page into real DOM elements.
- [ ] All atomic primitives (box, text, image, click-region, edit-field, scroll, group) are visible on screen.
- [ ] A composite primitive defined in YAML can be referenced and reused from another YAML file.
- [ ] A small demo page produces visual output indistinguishable from a hand-written HTML+CSS equivalent.
- [ ] A first non-trivial example — a page with header, sidebar, content area, and a form — renders correctly.

**Stage exit criterion.** The architecture is no longer hypothetical: a working browser demo exists and can be shown to anyone.

---

## 2. MVP — Minimum Viable Product

**Goal.** A real team can ship a real product using source-crafter on the browser. Production-quality on one platform beats half-quality on five.

**Checkpoints:**

- [ ] All five UI stages work end-to-end: render, layout, cross-component interaction, business logic, navigation.
- [ ] Forms with field-level validation can be expressed entirely in YAML.
- [ ] Data providers with loading and error states are fully integrated with the engine.
- [ ] Multi-page navigation with a working history stack, back behavior, and route-based URL handling is supported.
- [ ] An application with at least ten pages can be built and runs end-to-end without crashes or layout glitches.
- [ ] Browser-adapter performance is acceptable for real product use — no perceptible frame drops on typical pages.
- [ ] The schema (yaml.md) is stable enough that page authors don't get broken by version bumps within the MVP timeframe.
- [ ] At least one team outside the core maintainers has built something non-trivial with source-crafter and shipped it.

**Stage exit criterion.** Source-crafter is production-quality on the browser and at least one external user has shipped a real product on it.

---

## 3. Enhancements

**Goal.** Expand the surface to cover more platforms, patterns, and product needs — without compromising the schema or the engine's predictability.

**Checkpoints:**

- [ ] Additional platform adapters are available — Android, iOS — each rendering the same YAML correctly.
- [ ] Library imports (component libraries, theme packs, font sources) are supported and work in real products.
- [ ] Animations and transitions, including continuous loops, render correctly across platforms.
- [ ] The medium-priority gaps surfaced in the audit are addressed as concrete user demand arises: internationalization, real-time data, drag and drop, app-level keyboard shortcuts, file uploads, media (video/audio), error boundaries, lazy loading / virtualization, theme switching.
- [ ] The low-priority gaps (context menus, multi-touch gestures, clipboard actions, free-form canvas, webview/iframe embeds, print layouts) are covered as needed.
- [ ] A developer-experience layer exists — hot reload of YAML changes during development, a parse-time error overlay, and an in-app inspector for live state and references.
- [ ] AI-driven YAML generation has at least one working integration where a prompt in plain English produces a renderable page.

**Stage exit criterion.** Source-crafter is the natural choice for multi-platform product UIs within its target zone, and the platform adapter set covers the surfaces most teams ship on.

---

## 4. Moderation — Steady State

**Goal.** Keep the project stable, responsive, and trustworthy for production users. This stage does not end — it is the ongoing posture once the framework is in real use.

**Checkpoints:**

- [ ] Bug reports have a clear intake, triage, and resolution path that users can rely on.
- [ ] Performance regressions are caught by automated checks before each release lands on users.
- [ ] Schema changes follow a documented versioning and migration policy — no silent breaking changes.
- [ ] Backward-compatibility commitments are written down and honored.
- [ ] A feedback channel between users and maintainers exists and stays responsive.
- [ ] Documentation (philosophy, design, yaml schema, roadmap) stays in sync with the shipped behavior — no documented field that doesn't work, no working field that isn't documented.
- [ ] Security issues have a private disclosure path and are handled with clear timelines.

**Stage characteristic.** Every new feature added at any future iteration passes through this same gate — backward compatibility, performance budget, documentation, regression tests — before it's considered done.

---

## How the stages relate

| Stage | What changes | What stays the same |
|---|---|---|
| **POC** | architecture validated, first primitives working | nothing yet — everything still being decided |
| **MVP** | full feature set on one platform (browser); real users | the schema (yaml.md) stabilizes; engine internals can still shift |
| **Enhancements** | more platforms, more patterns, more libraries | the schema's core stays additive — no breaking changes |
| **Moderation** | bug fixes, regressions, polish | the surface is now public; everything is contract |

Each stage builds on the previous one. The order is fixed: there's no point in adding Android support before the browser ships a real product, and no point chasing AI generation before the schema is stable. Each stage's exit criterion gates the next.
