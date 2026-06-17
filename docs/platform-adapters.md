# Platform Adapters

How `@orrery/ios` and `@orrery/android` are built.

---

## What The Adapter Does

The adapter is a thin, static layer with exactly 6 responsibilities:

```
┌─────────────────────────────────────────────────────┐
│                  Platform Adapter                    │
│            (@orrery/ios or @orrery/android)          │
│                                                      │
│  1. Fetch    — get pre-rendered files from CDN       │
│  2. Decode   — FlatBuffers zero-copy (prod)          │
│               or JSON decode (debug)                 │
│  3. Build    — walk tree, construct native views     │
│               via type ID → constructor array        │
│  4. Bind     — attach gesture/click handlers         │
│               from interaction definitions           │
│  5. Navigate — manifest lookup → fetch next page     │
│               from CDN → swap view tree              │
│  6. Diff     — apply patches when data changes       │
│               (swap views in changed regions only)   │
└─────────────────────────────────────────────────────┘
```

---

## Adapter vs Server Responsibilities

```
┌──────────────────────────────┐     ┌──────────────────────────────┐
│         ADAPTER              │     │         SERVER                │
│   (ships once to app store)  │     │   (runs on your backend)     │
├──────────────────────────────┤     ├──────────────────────────────┤
│ Fetch files from CDN         │     │ Parse YAML                   │
│ Decode FlatBuffers           │     │ Resolve use: references      │
│ Map type IDs to views        │     │ SSR extract components       │
│ Construct UIKit/Compose      │     │ Compile temple blobs         │
│ Attach gestures/clicks       │     │ Fetch provider data          │
│ show/hide/toggle/navigate    │     │ Evaluate compute/conditions  │
│ Apply diffs                  │     │ Build view tree              │
│ Virtual scroll / recycling   │     │ Serialize to FlatBuffers     │
│                              │     │ Pre-render + store to CDN    │
│ NO logic                     │     │ Execute handlers             │
│ NO conditions                │     │ Compute diffs                │
│ NO data fetching             │     │ Generate manifest            │
│ NO template rendering        │     │                              │
└──────────────────────────────┘     └──────────────────────────────┘
```

The adapter never needs to change for UI updates. It's a static interpreter of a data format. That's what makes OTA possible.

---

## iOS Adapter (`@orrery/ios`)

### Public API

```swift
import UIKit
import FlatBuffers

public class OrreryAdapter {
    private let cdnBase: URL
    private var manifest: RouteManifest?
    private var constructors: [(Props) -> UIView] = []
    private var currentPageView: UIView?
    private var currentInteractions: [Interaction] = []
    private var navigationStack: [String] = []
    private let container: UIView

    public init(container: UIView, cdn: String) {
        self.container = container
        self.cdnBase = URL(string: cdn)!
        registerPrimitives()
    }

    public func mount(startPage: String) async {
        let manifestData = await fetch(path: "/manifest.json")
        self.manifest = try JSONDecoder().decode(RouteManifest.self, from: manifestData)
        await navigate(to: startPage)
    }
}
```

### Fetching (CDN)

```swift
extension OrreryAdapter {
    private func fetch(path: String) async -> Data {
        let url = cdnBase.appendingPathComponent(path)
        let (data, _) = try await URLSession.shared.data(from: url)
        return data
    }
}
```

### Navigation (Manifest-Based)

```swift
extension OrreryAdapter {
    public func navigate(to pageName: String) async {
        guard let pagePath = manifest?.pages[pageName] else { return }

        // Fetch pre-rendered FlatBuffers from CDN (no server round-trip)
        let pageData = await fetch(path: pagePath)
        let interactionData = await fetch(
            path: pagePath.replacingOccurrences(of: ".fb", with: ".interactions.json")
        )

        // Decode FlatBuffers (zero-copy, < 1ms)
        let viewTree = ViewTree.getRootAsViewTree(bb: ByteBuffer(data: pageData))
        let interactions = try JSONDecoder().decode([Interaction].self, from: interactionData)

        // Build native view hierarchy
        let pageView = buildView(from: viewTree.rootRegion)

        // Swap in container
        currentPageView?.removeFromSuperview()
        container.addSubview(pageView)
        pageView.frame = container.bounds
        currentPageView = pageView

        // Bind interactions
        currentInteractions = interactions
        bindInteractions(in: pageView, interactions: interactions)

        // Track navigation
        navigationStack.append(pageName)

        // Prefetch linked pages in background
        prefetchLinkedPages(interactions: interactions)
    }
}
```

### View Construction (Core Mapper)

```swift
extension OrreryAdapter {
    private func buildView(from node: ViewNode) -> UIView {
        switch node.type {
        case .region:
            let region = node.asRegion

            // Large lists → UICollectionView with cell recycling
            if region.itemCount > 100 {
                return buildCollectionView(region: region)
            }

            let stack = UIStackView()
            stack.axis = region.direction == .horizontal ? .horizontal : .vertical
            stack.spacing = CGFloat(region.gap)

            for i in 0..<region.childCount {
                let child = region.child(at: i)
                stack.addArrangedSubview(buildView(from: child))
            }
            return stack

        case .component:
            let component = node.asComponent
            // Integer type ID → array index → constructor function
            let constructor = constructors[Int(component.typeId)]
            return constructor(component.propsDict)
        }
    }

    private func buildCollectionView(region: RegionNode) -> UIView {
        let layout = UICollectionViewFlowLayout()
        layout.scrollDirection = region.direction == .horizontal ? .horizontal : .vertical

        let cv = UICollectionView(frame: .zero, collectionViewLayout: layout)
        cv.register(OrreryCell.self, forCellWithReuseIdentifier: "cell")

        let dataSource = OrreryDataSource(region: region, constructors: constructors)
        cv.dataSource = dataSource
        return cv
    }
}
```

### Interaction Binding

```swift
extension OrreryAdapter {
    private func bindInteractions(in view: UIView, interactions: [Interaction]) {
        for interaction in interactions {
            let (id, event) = interaction.on.splitOnDot()
            guard let targetView = view.findView(withDataId: id) else { continue }

            switch event {
            case "click", "tap":
                let tap = UITapGestureRecognizer(target: self, action: #selector(handleTap(_:)))
                tap.accessibilityLabel = interaction.id
                targetView.addGestureRecognizer(tap)
                targetView.isUserInteractionEnabled = true

            case "longpress":
                let lp = UILongPressGestureRecognizer(target: self, action: #selector(handleLongPress(_:)))
                lp.accessibilityLabel = interaction.id
                targetView.addGestureRecognizer(lp)

            case "swipe":
                let swipe = UISwipeGestureRecognizer(target: self, action: #selector(handleSwipe(_:)))
                swipe.accessibilityLabel = interaction.id
                targetView.addGestureRecognizer(swipe)

            default: break
            }
        }
    }

    @objc private func handleTap(_ gesture: UITapGestureRecognizer) {
        let interactionId = gesture.accessibilityLabel!
        let interaction = currentInteractions.first { $0.id == interactionId }!
        executeAction(interaction)
    }
}
```

### Action Execution

```swift
extension OrreryAdapter {
    private func executeAction(_ interaction: Interaction) {
        switch interaction.action {
        case .show(let target):
            findRegion(target)?.isHidden = false
        case .hide(let target):
            findRegion(target)?.isHidden = true
        case .toggle(let target):
            findRegion(target)?.isHidden.toggle()
        case .navigate(let pageName):
            Task { await navigate(to: pageName) }
        case .back:
            navigationStack.removeLast()
            if let previous = navigationStack.last {
                Task { await navigate(to: previous) }
            }
        case .notify(let message):
            showToast(message)
        case .refresh(let target):
            Task { await refreshRegion(target) }
        case .handler(let name, let args, let then):
            Task { await callHandler(name: name, args: args, then: then) }
        case .extension(let name, let args):
            Task { await callExtension(name: name, args: args) }
        }
    }
}
```

### Handler Calls (Only Server Round-Trip)

```swift
extension OrreryAdapter {
    private func callHandler(name: String, args: [String: Any], then: [Action]) async {
        let payload = HandlerRequest(handler: name, args: args)
        let data = try await post(to: "/api/orrery/handler", body: payload)
        let response = try JSONDecoder().decode(HandlerResponse.self, from: data)

        for diff in response.diffs {
            applyDiff(diff)
        }
        for action in response.actions + then {
            executeAction(action)
        }
    }
}
```

### Diff Application

```swift
extension OrreryAdapter {
    private func applyDiff(_ diff: ViewDiff) {
        guard let targetRegion = findRegion(diff.region) as? UIStackView else { return }

        targetRegion.arrangedSubviews.forEach { $0.removeFromSuperview() }

        let newTree = ViewTree.getRootAsViewTree(bb: ByteBuffer(data: diff.content))
        for i in 0..<newTree.rootRegion.childCount {
            let child = newTree.rootRegion.child(at: i)
            targetRegion.addArrangedSubview(buildView(from: child))
        }

        bindInteractions(in: targetRegion, interactions: diff.interactions)
    }
}
```

### Prefetch

```swift
extension OrreryAdapter {
    private func prefetchLinkedPages(interactions: [Interaction]) {
        for interaction in interactions {
            if case .navigate(let pageName) = interaction.action,
               let path = manifest?.pages[pageName] {
                Task(priority: .background) {
                    let _ = await fetch(path: path)
                }
            }
        }
    }
}
```

### Primitive Registration

```swift
extension OrreryAdapter {
    private func registerPrimitives() {
        constructors = [
            // 0: text
            { props in
                let label = UILabel()
                label.text = props["content"] as? String
                label.numberOfLines = 0
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
                field.borderStyle = .roundedRect
                return field
            },
            // 3: image
            { props in
                let imageView = UIImageView()
                if let urlString = props["src"] as? String,
                   let url = URL(string: urlString) {
                    imageView.loadAsync(from: url)
                }
                return imageView
            },
            // 4: heading
            { props in
                let label = UILabel()
                label.text = props["text"] as? String
                label.font = .boldSystemFont(ofSize: 24)
                return label
            },
            // 5: divider
            { _ in
                let view = UIView()
                view.backgroundColor = .separator
                view.heightAnchor.constraint(equalToConstant: 1).isActive = true
                return view
            },
            // 6: icon
            { props in
                let imageView = UIImageView()
                let name = props["name"] as? String ?? ""
                imageView.image = IconRegistry.shared.icon(named: name)
                return imageView
            },
        ]
    }

    // Developer registers custom components
    public static func register(_ name: String, constructor: @escaping (Props) -> UIView) {
        // Added to constructors array at the type ID assigned by server type map
    }
}
```

---

## Android Adapter (`@orrery/android`)

### Public API

```kotlin
class OrreryAdapter(private val cdnBase: String) {
    internal var manifest: RouteManifest? = null
    internal val factories = mutableListOf<@Composable (Props) -> Unit>()
    internal val extensions = mutableMapOf<String, suspend (Map<String, Any>) -> Map<String, Any>>()
    internal val coroutineScope = CoroutineScope(Dispatchers.Main + SupervisorJob())

    init { registerPrimitives() }

    suspend fun mount(startPage: String): OrreryState {
        val manifestData = fetch("/manifest.json")
        manifest = Json.decodeFromString(RouteManifest.serializer(), String(manifestData))
        return OrreryState(adapter = this, startPage = startPage)
    }

    internal suspend fun fetch(path: String): ByteArray {
        val url = "$cdnBase$path"
        return httpClient.get(url).body()
    }
}
```

### Compose State

```kotlin
@Stable
class OrreryState(
    private val adapter: OrreryAdapter,
    startPage: String
) {
    var currentPage by mutableStateOf<PageData?>(null)
        private set
    var currentPageName by mutableStateOf(startPage)
        private set
    var isLoading by mutableStateOf(true)
        private set
    private val navigationStack = mutableListOf<String>()

    init {
        adapter.coroutineScope.launch { navigate(startPage) }
    }

    suspend fun navigate(pageName: String) {
        isLoading = true
        val pagePath = adapter.manifest?.pages?.get(pageName) ?: return
        val pageBytes = adapter.fetch(pagePath)
        val interactionBytes = adapter.fetch(pagePath.replace(".fb", ".interactions.json"))

        currentPage = PageData(
            viewTree = ViewTree.getRootAsViewTree(ByteBuffer.wrap(pageBytes)),
            interactions = Json.decodeFromString(interactionBytes.decodeToString())
        )
        currentPageName = pageName
        navigationStack.add(pageName)
        isLoading = false

        prefetchLinkedPages(currentPage!!.interactions)
    }

    fun goBack() {
        if (navigationStack.size > 1) {
            navigationStack.removeLast()
            adapter.coroutineScope.launch { navigate(navigationStack.last()) }
        }
    }

    private suspend fun prefetchLinkedPages(interactions: List<Interaction>) {
        for (interaction in interactions) {
            if (interaction.action is Action.Navigate) {
                val path = adapter.manifest?.pages?.get(interaction.to) ?: continue
                adapter.coroutineScope.launch(Dispatchers.IO) { adapter.fetch(path) }
            }
        }
    }
}
```

### Root Composable

```kotlin
@Composable
fun OrreryRoot(state: OrreryState) {
    val page = state.currentPage
    if (page == null || state.isLoading) {
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            CircularProgressIndicator()
        }
        return
    }

    RenderNode(
        node = page.viewTree.rootRegion,
        interactions = page.interactions,
        state = state
    )
}
```

### Node Renderer

```kotlin
@Composable
fun RenderNode(node: ViewNode, interactions: List<Interaction>, state: OrreryState) {
    when (node.type) {
        NodeType.REGION -> {
            val region = node.asRegion

            if (region.itemCount > 100) {
                RenderLazyList(region, interactions, state)
                return
            }

            when (region.direction) {
                Direction.HORIZONTAL -> Row(
                    horizontalArrangement = Arrangement.spacedBy(region.gap.dp),
                    modifier = Modifier.applyRegionStyle(region)
                ) {
                    for (i in 0 until region.childCount) {
                        RenderNode(region.child(i), interactions, state)
                    }
                }
                Direction.VERTICAL -> Column(
                    verticalArrangement = Arrangement.spacedBy(region.gap.dp),
                    modifier = Modifier.applyRegionStyle(region)
                ) {
                    for (i in 0 until region.childCount) {
                        RenderNode(region.child(i), interactions, state)
                    }
                }
                Direction.GRID -> LazyVerticalGrid(
                    columns = GridCells.Fixed(region.columns),
                    horizontalArrangement = Arrangement.spacedBy(region.gap.dp),
                    verticalArrangement = Arrangement.spacedBy(region.gap.dp)
                ) {
                    items(region.childCount) { i ->
                        RenderNode(region.child(i), interactions, state)
                    }
                }
            }
        }

        NodeType.COMPONENT -> {
            val component = node.asComponent
            val props = component.propsMap
            val interaction = interactions.find { it.targetId == component.dataId }

            val modifier = if (interaction != null) {
                Modifier.clickable { state.executeAction(interaction) }
            } else {
                Modifier
            }

            Box(modifier = modifier) {
                state.adapter.factories[component.typeId](props)
            }
        }
    }
}
```

### Lazy Lists

```kotlin
@Composable
fun RenderLazyList(region: RegionNode, interactions: List<Interaction>, state: OrreryState) {
    when (region.direction) {
        Direction.VERTICAL -> LazyColumn(
            verticalArrangement = Arrangement.spacedBy(region.gap.dp)
        ) {
            items(
                count = region.childCount,
                key = { region.child(it).key }
            ) { i ->
                RenderNode(region.child(i), interactions, state)
            }
        }
        Direction.HORIZONTAL -> LazyRow(
            horizontalArrangement = Arrangement.spacedBy(region.gap.dp)
        ) {
            items(
                count = region.childCount,
                key = { region.child(it).key }
            ) { i ->
                RenderNode(region.child(i), interactions, state)
            }
        }
    }
}
```

### Action Execution

```kotlin
fun OrreryState.executeAction(interaction: Interaction) {
    val adapter = this.adapter
    when (interaction.action) {
        is Action.Show -> findRegion(interaction.target)?.visible = true
        is Action.Hide -> findRegion(interaction.target)?.visible = false
        is Action.Toggle -> findRegion(interaction.target)?.let { it.visible = !it.visible }
        is Action.Navigate -> adapter.coroutineScope.launch { navigate(interaction.to) }
        is Action.Back -> goBack()
        is Action.Notify -> showSnackbar(interaction.message)
        is Action.Refresh -> adapter.coroutineScope.launch { refreshRegion(interaction.target) }
        is Action.Handler -> adapter.coroutineScope.launch {
            callHandler(interaction.handler, interaction.args, interaction.then)
        }
        is Action.Extension -> adapter.coroutineScope.launch {
            callExtension(interaction.name, interaction.args)
        }
    }
}
```

### Handler Call

```kotlin
suspend fun OrreryState.callHandler(name: String, args: Map<String, Any>, then: List<Action>) {
    val payload = HandlerRequest(handler = name, args = args)
    val response = adapter.post("/api/orrery/handler", payload)
    val result = Json.decodeFromString<HandlerResponse>(response)

    for (diff in result.diffs) { applyDiff(diff) }
    for (action in result.actions + then) { executeAction(action) }
}
```

### Primitive Registration

```kotlin
fun OrreryAdapter.registerPrimitives() {
    factories.addAll(listOf(
        // 0: text
        { props -> Text(text = props["content"] as String) },
        // 1: button
        { props -> Button(onClick = {}) { Text(props["text"] as String) } },
        // 2: input
        { props ->
            var value by remember { mutableStateOf("") }
            TextField(
                value = value,
                onValueChange = { value = it },
                placeholder = { Text(props["placeholder"] as? String ?: "") }
            )
        },
        // 3: image
        { props -> AsyncImage(model = props["src"]) },
        // 4: heading
        { props ->
            Text(
                text = props["text"] as String,
                style = MaterialTheme.typography.headlineMedium
            )
        },
        // 5: divider
        { _ -> HorizontalDivider() },
        // 6: icon
        { props ->
            Icon(
                imageVector = IconRegistry.get(props["name"] as String),
                contentDescription = props["name"] as String
            )
        },
    ))
}

// Developer registers custom components
fun OrreryAdapter.register(name: String, factory: @Composable (Props) -> Unit) {
    factories.add(factory)
}
```

---

## How The Adapter Stays Thin

| Responsibility | iOS (lines) | Android (lines) |
|---------------|-------------|-----------------|
| Fetch (CDN + manifest) | ~100 | ~80 |
| Decode (FlatBuffers) | ~50 | ~50 |
| Build views (tree walk + constructors) | ~200 | ~150 |
| Bind interactions (gestures/clicks) | ~150 | ~100 |
| Execute actions (show/hide/navigate/notify) | ~100 | ~80 |
| Diff application (partial update) | ~100 | ~80 |
| **Adapter core** | **~700** | **~540** |
| Primitives (text, button, input, image, etc.) | ~200 | ~150 |
| UICollectionView / LazyColumn setup | ~150 | ~100 |
| **Total** | **~1050** | **~790** |

Everything else — layout logic, data fetching, conditions, compute, template rendering — lives on the server. The adapter is dumb by design.

---

## Integration In Consumer App

### iOS (UIKit)

```swift
// AppDelegate or SceneDelegate
class SceneDelegate: UIResponder, UIWindowSceneDelegate {
    var window: UIWindow?
    var orrery: OrreryAdapter?

    func scene(_ scene: UIScene, willConnectTo session: UISceneSession, options: UIScene.ConnectionOptions) {
        guard let windowScene = scene as? UIWindowScene else { return }

        let window = UIWindow(windowScene: windowScene)
        let rootVC = UIViewController()
        window.rootViewController = rootVC
        window.makeKeyAndVisible()
        self.window = window

        // Mount Orrery
        orrery = OrreryAdapter(container: rootVC.view, cdn: "https://cdn.myapp.com")

        // Register custom components
        OrreryAdapter.register("product-card") { props in
            // Custom UIView for product card
        }

        Task { await orrery?.mount(startPage: "home") }
    }
}
```

### Android (Compose)

```kotlin
// MainActivity
class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val adapter = OrreryAdapter(cdnBase = "https://cdn.myapp.com")

        // Register custom components
        adapter.register("product-card") { props ->
            ProductCard(name = props["name"] as String, price = props["price"] as String)
        }

        setContent {
            var state by remember { mutableStateOf<OrreryState?>(null) }

            LaunchedEffect(Unit) {
                state = adapter.mount(startPage = "home")
            }

            state?.let { OrreryRoot(it) }
        }
    }
}
```

### Hybrid (Partial Orrery)

The adapter can also be used for just part of the screen. The rest is native:

```swift
// iOS — Orrery for one section, native for the rest
let nativeHeader = MyCustomHeaderView()
let orreryContainer = UIView()
let nativeFooter = MyCustomTabBar()

// Only the content area is driven by Orrery
orrery = OrreryAdapter(container: orreryContainer, cdn: "https://cdn.myapp.com")
Task { await orrery?.mount(startPage: "home") }
```

```kotlin
// Android — Orrery for content, native for shell
setContent {
    Scaffold(
        topBar = { MyNativeTopBar() },
        bottomBar = { MyNativeBottomBar() }
    ) { padding ->
        Box(modifier = Modifier.padding(padding)) {
            state?.let { OrreryRoot(it) }
        }
    }
}
```
