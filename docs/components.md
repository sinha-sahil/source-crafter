# Component System

How components are defined, referenced, and rendered.

---

## How Components Work

The YAML author references components from a library declared in `use:`:

```yaml
page: shop

use:
  components: "@shadcn/ui"         # component library
  icons: "lucide"                   # icon library

regions:
  main:
    components:
      - type: button                # from @shadcn/ui
        props:
          text: "Add to Cart"
          variant: "primary"
      - type: product-card          # from @shadcn/ui
        props:
          name: "$product.name"
          price: "$product.price"
      - type: icon                  # from lucide
        props:
          name: "shopping-cart"
```

At compile time, the engine:
1. Fetches the component library
2. Extracts HTML templates via SSR (see `how-it-works.md`)
3. Compiles templates to temple blobs
4. At pre-render time, executes blobs with resolved props → HTML

```
YAML author writes:  use: components: "@shadcn/ui"
                     type: "product-card"
                     props: { name: "$product.name" }
                           │
Engine at compile:   SSR extract → HTML template → temple blob
                           │
Engine at pre-render: blob + { name: "Wireless Headphones" }
                           │
Output .html:        <div class="card"><h3>Wireless Headphones</h3>...</div>
                           │
Browser:             innerHTML → user sees the card
```

## Two Kinds of Components

### 1. Primitives (shipped with Orrery)

Built into every adapter. Work out of the box.

| Type | Description | Web | iOS | Android |
|------|-------------|-----|-----|---------|
| `text` | Text display | `<span>` / `<p>` | `UILabel` | `Text()` |
| `heading` | Heading text | `<h1>`-`<h6>` | `UILabel` styled | `Text()` styled |
| `button` | Clickable button | `<button>` | `UIButton` | `Button()` |
| `input` | Text input | `<input>` / `<textarea>` | `UITextField` / `UITextView` | `TextField()` |
| `image` | Image display | `<img>` | `UIImageView` | `Image()` / `AsyncImage()` |
| `divider` | Horizontal rule | `<hr>` | `UIView` (1px) | `Divider()` |
| `tabs` | Tab selector | Custom DOM | `UISegmentedControl` | `TabRow()` |
| `avatar` | User avatar | Custom DOM | Custom `UIView` | Custom composable |
| `skeleton` | Loading placeholder | Custom DOM | Custom `UIView` | Custom composable |

### 2. Custom Components (registered by the developer)

The developer builds these and registers them with the adapter.

---

## Registering Custom Components

### Web — HTML Template

Components are HTML templates. Props are injected by the server.

**Option A: Define in a components YAML file:**
```yaml
# components.yaml
components:
  product-card:
    template: |
      <div class="card">
        <img src="{{ input.image }}" />
        <h3>{{ input.name }}</h3>
        <span class="price">{{ input.price }}</span>
      </div>
    style: |
      .card { border: 1px solid #30363d; border-radius: 8px; padding: 16px; }
      .card h3 { margin: 8px 0 4px; }
      .price { color: #3fb950; font-weight: bold; }

  cart-summary:
    template: |
      <div class="cart-summary">
        <span>Items: {{ input.itemCount }}</span>
        <span class="total">Total: {{ input.total }}</span>
      </div>
    style: |
      .cart-summary { display: flex; justify-content: space-between; }
      .total { font-weight: bold; }
```

The server compiles these templates to temple blobs at deploy time. At request time, it renders props into the HTML template in ~1 microsecond.

**Option B: Register programmatically on the client:**
```typescript
import { mount, registerComponent } from '@orrery/web'

registerComponent('product-card', (el, props) => {
  el.innerHTML = `
    <div class="card">
      <img src="${props.image}" />
      <h3>${props.name}</h3>
      <span class="price">${props.price}</span>
    </div>
  `
})

mount('#app', { endpoint: '/api/orrery/shop' })
```

Option A is preferred — server renders the HTML, client just does innerHTML. Option B runs on the client and is slightly slower.

### iOS — UIView Factory

```swift
import Orrery

OrreryAdapter.register("product-card") { props in
    let card = UIView()
    let imageView = UIImageView()
    imageView.load(url: props["image"])
    let nameLabel = UILabel()
    nameLabel.text = props["name"] as? String
    let priceLabel = UILabel()
    priceLabel.text = props["price"] as? String
    // layout code...
    card.addSubview(imageView)
    card.addSubview(nameLabel)
    card.addSubview(priceLabel)
    return card
}
```

### Android — Composable Factory

```kotlin
import com.orrery.android.OrreryAdapter

OrreryAdapter.register("product-card") { props ->
    Column(modifier = Modifier.padding(16.dp)) {
        AsyncImage(model = props["image"])
        Text(text = props["name"] as String, style = MaterialTheme.typography.titleMedium)
        Text(text = props["price"] as String, color = Color(0xFF3FB950))
    }
}
```

---

## Using Existing Component Libraries

Reference any component library in YAML. The server handles everything — downloading, detecting the framework, extracting HTML templates:

```yaml
page: shop

use:
  components: "@shadcn/ui"      # npm package name

regions:
  main:
    components:
      - type: product-card      # from the library
        props:
          name: "$product.name"
          price: "$product.price"
```

The server auto-detects which extraction tier to use:

### Three Tiers of Extraction

| Tier | What the Library Ships | What Orrery Does | Examples |
|------|----------------------|-----------------|----------|
| **1. HTML Templates** | `.html` files, template strings | Uses directly — zero extraction | Custom Orrery-native libs |
| **2. Web Components** | Custom element JS (`<sui-card>`) | Template = `<sui-card text="{{ input.text }}">`, ships WC bundle to browser | `@juspay/svelte-ui-components` (Web Component builds) |
| **3. Framework SSR** | `.jsx`, `.svelte`, `.vue` source | esbuild bundles framework + component → QuickJS executes SSR → HTML string → template | Most npm libraries (`@shadcn/ui`, `@radix/ui`, etc.) |

### How Tier 3 Works (Framework SSR)

The server is pure Rust. It embeds QuickJS (lightweight JS runtime) and esbuild (JS bundler) to extract HTML:

```
1. Download component library from npm registry (HTTP, no npm CLI)
2. Read package.json → detect framework (React/Svelte/Vue)
3. Download the framework package from npm registry
4. Generate entry.js that imports framework SSR + components
5. esbuild bundles entry.js + framework + components → single bundle.js
6. QuickJS executes bundle.js → calls SSR function with marker props → HTML string
7. Replace markers with temple placeholders → HTML template
8. Compile template → temple blob (pure Rust)
9. Cache everything — next startup skips extraction entirely
```

See `how-it-works.md` for the detailed step-by-step with code examples.

### Real-World Example: @juspay/svelte-ui-components

This library ships 67 raw `.svelte` files (Svelte 5 runes syntax, NOT compiled JS). It also has Web Component wrappers (`<sui-card>`, `<sui-button>`).

Orrery can use it two ways:

```yaml
# Option A: Web Component path (Tier 2, simpler)
use:
  components:
    from: "@juspay/svelte-ui-components"
    mode: "web-components"

# Option B: SSR extraction path (Tier 3, general)
use:
  components: "@juspay/svelte-ui-components"
  # Server auto-detects Svelte, downloads svelte compiler,
  # compiles .svelte → JS → SSR render → HTML templates
```

### Manual Registration (Optional)

Developers can also register component templates directly in Rust:

```rust
server.register_component_library("my-design-system", ComponentLibrary {
    components: vec![
        ComponentDef {
            name: "product-card",
            template: r#"
                <div class="card">
                    <img src="{{ input.image }}" />
                    <h3>{{ input.name }}</h3>
                    <span class="price">{{ input.price }}</span>
                </div>
            "#,
            style: ".card { border: 1px solid #30363d; border-radius: 8px; padding: 16px; }",
        },
    ],
});
```

The HTML templates are compiled to temple blobs at registration time. At request time, props are injected and HTML is rendered in microseconds. No React, no Vue, no Svelte — just HTML.

---

## Component Resolution Order

When the adapter encounters a component type:

```
1. Check registered custom components → found? → render custom
2. Check built-in primitives          → found? → render primitive
3. Not found                          → debug: show error placeholder
                                        prod: skip silently
```

---

## Integer Type ID Mapping

The server assigns integer IDs to component types. The adapter maps IDs to constructors via array index.

**Server (at startup):**
```
Scan all component types used across all YAML templates
Assign sequential IDs:
  0 → text
  1 → button
  2 → input
  3 → image
  4 → heading
  5 → divider
  6 → product-card
  7 → cart-summary
  ...
```

**Client adapter:**
```
constructors = [
  TextRenderer,         // [0] text
  ButtonRenderer,       // [1] button
  InputRenderer,        // [2] input
  ImageRenderer,        // [3] image
  HeadingRenderer,      // [4] heading
  DividerRenderer,      // [5] divider
  ProductCardRenderer,  // [6] product-card
  CartSummaryRenderer,  // [7] cart-summary
]

// Render: constructors[node.type_id](node.props)
```

Array index lookup: O(1). No string comparison. No HashMap overhead.

The type ID mapping is sent once to the client (as part of the initial handshake or embedded in the first response). After that, all view tree nodes use integer IDs only.

---

## Props Resolution

All props are resolved by the server before the view tree is sent to the client. The client never sees `$references`.

```yaml
# YAML author writes:
- type: product-card
  props:
    name: "$product.name"
    price: "$product.price"
    in-cart: "$cart.itemIds.any(id == $product.id)"
    discount: "$compute.discount"
```

```json
// Client receives (in view tree):
{
  "type_id": 6,
  "props": {
    "name": "Wireless Headphones",
    "price": "4999",
    "in-cart": true,
    "discount": "499.90"
  }
}
```

The client adapter passes `props` directly to the component constructor. No resolution, no evaluation, no string parsing on the client.
