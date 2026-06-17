# YAML Reference

Complete syntax guide for Orrery YAML templates.

---

## Page Structure

A YAML template has these top-level sections:

```yaml
page: page-name              # Page identifier

use:                          # External references (optional)
  components: "@shadcn/ui"
  icons: "lucide"
  theme: "https://cdn.myapp.com/themes/dark.yaml"
  fonts:
    - "Inter"

style:                        # Global style tokens
  primary: "#238636"
  accent: "#58a6ff"

data:                         # Data provider mappings
  product: productDetailProvider
  cart: cartProvider

compute:                      # Computed values (optional)
  subtotal: "$cart.items.sum(price * quantity)"

regions:                      # UI layout tree
  header:
    direction: horizontal
    components:
      - type: text
        props:
          content: "$product.name"

interactions:                 # Event bindings
  - on: buy-btn.click
    handler: addToCart
    args:
      product_id: "$product.id"
```

All sections are optional except `page` and `regions`.

---

## Use Section

Declare external resources for the page. The engine resolves and compiles them at deploy time.

```yaml
use:
  components: "@shadcn/ui"                           # npm package
  icons: "lucide"                                     # icon library
  theme: "https://cdn.myapp.com/themes/dark.yaml"   # theme tokens URL
  layout: "dashboard"                                 # pre-built layout
  fonts:
    - "Inter"
    - "JetBrains Mono"
```

### Reference formats:

```yaml
# npm package name
use:
  components: "@shadcn/ui"

# URL
use:
  components: "https://unpkg.com/@my-lib/components"

# GitHub repo
use:
  components: "github:user/repo"

# Local path
use:
  components: "./my-components.yaml"

# Multiple sources (last wins on name conflict)
use:
  components:
    - "@shadcn/ui"
    - "./overrides"

# Cherry-pick
use:
  components:
    from: "@shadcn/ui"
    pick: [button, card, dialog]

# Inline HTML templates
use:
  components:
    product-card:
      template: '<div class="card"><h3>{{ input.name }}</h3></div>'
      style: '.card { border: 1px solid #30363d; }'
```

See `how-it-works.md` for full details on how the engine resolves and extracts components.

---

## Style Tokens

Reusable values referenced as `$style.*` anywhere in the template.

```yaml
style:
  primary: "#238636"
  accent: "#58a6ff"
  danger: "#f85149"
  surface: "#161b22"
  border: "#30363d"
  border-radius: 8
  spacing: 16
```

Usage:
```yaml
style:
  background: "$style.surface"
  border: "1px solid $style.border"
  border-radius: "$style.border-radius"
```

---

## Data Providers

Maps data names to provider functions registered by the host app.

```yaml
data:
  product: productDetailProvider
  user: userProfileProvider
  cart: cartProvider
  reviews: reviewsProvider
```

Provider data is accessed via `$providerName.field`:
```yaml
props:
  name: "$product.name"
  avatar: "$user.avatar"
  total: "$cart.total"
```

---

## Compute Section

Derived values computed from provider data. Referenced as `$compute.*`.

**Simple math:**
```yaml
compute:
  total: "$cart.subtotal + $cart.tax"
  item-count: "$cart.items.len()"
```

**Conditional (when):**
```yaml
compute:
  tier:
    when:
      - "$user.spend >= 1000": "gold"
      - "$user.spend >= 250": "silver"
      - else: "standard"
```

**Collection operations:**
```yaml
compute:
  subtotal: "$cart.items.sum(price * quantity)"
  expensive-items: "$cart.items.filter(price > 1000)"
  item-names: "$cart.items.map(name)"
  has-electronics: "$cart.items.any(category == electronics)"
  cheapest: "$cart.items.min(price)"
  sorted-by-price: "$cart.items.sort(price)"
```

**Cross-referencing computes:**
```yaml
compute:
  subtotal: "$cart.items.sum(price * quantity)"
  discount: "$compute.subtotal * 0.1"
  tax: "($compute.subtotal - $compute.discount) * 0.18"
  total: "$compute.subtotal - $compute.discount + $compute.tax"
```

Server evaluates these in dependency order using temple's DAG resolver.

---

## Regions

Layout containers that hold components or nested regions.

```yaml
regions:
  header:
    direction: horizontal       # horizontal | vertical | grid
    justify: space-between      # start | center | end | space-between
    gap: 16                     # spacing between children (px)
    style:
      padding: 16
      background: "$style.surface"
    components:
      - type: text
        props:
          content: "Hello"
```

### Region Properties

| Property | Type | Description |
|----------|------|-------------|
| `direction` | string | `horizontal`, `vertical`, `grid` |
| `justify` | string | `start`, `center`, `end`, `space-between` |
| `gap` | number | Spacing between children in px |
| `columns` | number | Grid column count (when direction = grid) |
| `collapse-below` | number | Hide region when viewport width < value |
| `stack-below` | number | Switch horizontal to vertical below this width |
| `layer` | number | Z-layer for overlay positioning (0 = normal, 1+ = overlay) |
| `style` | object | CSS-like style overrides |
| `condition` | condition | Show/hide based on data (see Conditions) |
| `data-source` | string | Provider name for list rendering |
| `template` | object | Component template for each item in data-source |
| `loading` | object | Component to show while data-source loads |
| `error` | object | Component to show on data-source error |
| `empty` | object | Component to show when data-source returns empty |
| `form` | boolean | Mark region as a form for validation |
| `validate-on` | string | `blur`, `change`, or `submit` |
| `components` | array | Child components |
| `regions` | object | Nested sub-regions |

### Nested Regions

```yaml
regions:
  header:
    direction: horizontal
    regions:
      brand:
        direction: horizontal
        components:
          - type: logo
            props:
              text: "ShopApp"
      actions:
        direction: horizontal
        components:
          - type: button
            props:
              text: "Cart"
```

### Grid Layout

```yaml
regions:
  product-grid:
    direction: grid
    columns: 3
    gap: 16
    components:
      - type: product-card
        colSpan: 2              # Spans 2 grid columns
        props:
          name: "Featured"
      - type: product-card
        props:
          name: "Product 2"
```

---

## Components

Leaf nodes in the layout tree. Each has a `type`, optional `id`, and `props`.

```yaml
components:
  - type: button
    id: add-to-cart
    props:
      text: "Add to Cart"
      variant: primary
    style:
      margin-top: 8
```

### Component Properties

| Property | Type | Description |
|----------|------|-------------|
| `type` | string | Component type name (mapped to native view by adapter) |
| `id` | string | Unique identifier (required for interactions and form fields) |
| `props` | object | Data passed to the component (resolved by server) |
| `span` | number | Proportional width in horizontal layouts |
| `colSpan` | number | Grid column span |
| `rowSpan` | number | Grid row span |
| `style` | object | CSS-like style overrides |
| `condition` | condition | Show/hide based on data |

### Built-in Primitive Types

These ship with every Orrery adapter:

| Type | Web | iOS | Android |
|------|-----|-----|---------|
| `text` | `<span>` / `<p>` | `UILabel` | `Text()` |
| `heading` | `<h1>`-`<h6>` | `UILabel` (styled) | `Text()` (styled) |
| `button` | `<button>` | `UIButton` | `Button()` |
| `input` | `<input>` / `<textarea>` | `UITextField` / `UITextView` | `TextField()` |
| `image` | `<img>` | `UIImageView` | `Image()` |
| `divider` | `<hr>` | `UIView` (1px) | `Divider()` |
| `tabs` | custom DOM | `UISegmentedControl` | `TabRow()` |

### Custom Components

Registered by the developer. See `components.md` for details.

---

## $References

All `$` prefixed strings are resolved by the server before output.

| Prefix | Source | Example |
|--------|--------|---------|
| `$data.*` or `$providerName.*` | Provider data | `$product.name`, `$cart.total` |
| `$style.*` | Style tokens | `$style.primary`, `$style.border-radius` |
| `$compute.*` | Computed values | `$compute.subtotal`, `$compute.tier` |
| `$event.*` | Event data (in interactions) | `$event.value`, `$event.quantity` |
| `$form.*` | Form field values (in validation/submit) | `$form.email`, `$form.password` |
| `$item.*` | Current item in data-source loop | `$item.name`, `$item.price` |
| `$self` | Current field value (in validation) | `$self` |
| `$field.*` | Other field values (cross-field validation) | `$field.password` |

### Nested Access

```yaml
props:
  name: "$product.details.name"
  first-tag: "$product.tags[0]"
  city: "$user.address.city"
```

### String Interpolation

```yaml
props:
  text: "Hello, $user.name!"
  label: "Items: $compute.item-count"
  warning: "Only $product.stock left in stock"
```

---

## Conditions

Show or hide regions/components based on provider data.

### Simple Truthy

```yaml
admin-panel:
  condition: "$user.isAdmin"
  components:
    - type: button
      props:
        text: "Delete All"
```

### Comparison Operators

```yaml
low-stock:
  condition:
    lt:
      - "$product.stock"
      - 5
```

| Operator | Meaning |
|----------|---------|
| `equals` | Equal to |
| `notEquals` | Not equal to |
| `gt` | Greater than |
| `lt` | Less than |
| `gte` | Greater than or equal |
| `lte` | Less than or equal |
| `contains` | Array contains item |
| `in` | Item is in array |
| `matches` | Regex match |
| `length` | Array/string length check |

### Logical Operators

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

### Component-Level Conditions

```yaml
components:
  - type: button
    id: add-to-cart
    condition:
      not:
        contains:
          - "$cart.itemIds"
          - "$product.id"
    props:
      text: "Add to Cart"

  - type: button
    id: remove-from-cart
    condition:
      contains:
        - "$cart.itemIds"
        - "$product.id"
    props:
      text: "Remove"
```

---

## Data-Source Lists

Loop over provider data to render repeated components.

```yaml
reviews-section:
  data-source: reviews
  template:
    type: review-card
    props:
      author: "$item.author"
      text: "$item.text"
      rating: "$item.rating"
```

### Loading / Error / Empty States

```yaml
reviews-section:
  data-source: reviews
  template:
    type: review-card
    props:
      author: "$item.author"
  loading:
    type: skeleton
    count: 3
  error:
    type: error-banner
    props:
      message: "$reviews.error"
  empty:
    type: empty-state
    props:
      text: "No reviews yet"
```

---

## Form Validation

Mark a region as a form and add validation to its components.

```yaml
login-form:
  direction: vertical
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
```

### Validation Shorthands

| Shorthand | Expands to |
|-----------|-----------|
| `required: true` | `notEquals: [$self, ""]` |
| `min-length: 8` | `gte: [length($self), 8]` |
| `max-length: 100` | `lte: [length($self), 100]` |
| `min: 18` | `gte: [$self, 18]` |
| `max: 100` | `lte: [$self, 100]` |
| `matches: "regex"` | `matches: [$self, "regex"]` |

### Full Condition Validation

```yaml
validate:
  condition:
    and:
      - gte:
          - "$self"
          - 1
      - lte:
          - "$self"
          - 5
  error: "Rating must be between 1 and 5"
```

### Cross-Field Validation

```yaml
- type: input
  id: confirm-password
  validate:
    condition:
      equals:
        - "$self"
        - "$field.password"
    error: "Passwords don't match"
```

---

## Interactions

Event bindings: built-in actions or handler calls.

### Built-in Actions

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

  # Submit a form (validate then call handler)
  - on: submit-btn.click
    do: submit
    target: login-form
    handler: login
    args:
      email: "$form.email"
      password: "$form.password"
```

### Handler Calls

```yaml
interactions:
  - on: add-to-cart.click
    handler: addToCart
    args:
      product_id: "$product.id"
      quantity: 1
      price: "$product.price"
    then:
      - do: refresh
        target: cart
      - do: notify
        message: "Added to cart!"

  - on: delete-btn.click
    handler: deleteProduct
    args:
      product_id: "$product.id"
    then:
      - do: hide
        target: confirm-dialog
      - do: navigate
        to: shop.yaml
      - do: notify
        message: "Deleted"
```

### Then Chains

Multiple actions executed in sequence after a handler completes:

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

### Platform Extensions

For native-only capabilities (camera, biometrics, share):

```yaml
- on: take-photo.click
  do: extension
  name: camera
  then:
    - handler: uploadPhoto
      args:
        image: "$extension.result"
```

Extensions are registered per-platform by the adapter developer.

---

## Transitions (Planned)

Composable animations from built-in primitives.

```yaml
# Define custom transitions at page level
transitions:
  slide-fade-in:
    - fade-in
    - slide-from: right
    duration: 400
    easing: ease-out

# Use on regions/components
cart-modal:
  layer: 1
  transition:
    enter: slide-fade-in
    exit: fade-out

# Staggered enter for lists
product-grid:
  transition:
    enter: pop-in
    stagger: 50
```
