# Component System

How components are defined, registered, referenced, and rendered.

---

## How Components Work

A component in YAML is just a type name and props:

```yaml
- type: product-card
  props:
    name: "$product.name"
    price: "$product.price"
    image: "$product.image"
```

The server resolves `$references` and outputs the component with its integer type ID and final prop values. The client adapter looks up the type ID and renders the native view.

```
YAML author writes:  type: "product-card"
                     props: { name: "$product.name" }
                           │
Server resolves:     type_id: 4
                     props: { name: "Wireless Headphones" }
                           │
Client adapter:      constructors[4](props)
                     → native view with "Wireless Headphones"
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

If you have components in React, Vue, Svelte, or any framework — the engine rewrites them to HTML templates internally. The user references the component library in YAML, and the server handles the conversion.

```yaml
# User references their component library
components:
  from: my-design-system        # component library name

regions:
  main:
    components:
      - type: product-card      # component from the library
        props:
          name: "$product.name"
          price: "$product.price"
```

The developer registers the component library with the server. The server knows how to render each component to HTML — no framework runtime shipped to the client.

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
        // ... more components
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
