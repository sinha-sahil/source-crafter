# Source-crafter — Philosophy

UI can be broadly divided into two major areas: static content and dynamic content.

Static content generally refers to components, forms, API callers, simple view/state transitions. Effectively, they are repetitive, just that they are driven with schema and network integration.

On the other hand, Dynamic content is composed of elements which change per user, per device and more such params. It can be more creative like a 3D view, fancy and comes with platform specific heavy implementation details.

This project mainly aims to reduce or maximise eradication of redundant effort required to build the static content by implementing a platform agnostic DSL which later can translate to your stack. It doesn't enforce coding patterns, architecture, but gives you full freedom to adapt the output to your stack.

Additionally this enables to move faster cause the code surface exposed to your coding agent is minimal to writing the spec which we propose in YAML. Without that, an AI agent writing UI in a framework has to learn the framework AND your team's conventions before it can produce anything reliable — frameworks let you build the same UI in many different ways, and every codebase ends up with its own setup. With YAML there's nothing to learn — the structure is fixed, AI writes the spec, the engine renders. That's what people are calling vibe coding.

This project mainly targets the middle ground of UI development. Applications like dashboards, admin portals, banking interfaces, and similar business workflows are a good fit, since they generally focus on functionality and flow rather than complex transitions or highly customized interactions. For such applications, consistency and maintainability matter more than unlimited customization, which makes a YAML-driven approach work well.

On the other hand, highly customized UIs such as game-like interfaces, motion-heavy marketing pages, drawing canvases, or real-time collaborative editors can technically be represented in YAML. However, the volume of YAML required for them defeats the readability and AI-friendliness benefits. For such cases, framework code is the more suitable approach.

The goal here is not to replace every UI development approach. The aim is to provide a structured, reusable, and AI-friendly way to build UIs where consistency, maintainability, and faster development are the priority.
