# Rendering

How the view tree becomes native UI on each platform.

---

## The Pipeline

```
Server output
      │
      ├── Web: HTML string
      │         │
      │         v
      │    innerHTML → event binding → virtual scroll
      │
      ├── iOS: FlatBuffers (prod) / JSON (debug)
      │         │
      │         v
      │    Decode → walk tree → UIKit view hierarchy → UICollectionView for lists
      │
      └── Android: FlatBuffers (prod) / JSON (debug)
                │
                v
           Decode → walk tree → Compose composables → LazyColumn for lists
```

---

## Web Rendering

### Server Sends HTML

The server renders the complete page as an HTML string. Layout regions become `<div>` elements with flex/grid styles. Components become their HTML templates with resolved props.

```html
<!-- Server output for a header region -->
<div data-region="header" style="display:flex;justify-content:space-between;gap:16px;padding:16px;background:#161b22">
  <div data-region="brand" style="display:flex;gap:12px">
    <span data-id="logo" data-type="0">ShopApp</span>
    <input data-id="search-bar" data-type="2" placeholder="Search products..." />
  </div>
  <div data-region="actions" style="display:flex;gap:8px">
    <button data-id="cart-btn" data-type="1">Cart (3)</button>
  </div>
</div>
```

### Client Mounts

```typescript
// @orrery/web internals (simplified)
function mount(container, html, interactions) {
  // 1. Inject HTML — browser parses at 152 MB/s
  container.innerHTML = html

  // 2. Bind events from interaction definitions
  for (const interaction of interactions) {
    const [id, event] = interaction.on.split('.')
    const el = container.querySelector(`[data-id="${id}"]`)
    el.addEventListener(event, (e) => handleInteraction(interaction, e))
  }

  // 3. Initialize virtual scroll for data-source regions with many items
  for (const region of findDataSourceRegions(container)) {
    if (region.itemCount > 200) {
      initVirtualScroll(region)
    }
  }
}
```

### Virtual Scroll (Web)

When a data-source region has more than ~200 items, virtual scroll activates:

- Only ~20-30 DOM nodes exist at any time
- Scroll position tracked, items swapped in/out as user scrolls
- Uses `IntersectionObserver` (off main thread) instead of scroll events
- 10K items: same performance as 20 items

### Diff Updates (Web)

When data changes:
1. Server re-renders affected regions
2. Sends HTML diff: `{ region: "cart", html: "<new cart html>" }`
3. Client replaces only that region's innerHTML
4. Re-binds events for the replaced region

---

## iOS Rendering

### Decode

**FlatBuffers (production):**
Zero-copy access. No parsing. Read fields directly from the byte buffer.

```swift
let buffer = response.data
let viewTree = ViewTree.getRootAsViewTree(bb: ByteBuffer(data: buffer))
// No allocation, no parsing — fields read by offset
let firstRegion = viewTree.regions(at: 0)
let direction = firstRegion?.direction  // reads directly from buffer
```

**JSON (debug):**
Standard JSONDecoder. Slower but human-readable.

```swift
let viewTree = try JSONDecoder().decode(ViewTree.self, from: response.data)
```

### View Hierarchy Construction

Walk the view tree top-down. Regions become UIStackView or manual frame layout. Components become native views via constructor array lookup.

```swift
func buildView(node: ViewNode) -> UIView {
    switch node {
    case .region(let region):
        let container = UIStackView()
        container.axis = region.direction == .horizontal ? .horizontal : .vertical
        container.spacing = CGFloat(region.gap)
        for child in region.children {
            container.addArrangedSubview(buildView(node: child))
        }
        return container

    case .component(let component):
        let constructor = constructors[component.typeId]
        return constructor(component.props)
    }
}
```

### Lists (UICollectionView)

Data-source regions with many items use UICollectionView with cell recycling:

- `dequeueReusableCell` for visible cells (~15 on screen)
- `prepareForReuse` + reconfigure on scroll
- ~568-2,695 cells/sec throughput
- 10K items: same memory as 15 items

UICollectionView is used, not SwiftUI List or LazyVStack. UIKit provides:
- 2.7x less memory than SwiftUI
- 5x fewer scroll hitches
- Cell recycling (SwiftUI LazyVStack leaks memory)

### Diff Updates (iOS)

1. Server sends changed nodes with their region paths
2. Client walks existing view hierarchy to the target region
3. Removes old children, builds new children from diff
4. Animates the transition if configured

---

## Android Rendering

### Decode

**FlatBuffers (production):**
```kotlin
val buffer = ByteBuffer.wrap(response.body)
val viewTree = ViewTree.getRootAsViewTree(buffer)
// Zero-copy access — same as iOS
```

**JSON (debug):**
```kotlin
val viewTree = moshi.adapter(ViewTree::class.java).fromJson(response.body)
```

### Compose Tree Construction

```kotlin
@Composable
fun RenderNode(node: ViewNode) {
    when (node) {
        is RegionNode -> {
            when (node.direction) {
                Direction.HORIZONTAL -> Row(
                    horizontalArrangement = Arrangement.spacedBy(node.gap.dp)
                ) {
                    node.children.forEach { RenderNode(it) }
                }
                Direction.VERTICAL -> Column(
                    verticalArrangement = Arrangement.spacedBy(node.gap.dp)
                ) {
                    node.children.forEach { RenderNode(it) }
                }
                Direction.GRID -> LazyVerticalGrid(
                    columns = GridCells.Fixed(node.columns)
                ) {
                    items(node.children) { RenderNode(it) }
                }
            }
        }
        is ComponentNode -> {
            val factory = factories[node.typeId]
            factory(node.props)
        }
    }
}
```

### Lists (LazyColumn)

Data-source regions use LazyColumn:

```kotlin
LazyColumn {
    items(
        count = dataItems.size,
        key = { dataItems[it].id }  // keys prevent 850ms → 45ms update regression
    ) { index ->
        val factory = factories[dataItems[index].typeId]
        factory(dataItems[index].props)
    }
}
```

- Compose 1.9+: 0.2% jank rate, matching RecyclerView
- P50 frame CPU: 4.8ms, P99: 15.3ms
- Keys are mandatory for performance

### Low-End Android (SD 400, 2GB RAM)

On low-end devices, Compose initial render is 19-36% slower than XML Views. Options:

1. Use Compose (simpler code, acceptable for most apps)
2. Use XML Views + RecyclerView (faster startup on budget devices)

The adapter can support both — the type ID constructor array points to either Compose factories or XML ViewHolder factories.

### Diff Updates (Android)

1. Server sends diff
2. Client updates the Compose state (triggers recomposition only for changed nodes)
3. Compose's smart recomposition skips unchanged composables

---

## The Mapper Code

Each adapter has a constructor/factory array indexed by type ID.

### Web Mapper

```typescript
// Built-in primitives
const primitives: Record<number, (props: Props) => string> = {
  0: (p) => `<span>${p.content}</span>`,                           // text
  1: (p) => `<button class="${p.variant}">${p.text}</button>`,     // button
  2: (p) => `<input placeholder="${p.placeholder}" />`,            // input
  3: (p) => `<img src="${p.src}" alt="${p.alt || ''}" />`,         // image
  4: (p) => `<h${p.level || 2}>${p.text}</h${p.level || 2}>`,    // heading
  5: () => `<hr />`,                                               // divider
}

// Custom components registered by developer are added to the same array
// Server-rendered HTML templates replace client-side primitives for web
```

For web, the server already outputs complete HTML including component HTML. The client mapper is only used for diff updates and client-registered components (Option B from components.md).

### iOS Mapper

```swift
let constructors: [(Props) -> UIView] = [
    // 0: text
    { props in
        let label = UILabel()
        label.text = props["content"] as? String
        return label
    },
    // 1: button
    { props in
        let button = UIButton(type: .system)
        button.setTitle(props["text"] as? String, for: .normal)
        return button
    },
    // 2: input
    { props in
        let field = UITextField()
        field.placeholder = props["placeholder"] as? String
        return field
    },
    // 3: image
    { props in
        let imageView = UIImageView()
        imageView.loadURL(props["src"] as? String)
        return imageView
    },
    // ... primitives + custom components appended
]
```

### Android Mapper

```kotlin
val factories: Array<@Composable (Props) -> Unit> = arrayOf(
    // 0: text
    { props -> Text(text = props["content"] as String) },
    // 1: button
    { props -> Button(onClick = {}) { Text(props["text"] as String) } },
    // 2: input
    { props -> TextField(value = "", onValueChange = {}, placeholder = { Text(props["placeholder"] as String) }) },
    // 3: image
    { props -> AsyncImage(model = props["src"]) },
    // ... primitives + custom components appended
)
```

---

## Type ID Synchronization

The server and client must agree on type ID assignments.

**Initial handshake:**
```json
// Server sends type map in first response (or as a separate endpoint)
{
  "type_map": {
    "text": 0,
    "button": 1,
    "input": 2,
    "image": 3,
    "heading": 4,
    "divider": 5,
    "product-card": 6,
    "cart-summary": 7
  }
}
```

The client adapter uses this map to build its constructor array in the correct order. After the handshake, all communication uses integer IDs only.

The type map is versioned. If it changes (new component types added), the server bumps the version and the client rebuilds its array.
