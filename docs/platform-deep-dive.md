# Platform Deep Dive

Complete flow from YAML to rendered UI on iOS, Android, and Browser — what the server produces, what the adapter receives, and how every piece connects.

---

## What the Server Produces

Given this YAML:

```yaml
page: shop

data:
  products: productsProvider
  cart: cartProvider                    # mode: "live" — per-user, fetched at runtime

compute:
  cartLabel: "Cart ($cart.count)"

regions:
  header:
    direction: horizontal
    style:
      padding: 16
      background: "#161b22"
    components:
      - type: heading
        id: shop-title
        props:
          text: "Shop"
          level: 1
      - type: button
        id: cart-btn
        props:
          text: "$compute.cartLabel"

  products:
    direction: vertical
    data-source: products
    template:
      type: card
      props:
        title: "$item.name"
        price: "$item.price"
        image: "$item.images[0]"
    loading:
      type: skeleton
      count: 3
    empty:
      type: text
      props:
        content: "No products found"

interactions:
  - on: cart-btn.click
    do: navigate
    to: cart.yaml

  - on: card.click
    handler: addToCart
    args:
      product_id: "$item.id"
      quantity: 1
    then:
      - do: refresh
        target: cart
      - do: notify
        message: "Added to cart!"
```

### For Web → `.html` + `.css` + `interactions.js`

The server resolves all `$references`, evaluates conditions, and produces complete HTML:

```html
<!-- /pages/shop/v5.html -->
<div data-region="header" style="display:flex;gap:16px;padding:16px;background:#161b22">
  <h1 data-id="shop-title" data-type="0">Shop</h1>
  <button data-id="cart-btn" data-type="1" data-live="cart.count" data-expression="Cart ($cart.count)">Cart (...)</button>
</div>
<div data-region="products" style="display:flex;flex-direction:column;gap:12px">
  <div data-id="card-0" data-type="2" class="card">
    <img src="https://cdn.myapp.com/headphones.jpg" />
    <h3>Headphones</h3>
    <span class="price">₹4999</span>
  </div>
  <div data-id="card-1" data-type="2" class="card">
    <img src="https://cdn.myapp.com/keyboard.jpg" />
    <h3>Keyboard</h3>
    <span class="price">₹2499</span>
  </div>
  <div data-id="card-2" data-type="2" class="card">
    <img src="https://cdn.myapp.com/mouse.jpg" />
    <h3>Mouse</h3>
    <span class="price">₹999</span>
  </div>
</div>
```

```javascript
// /pages/shop/v5.interactions.js
[
  {
    "on": "cart-btn.click",
    "do": "navigate",
    "to": "cart.yaml"
  },
  {
    "on": "card-0.click",
    "handler": "addToCart",
    "args": { "product_id": "hp-001", "quantity": 1 },
    "then": [
      { "do": "refresh", "target": "cart" },
      { "do": "notify", "message": "Added to cart!" }
    ]
  },
  {
    "on": "card-1.click",
    "handler": "addToCart",
    "args": { "product_id": "kb-001", "quantity": 1 },
    "then": [
      { "do": "refresh", "target": "cart" },
      { "do": "notify", "message": "Added to cart!" }
    ]
  },
  {
    "on": "card-2.click",
    "handler": "addToCart",
    "args": { "product_id": "ms-001", "quantity": 1 },
    "then": [
      { "do": "refresh", "target": "cart" },
      { "do": "notify", "message": "Added to cart!" }
    ]
  }
]
```

```json
// /pages/shop/v5.live.json
{
  "providers": [
    {
      "name": "cart",
      "url": "/api/cart",
      "mode": "live"
    }
  ],
  "bindings": [
    {
      "elementId": "cart-btn",
      "attribute": "textContent",
      "expression": "Cart ($cart.count)",
      "provider": "cart",
      "field": "count"
    }
  ]
}
```

### For iOS / Android → FlatBuffer Binary

The server produces a single FlatBuffer binary (~1-3KB) containing seven sections:

```
FlatBuffer binary
│
├── 1. TYPE MAP
│   { "heading": 0, "button": 1, "card": 2, "text": 3, "skeleton": 4, "image": 5, "input": 6 }
│
├── 2. LIVE PROVIDERS
│   [{ name: "cart", url: "/api/cart", mode: "live" }]
│
├── 3. INTERACTIONS
│   [
│     { viewId: "cart-btn", event: "click", do: "navigate", to: "cart.yaml" },
│     { viewId: "card-0", event: "click", handler: "addToCart",
│       args: { product_id: "hp-001", quantity: 1 },
│       then: [{ do: "refresh", target: "cart" }, { do: "notify", message: "Added to cart!" }] },
│     { viewId: "card-1", event: "click", handler: "addToCart",
│       args: { product_id: "kb-001", quantity: 1 },
│       then: [{ do: "refresh", target: "cart" }, { do: "notify", message: "Added to cart!" }] },
│     { viewId: "card-2", event: "click", handler: "addToCart",
│       args: { product_id: "ms-001", quantity: 1 },
│       then: [{ do: "refresh", target: "cart" }, { do: "notify", message: "Added to cart!" }] }
│   ]
│
├── 4. VIEW TREE
│   Region(id: "header", direction: horizontal, style: { padding: 16, background: "#161b22" })
│   ├── Component(id: "shop-title", typeId: 0, props: { text: "Shop", level: 1 },
│   │     liveBindings: [])
│   └── Component(id: "cart-btn", typeId: 1, props: { text: "Cart (...)" },
│         liveBindings: [{ prop: "text", expression: "Cart ($cart.count)",
│                          provider: "cart", field: "count" }])
│
│   Region(id: "products", direction: vertical, style: { gap: 12 })
│   ├── Component(id: "card-0", typeId: 2, props: {
│   │     title: "Headphones", price: "₹4999",
│   │     image: "https://cdn.myapp.com/headphones.jpg" })
│   ├── Component(id: "card-1", typeId: 2, props: {
│   │     title: "Keyboard", price: "₹2499",
│   │     image: "https://cdn.myapp.com/keyboard.jpg" })
│   └── Component(id: "card-2", typeId: 2, props: {
│         title: "Mouse", price: "₹999",
│         image: "https://cdn.myapp.com/mouse.jpg" })
│
├── 5. STYLES
│   { colors: { primary: "#238636", surface: "#161b22", text: "#e6edf3", border: "#30363d" },
│     spacing: { sm: 8, md: 16, lg: 24 },
│     radius: { md: 8 },
│     typography: { fontFamily: "Inter", bodySize: 14 } }
│
├── 6. NAVIGATION
│   { currentPage: "shop",
│     availablePages: [
│       { name: "shop", url: "/pages/shop/v5.fb" },
│       { name: "cart", url: "/pages/cart/v2.fb" },
│       { name: "home", url: "/pages/home/v3.fb" }
│     ],
│     prefetch: ["cart"] }
│
└── 7. PAGE META
    { version: "v5", title: "Shop", pullToRefresh: true, refreshProvider: "products" }
```

Static data (`$products`) is resolved — "Headphones", "₹4999" are baked in.
Live data (`$cart.count`) is NOT resolved — a liveBinding tells the adapter what to fetch.

---

## Browser Flow (@orrery/web)

### Step 1: First Page Load

The browser loads a static shell (deployed once, never changes):

```html
<!DOCTYPE html>
<html>
<head>
  <link rel="stylesheet" href="https://cdn/assets/components.css" />
  <link rel="stylesheet" href="https://cdn/assets/theme.css" />
  <script src="https://cdn/assets/orrery-runtime.js"></script>
</head>
<body>
  <div id="app"></div>
  <script>orrery.mount('#app', { cdn: 'https://cdn', startPage: 'shop' })</script>
</body>
</html>
```

`orrery-runtime.js` is `@orrery/web` (~5-8KB gzipped). It does everything:

```typescript
async function mount(selector: string, config: Config) {
  const container = document.querySelector(selector)

  // 1. Fetch manifest from CDN (no-cache, always fresh, ~500 bytes)
  const manifest = await fetch(`${config.cdn}/manifest.json`).then(r => r.json())
  sessionStorage.setItem('orrery:manifest', JSON.stringify(manifest))

  // 2. Fetch pre-rendered page files from CDN
  const pageBase = `${config.cdn}${manifest.pages[config.startPage]}`
  const [html, interactionsText, liveConfig] = await Promise.all([
    fetch(`${pageBase}.html`).then(r => r.text()),
    fetch(`${pageBase}.interactions.js`).then(r => r.json()),
    fetch(`${pageBase}.live.json`).then(r => r.json()).catch(() => null)
  ])

  // 3. Inject HTML — browser parses at ~152 MB/s
  container.innerHTML = html
  // USER SEES THE PAGE NOW (static data visible)

  // 4. Bind events from interactions
  for (const interaction of interactionsText) {
    const [id, event] = interaction.on.split('.')
    const el = container.querySelector(`[data-id="${id}"]`)
    el.addEventListener(event, () => handleInteraction(interaction))
  }

  // 5. Fetch live providers
  if (liveConfig) {
    for (const provider of liveConfig.providers) {
      const data = await fetch(`${config.apiBase}${provider.url}`, {
        headers: { 'Authorization': `Bearer ${getToken()}` }
      }).then(r => r.json())
      // data = { count: 3, items: [...] }

      // Fill in live bindings
      for (const binding of liveConfig.bindings) {
        if (binding.provider === provider.name) {
          const el = container.querySelector(`[data-id="${binding.elementId}"]`)
          const value = data[binding.field]
          const resolved = binding.expression.replace(`$${provider.name}.${binding.field}`, value)
          el[binding.attribute] = resolved
          // "Cart ($cart.count)" → "Cart (3)"
        }
      }
    }
  }

  // 6. Prefetch linked pages
  for (const interaction of interactionsText) {
    if (interaction.do === 'navigate' && manifest.pages[interaction.to]) {
      const link = document.createElement('link')
      link.rel = 'prefetch'
      link.href = `${config.cdn}${manifest.pages[interaction.to]}`
      document.head.appendChild(link)
    }
  }
}
```

### Step 2: User Clicks "Add to Cart" on Headphones

```typescript
async function handleInteraction(interaction: Interaction) {
  if (interaction.do) {
    // Built-in action — no server call
    switch (interaction.do) {
      case 'show':
        document.querySelector(`[data-region="${interaction.target}"]`).style.display = ''
        break
      case 'hide':
        document.querySelector(`[data-region="${interaction.target}"]`).style.display = 'none'
        break
      case 'navigate':
        await navigateTo(interaction.to)
        break
      case 'notify':
        showToast(interaction.message)
        break
    }
  }

  if (interaction.handler) {
    // Server call
    const response = await fetch(`${serverURL}/orrery/handler`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        handler: interaction.handler,   // "addToCart"
        args: interaction.args,         // { product_id: "hp-001", quantity: 1 }
        page: currentPage,
        session: sessionToken
      })
    }).then(r => r.json())

    // Apply server response (diff)
    if (response.diff) {
      for (const patch of response.diff) {
        const region = document.querySelector(`[data-region="${patch.region}"]`)
        region.innerHTML = patch.content
        rebindEvents(region)
      }
    }

    // Execute then chain
    if (interaction.then) {
      for (const action of interaction.then) {
        await handleInteraction(action)
      }
    }
  }
}
```

### Step 3: Navigation

```typescript
async function navigateTo(pageName: string) {
  const manifest = JSON.parse(sessionStorage.getItem('orrery:manifest'))
  const pageURL = manifest.pages[pageName]

  // Fetch from CDN (may already be prefetched → instant)
  const [html, interactions] = await Promise.all([
    fetch(`${cdnURL}${pageURL}.html`).then(r => r.text()),
    fetch(`${cdnURL}${pageURL}.interactions.js`).then(r => r.json())
  ])

  container.innerHTML = html
  bindEvents(interactions)
  history.pushState({}, '', `/${pageName}`)
  prefetchLinkedPages(interactions)
}
```

### Web Timeline

```
0ms       mount() called
20-40ms   manifest.json fetched from CDN
40-80ms   .html + .interactions.js + .live.json fetched from CDN
80ms      container.innerHTML = html → USER SEES PRODUCTS
82ms      Events bound (click handlers attached)
82ms      Live provider fetch starts: GET /api/cart
200ms     Cart API responds → "Cart (3)" filled in
200ms     Prefetch starts for linked pages

Total to first paint: ~80ms
Total with live data: ~200ms
```

---

## iOS Flow (@orrery/ios)

### Step 1: App Launch

```swift
// Developer's AppDelegate — their only Orrery code
import Orrery

class AppDelegate: UIApplicationDelegate {
    let adapter = OrreryAdapter(
        cdnURL: "https://cdn.myapp.com",
        apiBaseURL: "https://api.myapp.com"
    )

    func application(_ application: UIApplication,
                     didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        adapter.renderPage("shop", in: window!)
        return true
    }
}
```

### Step 2: Adapter Fetches and Decodes

```swift
// Inside @orrery/ios

func renderPage(_ pageName: String, in window: UIWindow) {
    Task {
        // 1. Fetch manifest from CDN
        let manifestData = try await fetch("\(cdnURL)/manifest.json")
        let manifest = try JSONDecoder().decode(RouteManifest.self, from: manifestData)

        // 2. Fetch FlatBuffer from CDN
        let pageURL = manifest.pages[pageName]!
        let pageData = try await fetch("\(cdnURL)\(pageURL)")

        // 3. Zero-copy decode — no parsing, no allocations
        let fb = ViewTree.getRootAsViewTree(bb: ByteBuffer(data: pageData))
        // fb points directly into pageData bytes. Nothing was copied.

        // 4. Read type map → build constructor array
        let typeMap = fb.typeMap
        buildConstructorArray(typeMap)
        // constructors[0] = renderHeading (built-in)
        // constructors[1] = renderButton (built-in)
        // constructors[2] = renderCard (built-in)

        // 5. Read styles → set up theme
        let styles = fb.styles
        Theme.current = Theme(
            primary: UIColor(hex: styles.colors.primary),
            surface: UIColor(hex: styles.colors.surface),
            textColor: UIColor(hex: styles.colors.text),
            fontFamily: styles.typography.fontFamily,
            bodySize: CGFloat(styles.typography.bodySize)
        )

        // 6. Read page meta → configure screen
        let meta = fb.pageMeta
        let vc = UIViewController()
        vc.title = meta.title  // "Shop"

        if meta.pullToRefresh {
            let refresh = UIRefreshControl()
            refresh.addTarget(self, action: #selector(onPullToRefresh))
            scrollView.refreshControl = refresh
        }

        // 7. Walk view tree → build UIKit views
        let pageView = buildView(node: fb.root)
        vc.view = pageView
        window.rootViewController = UINavigationController(rootViewController: vc)
        // USER SEES THE PAGE NOW (static data visible)

        // 8. Bind interactions
        for i in 0..<fb.interactionsCount {
            let interaction = fb.interactions(at: i)!
            bindInteraction(interaction)
        }

        // 9. Fetch live providers
        for i in 0..<fb.liveProvidersCount {
            let provider = fb.liveProviders(at: i)!
            Task { await fetchLiveProvider(provider) }
        }

        // 10. Prefetch linked pages
        let nav = fb.navigation
        for pageName in nav.prefetch {
            Task { await prefetchPage(nav.availablePages[pageName]!) }
        }
    }
}
```

### Step 3: Build UIKit Views (Tree Walk)

```swift
func buildView(node: ViewNode) -> UIView {
    if node.isRegion {
        let region = node.asRegion

        // Long list → UICollectionView with cell recycling
        if region.childrenCount > 100 {
            return buildCollectionView(region: region)
        }

        // Normal region → UIStackView
        let stack = UIStackView()
        stack.axis = region.direction == .horizontal ? .horizontal : .vertical
        stack.spacing = CGFloat(region.gap)

        // Apply styles from the region
        if let bg = region.style?.background {
            stack.backgroundColor = UIColor(hex: bg)
        }
        if let padding = region.style?.padding {
            stack.layoutMargins = UIEdgeInsets(all: CGFloat(padding))
            stack.isLayoutMarginsRelativeArrangement = true
        }

        // Recurse into children
        for i in 0..<region.childrenCount {
            let child = region.children(at: i)!
            let childView = buildView(node: child)
            stack.addArrangedSubview(childView)
        }

        return stack
    } else {
        let comp = node.asComponent

        // Look up constructor by type ID — array index, O(1)
        let constructor = constructors[Int(comp.typeId)]
        let view = constructor(comp.props)

        // Store reference for later updates
        viewMap[comp.id] = view

        // Track live bindings
        for i in 0..<comp.liveBindingsCount {
            let binding = comp.liveBindings(at: i)!
            providerBindings[binding.provider, default: []].append(
                LiveBinding(viewId: comp.id, prop: binding.prop,
                            expression: binding.expression, field: binding.field)
            )
        }

        return view
    }
}
```

What the walk produces for the shop page:

```
UINavigationController (title: "Shop")
└── UIScrollView
    └── UIStackView (root, vertical)
        ├── UIStackView (horizontal, bg=#161b22, padding=16)     ← header
        │   ├── UILabel ("Shop", bold 32pt)                       ← heading
        │   └── UIButton ("Cart (...)")                           ← button (live)
        │
        └── UIStackView (vertical, gap=12)                        ← products
            ├── UIView (card, rounded, bordered)                  ← card
            │   ├── UIImageView (headphones.jpg)
            │   ├── UILabel ("Headphones", bold)
            │   └── UILabel ("₹4999", green)
            ├── UIView (card)                                     ← card
            │   ├── UIImageView (keyboard.jpg)
            │   ├── UILabel ("Keyboard", bold)
            │   └── UILabel ("₹2499", green)
            └── UIView (card)                                     ← card
                ├── UIImageView (mouse.jpg)
                ├── UILabel ("Mouse", bold)
                └── UILabel ("₹999", green)
```

### Step 4: Bind Interactions

```swift
func bindInteraction(_ interaction: Interaction) {
    guard let view = viewMap[interaction.viewId] else { return }

    let tap = UITapGestureRecognizer(target: self, action: #selector(handleTap(_:)))
    view.isUserInteractionEnabled = true
    view.addGestureRecognizer(tap)

    interactionMap[interaction.viewId] = interaction
}

@objc func handleTap(_ gesture: UITapGestureRecognizer) {
    let viewId = gesture.view!.accessibilityIdentifier!
    guard let interaction = interactionMap[viewId] else { return }

    if let builtInAction = interaction.do {
        // Built-in action — no server call
        switch builtInAction {
        case "show":
            viewMap[interaction.target]?.isHidden = false
        case "hide":
            viewMap[interaction.target]?.isHidden = true
        case "toggle":
            let target = viewMap[interaction.target]!
            target.isHidden = !target.isHidden
        case "navigate":
            Task { await navigate(to: interaction.to) }
        case "notify":
            showToast(message: interaction.message)
        case "refresh":
            Task { await refreshProvider(interaction.target) }
        default: break
        }
    }

    if let handlerName = interaction.handler {
        // Handler call — server round-trip
        Task {
            let response = try await http.post(
                "\(apiBaseURL)/orrery/handler",
                body: [
                    "handler": handlerName,
                    "args": interaction.args,
                    "page": currentPage,
                    "session": sessionToken
                ]
            )

            // Apply diff from server
            if let diffs = response.diff {
                for diff in diffs {
                    let region = viewMap[diff.region]!
                    replaceChildren(of: region, with: diff.content)
                }
            }

            // Execute then chain
            if let thenChain = interaction.then {
                for action in thenChain {
                    await executeBuiltInAction(action)
                }
            }
        }
    }
}
```

### Step 5: Fetch Live Providers

```swift
func fetchLiveProvider(_ provider: LiveProvider) async {
    var request = URLRequest(url: URL(string: "\(apiBaseURL)\(provider.url)")!)
    request.setValue("Bearer \(userToken)", forHTTPHeaderField: "Authorization")

    let (data, _) = try await URLSession.shared.data(for: request)
    let json = try JSONSerialization.jsonObject(with: data) as! [String: Any]
    // json = { "count": 3, "items": [...] }

    // Store provider state
    providerState[provider.name] = json

    // Fill in all views waiting for this provider
    guard let bindings = providerBindings[provider.name] else { return }
    for binding in bindings {
        guard let view = viewMap[binding.viewId] else { continue }
        let value = json[binding.field]!
        let resolved = binding.expression
            .replacingOccurrences(of: "$\(provider.name).\(binding.field)", with: "\(value)")

        if let button = view as? UIButton {
            button.setTitle(resolved, for: .normal)
        } else if let label = view as? UILabel {
            label.text = resolved
        }
    }
}
```

### Step 6: Navigation

```swift
func navigate(to pageName: String) async {
    guard let pageURL = navigation.availablePages[pageName] else { return }

    let pageData = try await fetch("\(cdnURL)\(pageURL)")
    let fb = ViewTree.getRootAsViewTree(bb: ByteBuffer(data: pageData))

    buildConstructorArray(fb.typeMap)

    let pageView = buildView(node: fb.root)
    let vc = UIViewController()
    vc.title = fb.pageMeta.title
    vc.view = pageView

    navigationController.pushViewController(vc, animated: true)
    navigationStack.append(pageName)

    // Bind interactions, fetch live providers for the new page
    for i in 0..<fb.interactionsCount {
        bindInteraction(fb.interactions(at: i)!)
    }
    for i in 0..<fb.liveProvidersCount {
        Task { await fetchLiveProvider(fb.liveProviders(at: i)!) }
    }
}
```

### iOS Timeline

```
0ms       App launches, adapter init
30ms      manifest.json fetched from CDN
60ms      FlatBuffer fetched from CDN (~1-3KB)
60.001ms  Zero-copy decode (no parsing)
63ms      Type map read, constructor array built
65ms      View tree walked, UIKit hierarchy built
65ms      USER SEES PRODUCTS (pre-baked data)
67ms      Interactions bound (tap gestures attached)
67ms      Live provider fetch starts: GET /api/cart
200ms     Cart API responds → "Cart (3)" filled in
200ms     Prefetch starts for cart page

Total to first paint: ~65ms
Total with live data: ~200ms
```

---

## Android Flow (@orrery/android)

### Step 1: Activity Setup

```kotlin
// Developer's MainActivity — their only Orrery code
import com.orrery.android.OrreryAdapter

class MainActivity : ComponentActivity() {
    private val adapter = OrreryAdapter(
        cdnURL = "https://cdn.myapp.com",
        apiBaseURL = "https://api.myapp.com"
    )

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            OrreryApp(adapter = adapter, startPage = "shop")
        }
    }
}
```

### Step 2: Adapter Fetches and Decodes

```kotlin
@Composable
fun OrreryApp(adapter: OrreryAdapter, startPage: String) {
    val pageState by adapter.pageState.collectAsState()
    val liveData by adapter.liveData.collectAsState(initial = emptyMap())

    LaunchedEffect(startPage) {
        // 1. Fetch manifest from CDN
        val manifestData = adapter.fetch("${adapter.cdnURL}/manifest.json")
        adapter.manifest = Json.decodeFromString<RouteManifest>(String(manifestData))

        // 2. Fetch FlatBuffer from CDN
        val pageURL = adapter.manifest!!.pages[startPage]!!
        val pageData = adapter.fetch("${adapter.cdnURL}$pageURL")

        // 3. Zero-copy decode
        val fb = ViewTree.getRootAsViewTree(ByteBuffer.wrap(pageData))

        // 4. Read type map → build factory array
        adapter.buildFactoryArray(fb.typeMap)

        // 5. Read styles → set theme
        adapter.applyTheme(fb.styles)

        // 6. Store page state → triggers Compose recomposition
        adapter.setPageState(fb)

        // 7. Bind interactions
        adapter.bindInteractions(fb)

        // 8. Fetch live providers
        adapter.fetchLiveProviders(fb)

        // 9. Prefetch linked pages
        adapter.prefetchPages(fb.navigation)
    }

    // Render when page state is available
    pageState?.let { fb ->
        Scaffold(
            topBar = {
                TopAppBar(title = { Text(fb.pageMeta.title) })
            }
        ) { padding ->
            Column(modifier = Modifier.padding(padding)) {
                RenderNode(node = fb.root, liveData = liveData, adapter = adapter)
            }
        }
    }
}
```

### Step 3: Build Compose Tree (Tree Walk)

```kotlin
@Composable
fun RenderNode(node: ViewNode, liveData: Map<String, Any>, adapter: OrreryAdapter) {
    if (node.isRegion) {
        val region = node.asRegion

        // Long list → LazyColumn with cell recycling
        if (region.childrenCount > 100) {
            RenderLazyList(region = region, liveData = liveData, adapter = adapter)
            return
        }

        // Normal region
        val modifier = Modifier
            .fillMaxWidth()
            .then(
                if (region.style?.background != null)
                    Modifier.background(Color(parseColor(region.style!!.background)))
                else Modifier
            )
            .then(
                if (region.style?.padding != null)
                    Modifier.padding(region.style!!.padding.dp)
                else Modifier
            )

        when (region.direction) {
            Direction.HORIZONTAL -> Row(
                modifier = modifier,
                horizontalArrangement = Arrangement.spacedBy(region.gap.dp)
            ) {
                for (i in 0 until region.childrenCount) {
                    RenderNode(node = region.children(i)!!, liveData = liveData, adapter = adapter)
                }
            }
            Direction.VERTICAL -> Column(
                modifier = modifier,
                verticalArrangement = Arrangement.spacedBy(region.gap.dp)
            ) {
                for (i in 0 until region.childrenCount) {
                    RenderNode(node = region.children(i)!!, liveData = liveData, adapter = adapter)
                }
            }
        }
    } else {
        val comp = node.asComponent

        // Resolve live bindings if any
        val props = if (comp.liveBindingsCount > 0) {
            comp.props.resolveWith(liveData)
        } else {
            comp.props
        }

        // Look up factory by type ID — array index, O(1)
        val factory = adapter.factories[comp.typeId]

        // Wrap with click handler if this component has an interaction
        val interaction = adapter.interactionMap[comp.id]
        if (interaction != null) {
            Box(modifier = Modifier.clickable {
                adapter.handleInteraction(interaction)
            }) {
                factory(props)
            }
        } else {
            factory(props)
        }
    }
}
```

What Compose emits for the shop page:

```
Scaffold (topBar: "Shop")
└── Column (root)
    ├── Row (bg=#161b22, padding=16dp)                        ← header
    │   ├── Text ("Shop", headlineLarge)                       ← heading
    │   └── Button ("Cart (...)")                              ← button (live)
    │
    └── Column (gap=12dp)                                      ← products
        ├── Card (rounded 12dp, bordered)                      ← card
        │   ├── AsyncImage (headphones.jpg)
        │   ├── Text ("Headphones", titleMedium)
        │   └── Text ("₹4999", green)
        ├── Card                                               ← card
        │   ├── AsyncImage (keyboard.jpg)
        │   ├── Text ("Keyboard", titleMedium)
        │   └── Text ("₹2499", green)
        └── Card                                               ← card
            ├── AsyncImage (mouse.jpg)
            ├── Text ("Mouse", titleMedium)
            └── Text ("₹999", green)
```

### Step 4: Handle Interactions

```kotlin
fun handleInteraction(interaction: Interaction) {
    // Built-in action
    interaction.doAction?.let { action ->
        when (action) {
            "show" -> _viewState.update { it.show(interaction.target) }
            "hide" -> _viewState.update { it.hide(interaction.target) }
            "toggle" -> _viewState.update { it.toggle(interaction.target) }
            "navigate" -> scope.launch { navigate(interaction.to) }
            "notify" -> showSnackbar(interaction.message)
            "refresh" -> scope.launch { refreshProvider(interaction.target) }
        }
    }

    // Handler call
    interaction.handler?.let { handlerName ->
        scope.launch {
            val response = httpClient.post("${apiBaseURL}/orrery/handler") {
                contentType(ContentType.Application.Json)
                setBody(mapOf(
                    "handler" to handlerName,
                    "args" to interaction.args,
                    "page" to currentPage,
                    "session" to sessionToken
                ))
            }.body<HandlerResponse>()

            // Apply diff — update Compose state, triggers recomposition
            response.diff?.forEach { diff ->
                _viewState.update { it.applyDiff(diff) }
            }

            // Execute then chain
            interaction.then?.forEach { action ->
                handleInteraction(action)
            }
        }
    }
}
```

### Step 5: Live Providers

```kotlin
suspend fun fetchLiveProviders(fb: ViewTree) {
    for (i in 0 until fb.liveProvidersCount) {
        val provider = fb.liveProviders(i)!!
        launch {
            val data = httpClient.get("${apiBaseURL}${provider.url}") {
                header("Authorization", "Bearer $userToken")
            }.body<Map<String, Any>>()

            providerState[provider.name] = data

            // Update Compose state → Compose automatically recomposes
            // only the composables that read from liveData
            _liveData.update { current -> current + (provider.name to data) }
        }
    }
}
```

### Android Timeline

```
0ms       Activity.onCreate, Compose init
30ms      manifest.json fetched from CDN
60ms      FlatBuffer fetched from CDN (~1-3KB)
60.001ms  Zero-copy decode
63ms      Type map read, factory array built
70ms      Compose initial composition (emits composables)
73ms      First frame drawn
73ms      USER SEES PRODUCTS (pre-baked data)
75ms      Interactions bound (clickable modifiers)
75ms      Live provider fetch starts: GET /api/cart
220ms     Cart API responds → Compose recomposes cart button → "Cart (3)"
220ms     Prefetch starts for cart page

Total to first paint: ~73ms
Total with live data: ~220ms
```

---

## Data Modes

### Static (Default) — Pre-baked Into FlatBuffer/HTML

```yaml
data:
  products: productsProvider
```

Server fetches the data at pre-render time and bakes it into the output. Same for all users. Updates when server re-renders (webhook, polling, manual trigger).

```
Server pre-render time:  GET /api/products → bake into FlatBuffer
Adapter runtime:         reads from FlatBuffer → 0ms, no API call
When data changes:       server re-renders → new FlatBuffer on CDN → manifest version bumps
```

### Live — Fetched at Runtime by Adapter

```yaml
data:
  cart: cartProvider                    # mode: "live" configured in provider registration
```

Server does NOT fetch this at pre-render time. Puts a liveProvider entry in the FlatBuffer. Adapter fetches per-user at runtime.

```
Server pre-render time:  skips cart → puts { name: "cart", url: "/api/cart" } in FlatBuffer
Adapter runtime:         GET /api/cart with user's auth token → fills in live bindings
Latency:                 100-200ms (API call)
```

### Live + Polling — Auto-Refreshes

```yaml
data:
  notifications: notificationsProvider  # mode: "live", poll: 10 configured in provider
```

Same as live, but adapter re-fetches every N seconds.

```
Adapter runtime:         GET /api/notifications every 10 seconds
Use for:                 chat messages, stock prices, live scores
```

---

## Handler Calls (User Actions)

When a user interacts with an element bound to a handler:

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

All three platforms do the same thing:

```
1. User clicks/taps the element
2. Adapter reads interaction: handler="addToCart", args={product_id: "hp-001", quantity: 1}
3. Adapter POSTs to server:
   POST https://api.myapp.com/orrery/handler
   { "handler": "addToCart", "args": { "product_id": "hp-001", "quantity": 1 } }
4. Server runs the developer's handler function:
   - Writes to database
   - Returns diff (what changed) + actions (what to do next)
5. Adapter applies diff:
   - Web: replaces region innerHTML
   - iOS: replaces child UIViews in the region
   - Android: updates Compose state → recomposition
6. Adapter executes then chain:
   - refresh("cart") → re-fetches cart live provider
   - notify("Added to cart!") → shows toast/snackbar
```

### Infinite Scroll (Load More Pattern)

```yaml
interactions:
  - on: load-more.click
    handler: loadMore
    args:
      query: "$query.text"
      page: "$results.nextPage"
```

The adapter stores provider state in memory. Each handler response updates the state:

```
Search: POST { handler: "search", args: { query: "headphones" } }
  ← response: { items: [20 products], nextPage: 2, hasMore: true }
  → adapter stores: providerState["results"] = { nextPage: 2, hasMore: true }

Scroll: POST { handler: "loadMore", args: { query: "headphones", page: 2 } }
  ← response: { append: [20 more], nextPage: 3, hasMore: true }
  → adapter appends to list, updates: providerState["results"] = { nextPage: 3, hasMore: true }

Scroll: POST { handler: "loadMore", args: { query: "headphones", page: 3 } }
  ← response: { append: [20 more], nextPage: 4, hasMore: true }
  → same pattern

Last page: POST { handler: "loadMore", args: { query: "headphones", page: 143 } }
  ← response: { append: [7 items], hasMore: false }
  → adapter sees hasMore=false → stops triggering loadMore
```

The server is stateless. The adapter sends the current page number and query with every request. The server just queries the database with the given offset and limit.

---

## OTA Updates

All three platforms use the same mechanism:

```
1. YAML changes → server re-renders → new files on CDN → manifest version bumps

2. Web:      next page load fetches manifest (no-cache) → sees new version → fetches new HTML
   iOS:      app foreground → fetches manifest → sees new version → fetches new FlatBuffer
   Android:  app foreground → fetches manifest → sees new version → fetches new FlatBuffer

3. Versioned URLs prevent stale cache:
   manifest.json           → Cache-Control: no-cache (always fresh, ~500 bytes)
   /pages/shop/v5.html     → Cache-Control: immutable (cached forever)
   /pages/shop/v6.html     → Cache-Control: immutable (new URL for new content)
```

Web: no deploy needed, instant.
iOS/Android: no App Store submission needed. Adapter is a static binary — only the data on CDN changes.

---

## Performance Comparison

```
Step                          Web           iOS            Android
──────────────────────────    ───           ───            ───────
Manifest fetch (CDN)          20-40ms       20-40ms        20-40ms
Page fetch (CDN)              20-40ms       20-40ms        20-40ms
Decode                        N/A (HTML)    <0.001ms (FB)  <0.001ms (FB)
Build views                   ~0.5ms        ~3-5ms         ~5-8ms
                              (innerHTML)   (UIKit alloc)  (Compose emit)
Layout                        ~1ms          ~3-8ms         ~3-5ms
                              (browser)     (Auto Layout)  (Compose measure)
Paint                         ~1ms          ~1ms           ~2-3ms
Event binding                 ~0.5ms        ~0.5ms         ~0.5ms
Live provider fetch           100-200ms     100-200ms      100-200ms
──────────────────────────    ───           ───            ───────
First paint (static)          ~80ms         ~65ms          ~73ms
Complete with live data       ~200ms        ~200ms         ~220ms
```
