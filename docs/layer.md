# Layers of UI work

Every UI product is a mix of three concerns. Source-crafter owns two of them (75%) and gives you a clean surface for the third.

```mermaid
flowchart TD
    UI([UI work in a typical product])

    UI --> R["<b>Component Rendering + Layout</b><br/>50%"]
    UI --> B["<b>Business Logic</b><br/>25%"]
    UI --> F["<b>Flow + Transition</b><br/>25%"]

    R --> R1[Modularization]
    R --> R2[Layout System]
    R --> R3[Styling & States]

    B --> B1[Data Integration]
    B --> B2[State & Binding]
    B --> B3[Handlers & Actions]

    F --> F1[Navigation]
    F --> F2[Page Flow]
    F --> F3[Transitions]
    F --> F4[Animations]

    classDef area fill:#1f3a1f,stroke:#4ade80,color:#e5e7eb,font-weight:bold
    classDef leaf fill:#3a1f1f,stroke:#f87171,color:#fde68a
    class R,B,F area
    class R1,R2,R3,B1,B2,B3,F1,F2,F3,F4 leaf
```

- **Component Rendering + Layout (50%)** — engine-owned. Described in YAML; the engine parses, validates, and lays out.
- **Business Logic (25%)** — user-owned. You write providers (data fetching) and handlers (business rules); the engine wires them in by name.
- **Flow + Transition (25%)** — engine-owned. Navigation, page transitions, and animations are declared in YAML and driven by the engine.
