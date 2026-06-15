# Handlers & Interactions

How user actions flow through the system.

---

## Two Kinds of Actions

### Built-in Actions (client-side, no server round-trip)

These execute instantly on the client:

| Action | What it does |
|--------|-------------|
| `show` | Make a hidden region visible |
| `hide` | Hide a visible region |
| `toggle` | Toggle region visibility |
| `navigate` | Navigate to another YAML page |
| `back` | Navigate back |
| `notify` | Show a toast/notification |
| `refresh` | Re-fetch a data provider (triggers server round-trip for new data) |
| `submit` | Validate form fields, then call handler if valid |

```yaml
interactions:
  - on: cart-btn.click
    do: show
    target: cart-modal

  - on: close-cart.click
    do: hide
    target: cart-modal

  - on: logo.click
    do: navigate
    to: home.yaml
```

No server involved. The client adapter handles these natively.

### Handler Calls (server-side, requires round-trip)

For operations that need backend logic — API calls, data mutations, business rules:

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

The YAML author specifies:
- **Which handler** to call (by name)
- **What args** to pass ($references resolved by server)
- **What happens after** (then chain of built-in actions)

The handler config (URL, method, auth) is registered by the developer server-side. The YAML author never sees infrastructure details.

---

## Handler Registration

The developer registers handlers once on the server:

```rust
// Rust
let server = OrreryServer::new()
    .register_handler("addToCart", HandlerConfig {
        url: "https://api.myapp.com/cart/add",
        method: Method::POST,
    })
    .register_handler("deleteProduct", HandlerConfig {
        url: "https://api.myapp.com/products/{product_id}",
        method: Method::DELETE,
    })
    .register_handler("submitReview", HandlerConfig {
        url: "https://api.myapp.com/reviews",
        method: Method::POST,
    })
    .register_handler("searchProducts", HandlerConfig {
        url: "https://api.myapp.com/search",
        method: Method::GET,
    });
```

```typescript
// Node.js (via NAPI-RS)
const server = createOrreryServer()
  .registerHandler('addToCart', {
    url: 'https://api.myapp.com/cart/add',
    method: 'POST',
  })
  .registerHandler('searchProducts', {
    url: 'https://api.myapp.com/search',
    method: 'GET',
  })
```

### Handler config options:

| Field | Description |
|-------|-------------|
| `url` | API endpoint. Supports `{param}` placeholders from args |
| `method` | HTTP method: GET, POST, PUT, DELETE, PATCH |
| `headers` | Additional headers (auth tokens, content-type) |
| `timeout` | Request timeout in milliseconds |
| `retry` | Retry config (count, backoff) |

---

## Event Flow

```
1. User taps "Add to Cart"
         │
2. Client adapter captures click event on [data-id="add-to-cart"]
         │
3. Client sends to server:
   {
     "handler": "addToCart",
     "args": { "product_id": "abc", "quantity": 1 },
     "event": { "type": "click" },
     "data": { current provider data snapshot }
   }
         │
4. Server:
   a. Looks up handler config for "addToCart"
   b. Resolves args ($references → actual values via temple)
   c. Calls the API: POST https://api.myapp.com/cart/add
      body: { product_id: "abc", quantity: 1 }
   d. API returns success
         │
5. Server processes "then" chain:
   a. refresh("cart") → re-fetch cart provider → re-render cart regions
   b. notify("Added to cart!") → queue notification
         │
6. Server sends response to client:
   {
     "diff": [
       { "region": "cart-actions", "content": "<updated buttons>" },
       { "region": "cart-modal", "content": "<updated cart list>" }
     ],
     "actions": [
       { "do": "notify", "message": "Added to cart!" }
     ]
   }
         │
7. Client:
   a. Applies diff to DOM / UIKit / Compose
   b. Shows notification
```

---

## Handler Args

Args support all $reference types:

```yaml
- on: place-order.click
  handler: checkout
  args:
    items: "$cart.items"              # provider data (array)
    total: "$compute.total"           # computed value
    discount: "$compute.discount"     # computed value
    user_id: "$user.id"              # provider data
    currency: "INR"                   # static value
```

The server resolves all $references before calling the handler's API. The API receives fully resolved values.

---

## Form Submit

`submit` validates all fields in a form region, then calls the handler if validation passes.

```yaml
interactions:
  - on: submit-review.click
    do: submit
    target: review-form
    handler: submitReview
    args:
      product_id: "$product.id"
      title: "$form.review-title"
      text: "$form.review-text"
      rating: "$form.review-rating"
    then:
      - do: refresh
        target: reviews
      - do: notify
        message: "Review submitted!"
      - do: hide
        target: review-form
```

Flow:
1. Client validates all fields in `review-form` region
2. If any field fails validation → show error messages, stop
3. If all pass → send to server with form values in args
4. Server calls handler API
5. On success → execute then chain

---

## Then Chains

Sequential actions after a handler completes:

```yaml
then:
  - do: refresh
    target: cart              # re-fetch cart data, re-render
  - do: hide
    target: cart-modal        # close the cart modal
  - do: notify
    message: "Order placed!"  # show success toast
  - do: navigate
    to: confirmation.yaml     # go to confirmation page
```

Actions in `then` execute in order. If the handler fails, the then chain does not execute (the client can show an error notification instead).

---

## Platform Extensions

For native capabilities that can't be expressed in YAML:

### Register an extension (per platform):

```typescript
// Web
import { registerExtension } from '@orrery/web'

registerExtension('camera', async () => {
  const stream = await navigator.mediaDevices.getUserMedia({ video: true })
  const frame = await captureFrame(stream)
  return { imageData: frame }
})

registerExtension('share', (data) => {
  navigator.share({ title: data.title, url: data.url })
})

registerExtension('clipboard', async (data) => {
  await navigator.clipboard.writeText(data.text)
  return { success: true }
})
```

```swift
// iOS
OrreryAdapter.registerExtension("camera") {
    let picker = UIImagePickerController()
    // present picker, get result
    return ["imageData": imageData]
}

OrreryAdapter.registerExtension("share") { data in
    let ac = UIActivityViewController(activityItems: [data["url"]])
    // present
}
```

```kotlin
// Android
OrreryAdapter.registerExtension("camera") {
    val intent = Intent(MediaStore.ACTION_IMAGE_CAPTURE)
    // launch, get result
    mapOf("imageData" to imageData)
}
```

### Use in YAML:

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

- on: share-btn.click
  do: extension
  name: share
  args:
    title: "$product.name"
    url: "https://myapp.com/products/$product.id"
```

Extensions bridge the gap between declarative YAML and platform-specific APIs. They're the escape hatch for the 5% of interactions that need native code.

---

## Raw Handler Functions (Alternative to YAML handlers)

Temple-based YAML handlers are the default. But developers can register raw handler functions server-side for complex logic:

```rust
server.register_raw_handler("complexCheckout", |ctx| async {
    // Custom business logic in Rust
    let cart = ctx.data.get("cart");
    let user = ctx.data.get("user");

    // Apply complex discount rules
    let discount = calculate_tiered_discount(cart, user);

    // Call multiple APIs
    let order = api::create_order(cart, discount).await?;
    let payment = api::charge(order.total, user.payment_method).await?;

    if payment.success {
        api::send_confirmation_email(user.email, order.id).await?;
        ctx.refresh("cart");
        ctx.navigate("confirmation.yaml");
        ctx.notify("Order placed!");
    } else {
        ctx.notify("Payment failed. Please try again.");
    }

    Ok(())
});
```

The YAML still references it by name:
```yaml
- on: checkout.click
  handler: complexCheckout
  args:
    items: "$cart.items"
```

Raw handlers and YAML handlers coexist. Use YAML handlers for simple API calls, raw handlers for complex orchestration.
