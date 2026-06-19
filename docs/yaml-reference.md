# YAML Reference

Complete guide to every field in an Orrery YAML page.

---

## Page Structure

A page has these top-level sections:

```yaml
page: shop                       # required — page identifier

use:                              # external resources
style:                            # design tokens
data:                             # data providers
compute:                          # derived values
transitions:                      # reusable named transitions
regions:                          # the UI layout tree
interactions:                     # event bindings
```

Only `page` and `regions` are required. Everything else is optional.

```
page         → what this page is called
use          → what external libraries/themes/icons to load
style        → reusable design tokens (colors, spacing, radius)
data         → where the data comes from (APIs, providers)
compute      → calculated values from data
transitions  → reusable animation definitions
regions      → the actual UI — containers with components inside
interactions → what happens when user clicks/taps/submits
```

---

## `use:` — External Resources

Declare component libraries, icons, themes, fonts, and layouts. The server resolves these at compile time.

```yaml
use:
  components: "@shadcn/ui"
  icons: "lucide"
  theme: "https://cdn.myapp.com/themes/dark.yaml"
  layout: "dashboard"
  fonts:
    - "Inter"
    - "JetBrains Mono"
```

### Reference Formats

```yaml
# npm package
components: "@shadcn/ui"

# URL
components: "https://unpkg.com/@my-lib/components"

# GitHub repo
components: "github:user/repo"

# Local file
components: "./my-components.yaml"

# Multiple sources (last wins on name conflict)
components:
  - "@shadcn/ui"
  - "./overrides"

# Cherry-pick specific components
components:
  from: "@shadcn/ui"
  pick: [button, card, dialog]

# Inline HTML template
components:
  product-card:
    template: '<div class="card"><h3>{{ input.name }}</h3></div>'
    style: '.card { border: 1px solid #30363d; }'
```

---

## `style:` — Design Tokens

Reusable values referenced as `$style.*` anywhere in the page.

```yaml
style:
  primary: "#238636"
  accent: "#58a6ff"
  danger: "#f85149"
  surface: "#161b22"
  text: "#e6edf3"
  border: "#30363d"
  radius: 8
  spacing: 16
```

Usage anywhere:

```yaml
style:
  background: "$style.surface"        # → #161b22
  color: "$style.text"                # → #e6edf3
  border-radius: "$style.radius"      # → 8
```

---

## `data:` — Providers

Maps names to data sources registered by the developer's server.

```yaml
data:
  products: productsProvider           # static — pre-baked into FlatBuffer
  cart: cartProvider                    # live — fetched at runtime per user
  notifications: notificationsProvider # live + polling
```

### Static vs Live

```
Static (default):
  Server fetches data at pre-render time.
  Baked into the FlatBuffer/HTML.
  Same for all users. Fast (0ms at runtime).
  Updates when server re-renders (webhook, polling, manual).

Live (mode: "live" in provider registration):
  Adapter fetches at runtime with user's auth token.
  Different per user. 100-200ms latency.
  For: cart, profile, notifications — per-user data.

Live + Polling (mode: "live", poll: N):
  Adapter re-fetches every N seconds.
  For: chat messages, stock prices, live scores.
```

The mode is set in the developer's server-side provider registration, not in YAML. The YAML just references the provider name.

Access data with `$providerName.field`:

```yaml
props:
  name: "$products.name"
  total: "$cart.total"
  avatar: "$user.avatar"
```

---

## `compute:` — Derived Values

Calculated from provider data. Referenced as `$compute.*`.

```yaml
compute:
  subtotal: "$cart.items.sum(price * quantity)"
  discount: "$compute.subtotal * 0.1"
  tax: "($compute.subtotal - $compute.discount) * 0.18"
  total: "$compute.subtotal - $compute.discount + $compute.tax"
```

### Conditional

```yaml
compute:
  tier:
    when:
      - "$user.spend >= 1000": "gold"
      - "$user.spend >= 250": "silver"
      - else: "standard"
```

### Collection Operations

```yaml
compute:
  subtotal: "$cart.items.sum(price * quantity)"
  expensive: "$cart.items.filter(price > 1000)"
  names: "$cart.items.map(name)"
  has-electronics: "$cart.items.any(category == electronics)"
  cheapest: "$cart.items.min(price)"
  sorted: "$cart.items.sort(price)"
  count: "$cart.items.len()"
```

Server evaluates in dependency order using temple's DAG resolver.

---

## `transitions:` — Reusable Named Transitions

Define transitions at page level, reference by name on components/regions.

```yaml
transitions:
  card-enter:
    opacity: 0 -> 1
    y: 30 -> 0
    scale: 0.95 -> 1
    duration: 400
    easing: ease-out

  card-exit:
    opacity: 1 -> 0
    scale: 1 -> 0.9
    duration: 200
    easing: ease-in

  modal-appear:
    opacity: 0 -> 1
    scale: 0.8 -> 1
    blur: 4 -> 0
    duration: 300
    easing: m3-emphasized-decel
```

Usage:

```yaml
- type: card
  transition:
    enter: card-enter
    exit: card-exit

cart-modal:
  layer: 1
  transition:
    enter: modal-appear
    exit: fade-out               # shortcut name
```

See the Transitions section below for full details.

---

## `regions:` — Layout Containers

Regions are containers that arrange children. They control **where** things go.

### All Region Fields

```yaml
regions:
  my-region:
    # ── LAYOUT ──────────────────────────────────────
    direction: horizontal        # horizontal | vertical | grid
    justify: space-between       # start | center | end | space-between
    align: center                # start | center | end | stretch
    gap: 16                      # space between children (logical pixels)
    wrap: true                   # allow children to wrap to next line

    # ── GRID (only when direction: grid) ────────────
    columns: 3                   # number of grid columns

    # ── RESPONSIVE ──────────────────────────────────
    stack-below: 768             # switch horizontal → vertical below this width
    collapse-below: 480          # hide entire region below this width

    # ── OVERLAY ─────────────────────────────────────
    layer: 0                     # 0 = normal flow, 1+ = floats above content
    position: center             # where on screen (only when layer > 0)
    offset:                      # pixel nudge from position anchor
      x: 0
      y: 0

    # ── SCROLLING ───────────────────────────────────
    scroll: false                # enable scrolling if content overflows
    scroll-direction: vertical   # vertical | horizontal

    # ── DATA LOOP ───────────────────────────────────
    data-source: products        # provider name — repeat template for each item
    template: ...                # component template for one item
    loading: ...                 # component to show while loading
    error: ...                   # component to show on error
    empty: ...                   # component to show when empty

    # ── FORM ────────────────────────────────────────
    form: true                   # mark as form for validation
    validate-on: blur            # blur | change | submit

    # ── VISIBILITY ──────────────────────────────────
    condition: "$user.isAdmin"   # show/hide based on data

    # ── ANIMATION ───────────────────────────────────
    transition:
      enter: ...
      exit: ...
      stagger: 60                # delay between children's enter animations

    # ── APPEARANCE ──────────────────────────────────
    style:
      padding: 16
      background: "#161b22"
      border-radius: 12

    # ── CHILDREN (one or both) ──────────────────────
    components: [...]            # leaf components
    regions: { ... }             # nested sub-regions
```

### `direction` — How Children Are Arranged

```yaml
# HORIZONTAL — children sit side by side
header:
  direction: horizontal
  gap: 16
  components:
    - type: heading
      props: { text: "Shop" }
    - type: button
      props: { text: "Cart" }

# Renders:
# ┌──────────────────────────────┐
# │ Shop                    Cart │
# └──────────────────────────────┘


# VERTICAL — children stack top to bottom
sidebar:
  direction: vertical
  gap: 8
  components:
    - type: text
      props: { content: "Home" }
    - type: text
      props: { content: "Products" }
    - type: text
      props: { content: "Cart" }

# Renders:
# ┌──────────┐
# │ Home     │
# │ Products │
# │ Cart     │
# └──────────┘


# GRID — children fill a grid
product-grid:
  direction: grid
  columns: 3
  gap: 16
  components:
    - type: card
      props: { title: "A" }
    - type: card
      props: { title: "B" }
    - type: card
      props: { title: "C" }
    - type: card
      props: { title: "D" }

# Renders:
# ┌────┐ ┌────┐ ┌────┐
# │ A  │ │ B  │ │ C  │
# ├────┤ ├────┘ └────┘
# │ D  │
# └────┘
```

### `justify` and `align`

```yaml
header:
  direction: horizontal
  justify: space-between         # push children to edges
  align: center                  # vertically center children

# ┌──────────────────────────────────┐
# │ Logo                        Cart │  ← space-between
# └──────────────────────────────────┘

footer:
  direction: horizontal
  justify: center                # center children horizontally

# ┌──────────────────────────────────┐
# │          © 2026 MyApp            │  ← centered
# └──────────────────────────────────┘
```

### `layer` and `position` — Overlays

Only for content that floats ABOVE normal content (modals, badges, tooltips).

```yaml
# Modal centered on screen
cart-modal:
  layer: 1                       # floats above everything
  position: center
  style:
    width: 400
    background: "#161b22"
    border-radius: 16
    shadow: "0 8px 32px rgba(0,0,0,0.5)"
  condition: "$ui.cartModalOpen"
  components:
    - type: heading
      props: { text: "Your Cart" }

# Badge on top-right corner of parent
notification-badge:
  layer: 1
  position: top-right
  offset:
    x: -8                        # nudge 8px left
    y: 4                         # nudge 4px down
  components:
    - type: text
      props: { content: "$notifications.count" }
      style:
        background: "#f85149"
        color: "#ffffff"
        border-radius: 12
        padding: "2 8"
```

`position` values: `center`, `top`, `bottom`, `top-left`, `top-right`, `bottom-left`, `bottom-right`

Normal regions (layer: 0) don't use `position` — they're placed by the parent region's direction.

### Nested Regions

```yaml
regions:
  header:
    direction: horizontal
    justify: space-between
    regions:
      brand:
        direction: horizontal
        gap: 8
        components:
          - type: image
            props: { src: "/logo.png", width: 32 }
          - type: heading
            props: { text: "ShopApp" }
      actions:
        direction: horizontal
        gap: 8
        components:
          - type: button
            props: { text: "Cart" }
          - type: button
            props: { text: "Profile" }

# Renders:
# ┌────────────────────────────────────────────┐
# │ [logo] ShopApp              Cart  Profile  │
# │ ←── brand ──→              ←── actions ──→ │
# └────────────────────────────────────────────┘
```

### Data-Source Lists

Repeat a template for each item in a data array.

```yaml
reviews-section:
  direction: vertical
  gap: 12
  data-source: reviews            # loop over this provider
  template:                       # template for ONE item
    type: card
    props:
      author: "$item.author"      # $item = current item in the loop
      text: "$item.text"
      rating: "$item.rating"
  loading:                        # shown while data loads
    type: skeleton
    count: 3
  error:                          # shown on API error
    type: text
    props:
      content: "Failed to load reviews"
  empty:                          # shown when data array is empty
    type: text
    props:
      content: "No reviews yet"
```

### Responsive

```yaml
products:
  direction: horizontal
  stack-below: 768               # below 768px width → vertical
  gap: 16

# Desktop (>768px):            Mobile (<768px):
# ┌────┐ ┌────┐ ┌────┐        ┌──────────────┐
# │ A  │ │ B  │ │ C  │        │      A       │
# └────┘ └────┘ └────┘        ├──────────────┤
#                              │      B       │
#                              ├──────────────┤
#                              │      C       │
#                              └──────────────┘

sidebar:
  collapse-below: 768            # below 768px → hidden entirely
```

### Form

```yaml
login-form:
  direction: vertical
  gap: 12
  form: true
  validate-on: blur

  components:
    - type: input
      id: email
      props:
        placeholder: "Email"
      validate:
        required: true
        matches: "^.+@.+\\..+$"
        error: "Enter a valid email"

    - type: input
      id: password
      props:
        placeholder: "Password"
        type: password
      validate:
        required: true
        min-length: 8
        errors:
          required: "Password is required"
          min-length: "At least 8 characters"

    - type: input
      id: confirm-password
      props:
        placeholder: "Confirm password"
        type: password
      validate:
        condition:
          equals:
            - "$self"
            - "$field.password"
        error: "Passwords don't match"

    - type: button
      id: submit-btn
      props:
        text: "Log In"
```

Validation shorthands:

| Shorthand | Expands to |
|-----------|-----------|
| `required: true` | `notEquals: [$self, ""]` |
| `min-length: 8` | `gte: [length($self), 8]` |
| `max-length: 100` | `lte: [length($self), 100]` |
| `min: 18` | `gte: [$self, 18]` |
| `max: 100` | `lte: [$self, 100]` |
| `matches: "regex"` | `matches: [$self, "regex"]` |

---

## Components

Leaf nodes inside regions. A component has a `type` and renders as a native view.

### All Component Fields

```yaml
- type: card                     # WHAT — component type name
  id: product-card               # WHO — unique ID (for interactions, updates)

  props:                         # VALUES — data the component needs
    title: "$item.name"
    price: "$item.price"
    image: "$item.images[0]"

  style:                         # APPEARANCE — visual overrides
    width: "100%"
    padding: 16
    background: "#161b22"
    border-radius: 12

  span: 1                       # SIZE — proportional width in horizontal/vertical
  colSpan: 2                    # GRID — how many grid columns to span
  rowSpan: 1                    # GRID — how many grid rows to span

  condition: "$item.stock > 0"  # VISIBILITY — show/hide based on data

  transition:                    # ANIMATION — enter/exit/while
    enter: slide-up
    exit: fade-out

  validate: ...                  # FORM — validation rules (input components only)
```

### `type` — What Component to Render

```yaml
# Built-in primitives (ship with every adapter):
- type: text          # displays text
- type: heading       # heading text (h1-h6)
- type: button        # clickable button
- type: input         # text input / textarea
- type: image         # image display
- type: divider       # horizontal rule
- type: tabs          # tab selector

# Custom from component library:
- type: card          # from @shadcn/ui or wherever
- type: avatar        # from component library
- type: badge         # from component library
```

Platform mapping:

| Type | Web | iOS | Android |
|------|-----|-----|---------|
| `text` | `<span>` / `<p>` | `UILabel` | `Text()` |
| `heading` | `<h1>`-`<h6>` | `UILabel` (bold) | `Text()` (headline style) |
| `button` | `<button>` | `UIButton` | `Button()` |
| `input` | `<input>` / `<textarea>` | `UITextField` / `UITextView` | `TextField()` |
| `image` | `<img>` | `UIImageView` | `AsyncImage()` |
| `divider` | `<hr>` | `UIView` (1px) | `Divider()` |
| `tabs` | custom DOM | `UISegmentedControl` | `TabRow()` |

### `id` — Unique Identifier

Required when:
- The component is referenced in `interactions` (`on: my-btn.click`)
- The component needs live data updates
- The component is a form field

```yaml
- type: button
  id: add-to-cart                 # referenced in interactions
  props:
    text: "Add to Cart"

- type: input
  id: email                       # referenced as $form.email
  props:
    placeholder: "Email"
```

### `props` — Values Passed to the Component

Props are what the component needs to render itself. Different component types accept different props.

```yaml
# Built-in primitives — fixed prop names:
- type: text
  props:
    content: "Hello world"        # the text to display

- type: heading
  props:
    text: "Shop"                  # heading text
    level: 1                      # 1-6 (h1-h6)

- type: button
  props:
    text: "Add to Cart"           # button label
    variant: primary              # visual variant
    disabled: false

- type: image
  props:
    src: "$item.images[0]"        # image URL
    alt: "$item.name"             # accessibility text
    width: 200
    height: 150

- type: input
  props:
    placeholder: "Search..."
    type: password                # text | password | email | number
    value: "$form.email"          # pre-filled value

# Library components — props depend on the library:
- type: card                      # @shadcn/ui card
  props:
    title: "$item.name"           # prop names from the library
    description: "$item.desc"
```

Prop values can be:

```yaml
props:
  text: "Hello"                   # static string
  count: 42                       # static number
  active: true                    # static boolean
  name: "$product.name"           # from data provider
  total: "$compute.total"         # from compute section
  color: "$style.primary"         # from style tokens
  label: "Items: $compute.count"  # string interpolation
  first: "$product.tags[0]"       # array access
  city: "$user.address.city"      # nested access
```

### `span` — Proportional Size (horizontal/vertical layouts)

Divides available space among siblings proportionally.

```yaml
header:
  direction: horizontal
  components:
    - type: input
      span: 3                     # takes 3 parts
      props: { placeholder: "Search..." }
    - type: button
      span: 1                     # takes 1 part
      props: { text: "Go" }

# Total spans = 3 + 1 = 4
# Input gets 3/4 = 75% width
# Button gets 1/4 = 25% width
#
# ┌─────────────────────────────────────┬───────────┐
# │          Search...                   │    Go     │
# └─────────────────────────────────────┴───────────┘
# ←──────────── 75% ──────────────────→←──── 25% ──→
```

### `colSpan` / `rowSpan` — Grid Cell Spanning (grid layouts only)

When the parent region uses `direction: grid`, these control how many grid cells a component occupies.

```yaml
product-grid:
  direction: grid
  columns: 4
  gap: 16
  components:
    - type: card
      colSpan: 2                   # spans 2 of 4 columns
      rowSpan: 2                   # spans 2 rows
      props: { title: "Featured" }
    - type: card
      props: { title: "Item 2" }
    - type: card
      props: { title: "Item 3" }
    - type: card
      props: { title: "Item 4" }

# ┌──────────────────┬─────────┬─────────┐
# │                  │ Item 2  │ Item 3  │
# │    Featured      ├─────────┼─────────┤
# │   (2 cols wide,  │ Item 4  │         │
# │    2 rows tall)  │         │         │
# └──────────────────┴─────────┴─────────┘
```

`span` = proportional division in flex layouts (no fixed grid).
`colSpan`/`rowSpan` = cell spanning in grid layouts (fixed grid structure).

---

## `style:` — Visual Appearance

Applied on both regions and components. All values are in logical pixels (server converts to platform-specific units: px on web, pt on iOS, dp on Android).

```yaml
style:
  # SIZING
  width: 300                     # fixed width
  width: "100%"                  # percentage of parent
  width: "auto"                  # size to content
  height: 200
  min-width: 100
  max-width: 600
  min-height: 50
  max-height: 400
  aspect-ratio: "16/9"

  # SPACING
  padding: 16                    # all sides
  padding-top: 8
  padding-bottom: 8
  padding-horizontal: 16         # left + right
  padding-vertical: 8            # top + bottom
  margin: 8
  margin-top: 16
  margin-bottom: 0
  margin-horizontal: "auto"      # center horizontally

  # COLORS
  background: "#161b22"
  background: "$style.surface"   # from tokens
  color: "#e6edf3"               # text color
  opacity: 0.9                   # 0-1

  # BORDERS
  border: "1px solid #30363d"
  border-radius: 12
  border-color: "#30363d"
  border-width: 1

  # SHADOWS
  shadow: "0 4px 12px rgba(0,0,0,0.3)"

  # TEXT
  font-size: 16
  font-weight: bold              # normal | bold | 100-900
  font-family: "Inter"
  text-align: center             # left | center | right
  line-height: 1.5
  text-transform: uppercase      # uppercase | lowercase | capitalize

  # OVERFLOW
  overflow: hidden               # visible | hidden | scroll
```

```
YAML writes:          Each platform gets:
─────────────         ────────────────────
padding: 16           Web: padding: 16px
                      iOS: layoutMargins = UIEdgeInsets(all: 16)
                      Android: Modifier.padding(16.dp)

border-radius: 12     Web: border-radius: 12px
                      iOS: layer.cornerRadius = 12
                      Android: RoundedCornerShape(12.dp)

width: "100%"         Web: width: 100%
                      iOS: constraint equalTo superview.widthAnchor
                      Android: Modifier.fillMaxWidth()
```

---

## `condition:` — Show/Hide

Works on **both** regions and components. The server evaluates the condition and includes or excludes the node from the output.

```yaml
# On a region
admin-panel:
  condition: "$user.isAdmin"
  components:
    - type: button
      props: { text: "Delete All" }

# On a component
- type: button
  condition: "$item.stock > 0"
  props: { text: "Add to Cart" }

- type: text
  condition:
    lte:
      - "$item.stock"
      - 0
  props: { content: "Out of Stock" }
```

### Operators

| Operator | Meaning | Example |
|----------|---------|---------|
| `equals` | == | `equals: ["$user.role", "admin"]` |
| `notEquals` | != | `notEquals: ["$cart.count", 0]` |
| `gt` | > | `gt: ["$item.price", 1000]` |
| `lt` | < | `lt: ["$item.stock", 5]` |
| `gte` | >= | `gte: ["$user.age", 18]` |
| `lte` | <= | `lte: ["$item.stock", 0]` |
| `contains` | array has item | `contains: ["$cart.itemIds", "$product.id"]` |
| `in` | item in array | `in: ["$user.role", ["admin","mod"]]` |
| `matches` | regex | `matches: ["$user.email", ".*@company.com"]` |
| `not` | negate | `not: "$user.banned"` |
| `and` | all true | `and: [cond1, cond2]` |
| `or` | any true | `or: [cond1, cond2]` |

### Complex Example

```yaml
checkout-section:
  condition:
    and:
      - gt:
          - "$cart.total"
          - 0
      - not: "$user.banned"
      - or:
          - "$user.isVerified"
          - gte:
              - "$user.orderCount"
              - 1
```

---

## `transition:` — Animations

### The Primitive Approach

Every animation is **properties changing from one value to another**. Eight primitives combine to create any animation:

| Primitive | What It Controls | Unit | Example |
|-----------|-----------------|------|---------|
| `opacity` | transparency | 0-1 | `0 -> 1` |
| `y` | vertical position | px | `30 -> 0` |
| `x` | horizontal position | px | `-50 -> 0` |
| `scale` | uniform scale | ratio | `0.8 -> 1` |
| `scaleX` | horizontal scale | ratio | `0 -> 1` |
| `scaleY` | vertical scale | ratio | `0 -> 1` |
| `rotate` | rotation | degrees | `-90 -> 0` |
| `blur` | gaussian blur | px | `8 -> 0` |

### Three Phases

```yaml
transition:
  # ENTER — plays when component appears
  enter:
    opacity: 0 -> 1
    y: 30 -> 0
    duration: 300
    easing: ease-out

  # EXIT — plays when component disappears
  exit:
    opacity: 1 -> 0
    scale: 1 -> 0.9
    duration: 200
    easing: ease-in

  # WHILE — loops continuously while visible
  while:
    y: 0 -> -8 -> 0
    duration: 2000
    repeat: infinite
    easing: ease-in-out
    condition: "$item.featured"     # only animate when condition is true
```

Each phase has independent duration, easing, and delay.

### The `->` Syntax

```yaml
# Two values — from → to
opacity: 0 -> 1

# Three values — 0% → 50% → 100% (evenly spaced)
scale: 0.5 -> 1.08 -> 1               # bounce: overshoot then settle

# More values — for complex motion
x: 0 -> -4 -> 4 -> -4 -> 4 -> 0       # shake
rotate: 0 -> -3 -> 3 -> -3 -> 0       # wiggle
scale: 1 -> 1.15 -> 1 -> 1.08 -> 1    # heartbeat
```

### Timing Properties

```yaml
transition:
  enter:
    opacity: 0 -> 1
    y: 30 -> 0
    duration: 300                # milliseconds
    easing: ease-out             # acceleration curve
    delay: 100                   # wait before starting (ms)
```

### Easing Values

| Easing | Use For |
|--------|---------|
| `ease-out` | Enter transitions (fast start, slow end) |
| `ease-in` | Exit transitions (slow start, fast end) |
| `ease-in-out` | Symmetric / while animations |
| `linear` | Spinners, progress bars |
| `m3-standard` | Material Design 3 general motion |
| `m3-emphasized-decel` | Material Design 3 entrances |
| `m3-emphasized-accel` | Material Design 3 exits |
| `spring` | Overshoots and settles (playful, iOS-style) |

### Per-Property Timing

When different properties need different speeds:

```yaml
transition:
  enter:
    opacity:
      from: 0
      to: 1
      duration: 200
      easing: linear
    y:
      from: 40
      to: 0
      duration: 400
      easing: ease-out
    scale:
      from: 0.9
      to: 1
      duration: 400
      delay: 100                   # scale starts 100ms after opacity
      easing: spring
```

### Shortcuts

Named shortcuts for common patterns. Expand to primitives internally:

| Shortcut | Expands To |
|----------|-----------|
| `fade-in` | `opacity: 0 -> 1` |
| `fade-out` | `opacity: 1 -> 0` |
| `slide-up` | `opacity: 0 -> 1, y: 30 -> 0` |
| `slide-down` | `opacity: 0 -> 1, y: -30 -> 0` |
| `slide-left` | `opacity: 0 -> 1, x: 30 -> 0` |
| `slide-right` | `opacity: 0 -> 1, x: -30 -> 0` |
| `scale-in` | `opacity: 0 -> 1, scale: 0.8 -> 1` |
| `pop-in` | `opacity: 0 -> 1, scale: 0.5 -> 1.08 -> 1` |
| `blur-in` | `opacity: 0 -> 1, blur: 8 -> 0` |

Use directly:

```yaml
transition:
  enter: slide-up
  exit: fade-out
```

Or override duration/easing:

```yaml
transition:
  enter:
    type: slide-up
    duration: 500
    easing: spring
```

### Stagger (Lists)

When a region has `data-source`, stagger delays each item's enter animation:

```yaml
products:
  data-source: products
  transition:
    enter: slide-up
    stagger: 60                   # 60ms between each item

# Item 0 enters at 0ms
# Item 1 enters at 60ms
# Item 2 enters at 120ms
# Item 3 enters at 180ms
```

### Why Primitives, Not Predefined Animations

An earlier design used 41 predefined named animations (fade-in, slide-up, scale-in, pop-in, drop-in, flip-in, etc.). Each was a fixed keyframe. Combining two animations required a separate predefined combination (slide-fade-up, slide-scale-up, etc.).

Problems with that approach:
- Only 4 composed combinations were predefined — "what if I want slide + scale + blur? Not in the list."
- Adding new combinations required changing the engine, not just YAML
- Per-property timing was impossible — everything shared one duration
- Custom motion (like a specific bounce curve) was impossible

The primitive approach gives the user 8 building blocks that combine into infinite animations. Shortcuts still exist for convenience, but they're just aliases for primitives — not a separate system.

---

## `interactions:` — Event Bindings

Defines what happens when a user clicks, submits, or interacts with elements. Format: `element-id.event`.

### Built-in Actions (No Server Call)

```yaml
interactions:
  # Show a region
  - on: cart-btn.click
    do: show
    target: cart-modal

  # Hide a region
  - on: close-cart.click
    do: hide
    target: cart-modal

  # Toggle visibility
  - on: menu-btn.click
    do: toggle
    target: sidebar

  # Navigate to another page
  - on: logo.click
    do: navigate
    to: home.yaml

  # Go back
  - on: back-btn.click
    do: back

  # Show notification
  - on: copy-btn.click
    do: notify
    message: "Copied to clipboard"

  # Re-fetch a data provider
  - on: refresh-btn.click
    do: refresh
    target: reviews

  # Submit form (validate → call handler)
  - on: submit-btn.click
    do: submit
    target: login-form
    handler: login
    args:
      email: "$form.email"
      password: "$form.password"
```

| Action | What It Does | Server Call? |
|--------|-------------|-------------|
| `show` | Make a hidden region visible | No |
| `hide` | Hide a visible region | No |
| `toggle` | Toggle visibility | No |
| `navigate` | Go to another page | No |
| `back` | Go back in navigation | No |
| `notify` | Show a toast/notification | No |
| `refresh` | Re-fetch a data provider | Yes (data fetch only) |
| `submit` | Validate form, then call handler | Yes |

### Handler Calls (Server Round-Trip)

```yaml
interactions:
  - on: add-to-cart.click
    handler: addToCart
    args:
      product_id: "$product.id"
      quantity: 1
    then:
      - do: refresh
        target: cart
      - do: notify
        message: "Added to cart!"
```

```
What happens:
1. User clicks the element with id "add-to-cart"
2. Adapter sends POST to server: { handler: "addToCart", args: { product_id: "hp-001", quantity: 1 } }
3. Server runs the developer's handler function (database write, API call, etc.)
4. Server responds with what changed (diff)
5. Adapter applies diff to the UI
6. Adapter executes "then" chain: refresh cart data, show notification
```

### `then` Chains

Sequential actions after a handler completes:

```yaml
then:
  - do: refresh
    target: cart
  - do: hide
    target: cart-modal
  - do: notify
    message: "Order placed!"
  - do: navigate
    to: confirmation.yaml
```

If the handler fails, the `then` chain does not execute.

### Platform Extensions

For native capabilities that can't be expressed in YAML:

```yaml
- on: take-photo.click
  do: extension
  name: camera
  then:
    - handler: uploadPhoto
      args:
        image: "$extension.result.imageData"
    - do: notify
      message: "Photo uploaded!"
```

Extensions are registered per-platform by the developer (camera, biometrics, share, clipboard, etc.).

---

## `$references` — Data Access

All `$` prefixed strings are resolved by the server before output.

| Prefix | Source | Example |
|--------|--------|---------|
| `$providerName.*` | Provider data | `$product.name`, `$cart.total` |
| `$style.*` | Style tokens | `$style.primary`, `$style.radius` |
| `$compute.*` | Computed values | `$compute.subtotal`, `$compute.tier` |
| `$item.*` | Current item in data-source loop | `$item.name`, `$item.price` |
| `$form.*` | Form field values | `$form.email`, `$form.password` |
| `$event.*` | Event data (in interactions) | `$event.value` |
| `$self` | Current field value (in validation) | `$self` |
| `$field.*` | Other field values (cross-field validation) | `$field.password` |
| `$extension.result` | Extension return value | `$extension.result.imageData` |

### Access Patterns

```yaml
props:
  # Simple field
  name: "$product.name"

  # Nested object
  city: "$user.address.city"

  # Array index
  first-image: "$product.images[0]"

  # String interpolation
  label: "Hello, $user.name!"
  stock-warning: "Only $product.stock left"

  # Boolean expression
  in-stock: "$product.stock > 0"
```

---

## Full Page Example

```yaml
page: shop

use:
  components: "@shadcn/ui"
  icons: "lucide"
  theme: "./brand-dark.yaml"
  fonts:
    - "Inter"

style:
  primary: "#238636"
  surface: "#161b22"
  text: "#e6edf3"
  border: "#30363d"

data:
  products: productsProvider
  cart: cartProvider

compute:
  cartLabel: "Cart ($cart.count)"
  hasItems: "$cart.count > 0"

transitions:
  card-enter:
    opacity: 0 -> 1
    y: 20 -> 0
    scale: 0.95 -> 1
    duration: 300
    easing: ease-out

regions:
  header:
    direction: horizontal
    justify: space-between
    align: center
    style:
      padding: 16
      background: "$style.surface"
    components:
      - type: heading
        props:
          text: "Shop"
          level: 1
      - type: button
        id: cart-btn
        props:
          text: "$compute.cartLabel"
          variant: ghost

  products:
    direction: grid
    columns: 3
    gap: 16
    stack-below: 768
    style:
      padding: 16
    data-source: products
    transition:
      enter: card-enter
      stagger: 60
    template:
      type: card
      id: product-card
      props:
        title: "$item.name"
        price: "$item.price"
        image: "$item.images[0]"
        rating: "$item.rating"
      style:
        border-radius: "$style.radius"
    loading:
      type: skeleton
      count: 6
    empty:
      type: text
      props:
        content: "No products found"

  cart-modal:
    layer: 1
    position: center
    condition: "$ui.cartModalOpen"
    transition:
      enter:
        opacity: 0 -> 1
        scale: 0.85 -> 1
        duration: 300
        easing: m3-emphasized-decel
      exit:
        opacity: 1 -> 0
        duration: 200
        easing: ease-in
    style:
      width: 420
      max-height: "80%"
      background: "$style.surface"
      border-radius: 16
      shadow: "0 8px 32px rgba(0,0,0,0.5)"
    components:
      - type: heading
        props:
          text: "Your Cart"
          level: 2

interactions:
  - on: cart-btn.click
    do: show
    target: cart-modal

  - on: product-card.click
    handler: addToCart
    args:
      product_id: "$item.id"
      quantity: 1
    then:
      - do: refresh
        target: cart
      - do: notify
        message: "Added to cart!"

  - on: close-cart.click
    do: hide
    target: cart-modal
```
