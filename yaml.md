# Orrery YAML — every field, what it does, and its OpenUI equivalent

**Format:** each field carries a `# comment` explaining its usage plus the OpenUI v0.5 equivalent (or `# not in OpenUI`).
**Reference:** `yaml-schema-v2.md` for formal type definitions; `openui-comparison.yaml` for the deeper OpenUI mapping.

---

## 1. Top-level Page

```yaml
page: order-management                # What: unique page identifier (used for navigation)
                                      # Use: filename-slug convention
                                      # OpenUI: no equivalent — OpenUI is single-page per LLM call

params:                               # What: inputs the page accepts when navigated to
                                      # Use: like function arguments for the page
                                      # OpenUI: no equivalent — no page params
  orderId:
    type: string                      # ParamSpec.type — string | number | boolean | handler | enum | list | any
    required: true                    # ParamSpec.required — parser rejects render call without it
    default: ""                       # ParamSpec.default — used if not provided
    options: []                       # ParamSpec.options — only if type == enum

style:                                # What: design tokens for the whole page
                                      # Use: reference anywhere as $style.<name>
                                      # OpenUI: no equivalent — component library owns theming
  primary: "#238636"
  danger:  "#e94560"
  muted:   "#8b949e"

data:                                 # What: named data providers the page uses
                                      # Use: reference results as $data.<name>
                                      # OpenUI: equivalent to Query() statements at the top
  orders:
    provider: listOrders              # DataSpec.provider — name of a registered provider fn
    params:                           # DataSpec.params — args passed to the provider
      status: $state.filter
    depends_on:                       # DataSpec.depends_on — auto-refetch when any listed ref changes
      - $state.filter                 # OpenUI: Query auto-tracks $vars in args (equivalent behavior)
  user: userProfile                   # shorthand — no params, no auto-refetch

handlers:                             # What: handler function names the engine may call
                                      # Use: declared here so validator knows which are legal
                                      # OpenUI: no equivalent — actions are inline, no declaration
  - handleSearch
  - handleDelete

state:                                # What: reactive page-level state
                                      # Use: read as $state.<name>, write via bind/do:set/handler
                                      # OpenUI: equivalent to $var = default declarations
  filter:
    type: string                      # StateField.type — string | number | boolean | enum | list | any
    default: ""                       # StateField.default — initial value
  page:
    type: number
    default: 1
  activeTab:
    type: enum                        # for enum, options MUST be a non-empty list
    default: overview
    options: [overview, settings, history]

regions:                              # What: layout tree — named containers, always the entry point for rendering
                                      # Use: engine iterates this map to build the UI
                                      # OpenUI: equivalent to root = Root([children])
  header: { ... }                     # each key is a Region — full example in §4
  body:   { ... }

interactions:                         # What: page-level event → action wiring
                                      # Use: cross-component events (button click → refresh other region)
                                      # OpenUI: no page-level bucket — actions are attached inline to component props
  - on: search-btn.click              # EventBinding — "<componentId>.<eventName>"
    do: refresh                       # BuiltInAction — one of 18 verbs
    target: results-table
    with: { force: true }
    when: "$state.online"             # Expression — evaluated as boolean guard

  - on: form.submit
    handler: handleSubmit             # discriminated union — either do OR handler, never both
    with: { source: "topbar" }

flows:                                # What: multi-step sequences with per-step guards and result piping
                                      # Use: when a click needs to run 3 steps in order
                                      # OpenUI: equivalent to Action([@Run(a), @Run(b), @Reset($c)])
  onSave:
    on: save-btn.click
    steps:
      - call: "handler:validate"      # FlowStep with call — handler:<name> or built-in name
        out: validation               # exposes result as $flow.validation
      - do: notify                    # FlowStep with do — a BuiltInAction verb
        with: { message: "Invalid" }
        when: "!$flow.validation.ok"
      - call: "handler:persist"
        out: saved
        when: "$flow.validation.ok"
      - do: navigate
        with: { to: "success.json" }
        when: "$flow.saved.ok"
```

---

## 2. PrimitiveDef — reusable custom primitive

```yaml
kind: composite                        # What: file discriminator (this is a composite primitive definition)
                                       # OpenUI: no equivalent — vocabulary is fixed in system prompt

name: UserCard                         # What: the type name others reference via `kind: UserCard`
                                       # Use: PascalCase convention

params:                                # What: read-only inputs from the caller
                                       # Use: available inside body as $params.<name>
                                       # OpenUI: equivalent to component function args
  name:
    type: string
    required: true
  role:
    type: string
    default: "Member"

state:                                 # What: instance-owned reactive state
                                       # Use: available inside body as $self.<name>
                                       # OpenUI: no equivalent — component-instance state must be lifted to page $vars
  expanded:
    type: boolean
    default: false

body:                                  # What: the component tree that renders when this primitive is used
                                       # Use: same shape as any Component[] array
  - kind: box
    on:
      click:
        do: set
        target: $self.expanded
        with: { value: true }
    children:
      - kind: text
        content: $params.name
```

---

## 3. RegionTemplate — reusable region

```yaml
kind: region                           # What: file discriminator (this is a reusable region)
name: Sidebar                          # What: template name — referenced via `use: regions/sidebar.json`

params:
  items:
    type: list
    required: true

body:                                  # What: a single Region (not an array — a region)
  direction: vertical
  gap: 8
  components:
    - kind: NavItem
      each: $params.items              # iteration over the passed-in list
      as: $item
      params:
        label: $item.label
```

---

## 4. Region — all fields

```yaml
regions:
  header:
    id: header-region                  # What: optional identifier for interactions/actions to target
                                       # OpenUI: no equivalent — components have no id, only $var refs

    direction: horizontal              # What: layout axis — horizontal | vertical | grid
                                       # Use: engine → CSS display + flex-direction / grid
                                       # OpenUI: Row([]) / Col([]) as components

    gap: 16                            # What: pixel spacing between children
                                       # OpenUI: no equivalent — component library controls spacing

    justify: space-between             # What: main-axis alignment — start | center | end | space-between
                                       # OpenUI: no equivalent

    span: 1                            # What: proportional space vs siblings (uses whole numbers)
                                       # Use: sidebar=1 + content=3 → sidebar gets 25%
                                       # OpenUI: no equivalent

    columns: 3                         # What: grid column count — only when direction: grid
                                       # OpenUI: no equivalent

    layer: 0                           # What: z-order — 0 = page, 1 = modal, 2 = tooltip-on-modal
                                       # OpenUI: no equivalent — no overlay concept

    collapseBelow: 768                 # What: hide region when container width < N px
                                       # Use: sidebar disappears on narrow screens
                                       # OpenUI: no equivalent — no responsive rules

    stackBelow: 480                    # What: switch children to vertical when container width < N px
                                       # OpenUI: no equivalent

    hidden: false                      # What: static hide flag (not reactive — use when: for reactive)
                                       # OpenUI: no equivalent

    when: "$state.count > 0"           # What: reactive render guard — expression evaluates to bool
                                       # Use: skip rendering when false
                                       # OpenUI: cond ? Component() : null

    each: $data.products               # What: repeat this region once per collection item
                                       # Use: as: required — engine auto-indexes ID if set
                                       # OpenUI: @Each(array, "var", template)

    as: $item                          # What: loop variable name (required when each is set)
                                       # OpenUI: string name in second arg of @Each

    context:                           # What: values inherited by descendant components
                                       # Use: reference as $context.<name>
                                       # OpenUI: no equivalent — pass explicitly through props
      theme: dark
      currency: INR

    style:                             # StyleBlock — see §7
      background: $style.primary
      padding: 12

    regions:                           # What: nested sub-regions (mutually exclusive with `use:`)
                                       # Use: build compound layouts
      left:  { direction: horizontal, gap: 8, components: [...] }
      right: { direction: horizontal, gap: 8, components: [...] }

    components:                        # What: leaf components at this region level (mutually exclusive with `use:`)
      - kind: Logo
      - kind: SearchBar
        id: search
        span: 3

    use: regions/sidebar.json          # What: reference a RegionTemplate file (mutually exclusive with regions/components)
    params:                            # params passed to the RegionTemplate
      items: ["Home", "Orders", "Settings"]

    platform:                          # What: per-platform overrides — merged on top of base
      browser: { style: { padding: 16 } }
      ios:     { style: { padding: 12 } }
      android: { style: { padding: 8 } }
      terminal:{ hidden: true }
      desktop: { }
                                       # OpenUI: no equivalent — single platform
```

---

## 5. KindComponent — all fields

```yaml
components:
  - kind: Button                       # What: primitive name (box/text/edit-field/...) or composite name
                                       # Use: engine picks up the renderer by name
                                       # OpenUI: Button(...) as a function call

    id: save-btn                       # What: stable id for interactions, hydration, partial updates
                                       # Use: interactions target it as "save-btn.click"
                                       # OpenUI: no equivalent — no addressable ids

    size:                              # What: layout-level size hints (overrides style.width/height)
      width: fill                      # SizeValue: number | "hug" | "fill" | { percent: N } | { proportional: N }
      height: 48
                                       # OpenUI: passed via component params

    style:                             # StyleBlock — see §7
      background: $style.primary
      radius: 8

    states:                            # VisualStates — see §8
      hover:    { style: { opacity: 0.9 } }
      disabled: { style: { opacity: 0.4 } }

    state:                             # What: component-instance state (component-owned reactive fields)
                                       # Use: read as $<id>.<field>
      loading:
        type: boolean
        default: false

    disabled: false                    # What: writable boolean flag (separate from visual state)
                                       # Use: flipped by enable/disable actions
                                       # OpenUI: passed as `disabled` prop

    data:                              # What: bind this component to a data provider result
      provider: userProfile            # short form is `data: userProfile`
      params: { userId: $state.uid }
      depends_on: [$state.uid]
                                       # OpenUI: name = Query(tool, args, fallback)

    bind:                              # What: two-way binding from a state ref
      value: $state.searchTerm         # user input writes back to $state.searchTerm
                                       # OpenUI: automatic when $var is passed to an Input

    content: "Save"                    # What: primitive content (text nodes, image alt, etc.)
                                       # Use: can be a $ref
                                       # OpenUI: first positional arg on components

    children:                          # What: nested components (for container primitives)
      - kind: text
        content: $params.label

    each: $data.products               # What: repeat this component once per item (component-level)
    as: $item                          # required when each is set
                                       # OpenUI: @Each(array, "var", template)

    on:                                # What: local event bindings (fire in addition to page interactions)
      click:                           # Action inline shape
        do: refresh
        target: results-table
      hover: handleHover               # shorthand — handler name string
                                       # OpenUI: onClick=Action([...])

    when: "$state.count > 0"           # reactive render guard
    a11y:                              # A11yBlock — see §10
      role: button
      label: "Save order"
    span: 2                            # proportional size vs siblings
    platform: { ios: { style: { padding: 12 } } }

    params:                            # What: props passed to the component/composite
                                       # Use: any-shape map (validated against composite ParamSpec if applicable)
                                       # OpenUI: positional args on the function call
      text: "Save"
      variant: "primary"
```

---

## 6. UseComponent — reference a PrimitiveDef

```yaml
- use: primitives/user-card.json       # What: file path to a PrimitiveDef (relative to loading file)
                                       # OpenUI: no equivalent — no user-defined composites

  id: user-1                           # optional id
  size:     { width: 200 }             # can override layout at usage site
  style:    { padding: 8 }
  disabled: false
  each: ...                            # can also iterate
  as: $item
  on:       { click: ... }
  when:     "$state.showCard"
  a11y:     { role: article }
  span:     1
  platform: { }

  params:                              # params passed to the PrimitiveDef
    name: "Anshu Mishra"
    role: "Admin"
```

---

## 7. SwitchComponent — multi-branch rendering

```yaml
- switch: $state.activeTab             # What: reactive value; branch decision made at runtime
                                       # OpenUI: cond ? A() : B() (ternary only, no dedicated switch)
  cases:                               # value coerced to string; first match wins
    overview: { kind: OverviewPanel }
    settings: { kind: SettingsPanel }
    history:  { kind: HistoryPanel }
  default:    { kind: OverviewPanel }  # fallback when no case matches

  id: main-panel
  when: "$state.loggedIn"
  span: 3
  platform: { }
```

---

## 8. Style — tokens and blocks

```yaml
style:                                 # StyleTokens — at Page level
                                       # OpenUI: no equivalent
  primary: "#238636"
  onPrimary: "#ffffff"
  muted: "#8b949e"
  border: "#30363d"

# StyleBlock — usable anywhere a component/region can be styled
style:
  background: $style.primary           # Color: hex, rgb(), rgba(), named, or $style.<token>
  color: "$style.onPrimary"

  radius: 8                            # number OR object { topLeft, topRight, bottomLeft, bottomRight }

  padding: { x: 16, y: 12 }            # number OR Edges { x, y, top, right, bottom, left }
  margin: 8

  border:
    width: 1
    color: "#e5e7eb"
    style: solid                       # solid | dashed | dotted

  opacity: 0.9
  shadow: "0 2px 8px rgba(0,0,0,0.1)"

  font:                                # FontBlock
    family: "Inter"
    size: 16
    weight: bold                       # normal | bold | number
    style: italic                      # normal | italic
    lineHeight: 1.5
    letterSpacing: 0.2

  align: center                        # left | center | right | justify
  cursor: pointer                      # default | pointer | text | not-allowed
  outline: "2px solid $style.primary"
  transform: "rotate(2deg)"
  width: 200                           # escape hatch — prefer size.width
  height: "100%"
```

---

## 9. VisualStates — style hooks

```yaml
states:                                # What: style hooks per visual state — engine tracks native events
                                       # OpenUI: no equivalent — component library handles visual states
  hover:
    style: { background: $style.hoverBg }
    content: "Hovering!"               # StateOverride can also override content

  pressed:
    style: { opacity: 0.8 }

  focused:
    style: { outline: "2px solid $style.focusColor" }

  disabled:                            # responds to component.disabled flag
    style: { opacity: 0.4, cursor: not-allowed }

  selected:                            # responds to component's `selected` state
    style: { background: $style.selectedBg }
```

---

## 10. A11y block

```yaml
a11y:                                  # What: accessibility hints exposed to platform a11y APIs
                                       # OpenUI: no equivalent — a11y is component-library responsibility
  role: button                         # ARIA role
  label: "Close modal"                 # accessible name
  hint: "Closes the current dialog"    # extra description
  hidden: false                        # hide from a11y tree
  level: 2                             # heading level (h1..h6)
  live: polite                         # off | polite | assertive — announces changes
```

---

## 11. Interaction — page-level event wiring

```yaml
interactions:
  # Variant 1 — built-in action (18 verbs available)
  - on: search-btn.click               # EventBinding: "<componentId>.<eventName>"
    do: refresh                        # BuiltInAction verb
    target: results-table
    with: { force: true }
    when: "$state.online"              # optional guard
                                       # OpenUI: Action([@Run(refresh)])

  # Variant 2 — handler
  - on: form.submit
    handler: handleFormSubmit
    with: { source: "topbar" }
    when: "$state.formEnabled"
                                       # OpenUI: no direct equivalent — handlers are inline via @Run(mutation)

# The 18 BuiltInAction verbs:
#   show | hide | toggle              (visibility)
#   navigate | back                    (routing)
#   notify                             (toast)
#   scroll-to | focus                  (UX helpers)
#   enable | disable                   (writes .disabled flag)
#   update | reset                     (mass prop updates)
#   fetch | refresh                    (data providers)
#   set | increment | decrement        (state mutations)
```

---

## 12. Flow — multi-step sequences

```yaml
flows:
  onSave:
    on: save-btn.click                 # EventBinding
    when: "$state.dirty"               # optional flow-level guard
    steps:
      # step variant 1 — handler call
      - call: "handler:validate"       # "handler:<name>" or built-in name
        out: validation                # publishes result as $flow.validation
        when: "$state.dirty"

      # step variant 2 — built-in action
      - do: notify
        target: null
        with: { message: "Invalid" }
        when: "!$flow.validation.ok"

      - call: "handler:persist"
        out: saved
        when: "$flow.validation.ok"

      - do: navigate
        with: { to: "success.json" }
        when: "$flow.saved.ok"
                                       # OpenUI: Action([
                                       #   @Run(validate), @Run(persist), @OpenUrl(...)
                                       # ])
```

---

## 13. StateField reference

```yaml
state:
  searchTerm:
    type: string                       # string | number | boolean | enum | list | any
    default: ""
                                       # OpenUI: $searchTerm = ""

  count:
    type: number
    default: 0

  isOpen:
    type: boolean
    default: false

  activeTab:
    type: enum                         # requires non-empty options
    default: overview
    options: [overview, settings, history]

  selected:
    type: list                         # arrays
    default: []

  metadata:
    type: any                          # untyped blob (last resort)
    default: null
```

---

## 14. Reference prefixes — the whole vocabulary

```yaml
# All string values may contain $refs; the engine resolves them at render time.
#
# $state.<name>            page-level state          Page.state
# $self.<name>             primitive's own state     PrimitiveDef.state (inside body only)
# $context.<name>          inherited context         nearest Region.context (deep-merged from ancestors)
# $<id>.<field>            another component's state engine state store keyed by id
# $params.<name>           param value               PrimitiveDef/RegionTemplate body only
# $data.<name>             provider result           Page.data.<name>
# $style.<name>            design token              Page.style.<name>
# $nav.<name>              navigation data           most recent navigate call only
# $flow.<name>             flow step out: result     inside flow only
#
# OpenUI equivalents:
#   $var                   direct signal (single flat namespace)
#   name.field             dot access on returned data
#   no page-context / no design-token / no flow scope
```

---

## 15. Complete minimal working example

```yaml
# What: minimal counter — demonstrates state + built-in action
# Use: hand this to orrery.renderJson(container, page) and it works

page: counter                          # OpenUI equivalent below in comments

state:                                 # OpenUI: $count = 0
  count:
    type: number
    default: 0

regions:                               # OpenUI: root = Col([...])
  root:
    direction: vertical
    gap: 16
    components:
      - kind: text                     # OpenUI: Text(concat("Count: ", $count))
        content: "Count: $state.count"

      - kind: Button                   # OpenUI: Button("Add one", onClick=Action([@Set($count, $count+1)]))
        id: inc-btn
        params:
          text: "Add one"
        on:
          click:
            do: increment
            target: $state.count
```

---

## Reading guide

- `#` comments here are documentation — they explain what/how/OpenUI-equivalent per field.
- Only field names are stable. Values shown are examples.
- Mutual-exclusivity rules (kind vs use vs switch, regions/components vs use, do vs handler, do vs call) are enforced by the validator, not by the type system.
- Every OpenUI reference maps back to §1-10 in `openui-comparison.yaml` for the deeper story.
