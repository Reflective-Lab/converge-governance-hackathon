---
source: mixed
tags: [visualization, architecture, decisions]
---

# Visualization Alternatives: Trade-offs and When They Make Sense

## 1. Raw Three.js (Direct)

**How it would work:**
```javascript
const scene = new THREE.Scene()
const mesh = new THREE.Mesh(geometry, material)
mesh.position.set(pos[0], pos[1], pos[2])
scene.add(mesh)

// In animate loop, manually update everything
mesh.rotation.x += 0.002
mesh.position.lerp(targetPos, 0.1)
```

**When it makes sense:**
- You want maximum control over the scene graph
- You're building something with unique, custom rendering needs
- You don't mind imperative state management (it's familiar)
- Target is not React/Vue/Svelte (plain JS app, or Python backend with WebGL frontend)

**Trade-offs:**
- ✅ Full control, mature ecosystem, tons of examples
- ❌ 3x more boilerplate per feature
- ❌ You own the animation loop, update scheduling, state sync
- ❌ No automatic reactivity — you manually sync DOM → Three.js every frame
- ❌ Harder to integrate with Svelte (data flow is one-way, you pull from Svelte into Three.js)

**Example use case:** Interactive 3D CAD viewer where you need fine control over geometry generation, picking, manipulation.

---

## 2. React Three Fiber (React Alternative)

**How it would work:**
```jsx
<Canvas>
  <mesh position={agentPos}>
    <icosahedronGeometry args={[0.4, 4]} />
    <meshPhongMaterial color={color} />
  </mesh>
</Canvas>
```

Looks similar to Threlte, but with React hooks and lifecycle.

**When it makes sense:**
- Your entire stack is React (Next.js frontend, React component library)
- You need fine control over rendering via React suspense/error boundaries
- You want React ecosystem (hooks, context, state managers like Zustand/Jotai)

**Trade-offs:**
- ✅ Declarative like Threlte, but with React's maturity
- ✅ Hooks for animation (useFrame) are familiar to React developers
- ❌ React's re-render model + Three.js can conflict (you need useFrame to avoid re-renders every state change)
- ❌ Adds React + Fiber overhead (100KB+ vs 20KB for Threlte)
- ❌ Harder to type-check (React is less strict about props than Svelte)
- ❌ Incompatible with our stack — we chose Svelte, React brings impedance mismatch

**What you'd lose:**
- Svelte's built-in reactivity (React hooks are more verbose)
- Type safety (Svelte's type inference is stronger)
- Bundle size (React + Three Fiber + Tauri = 150MB+)

---

## 3. Bevy (Rust Game Engine)

**How it would work:**
```rust
// Cargo.toml
bevy = "0.13"

// main.rs
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, animate_agents)
        .run()
}

fn animate_agents(mut query: Query<&mut Transform, With<AgentNode>>) {
    for mut transform in query.iter_mut() {
        transform.rotation.x += 0.002
    }
}
```

**When it makes sense:**
- You want everything in Rust (no JS, no Svelte, pure Rust from backend to UI)
- You need physics simulation (realistic movement, collision detection)
- You need advanced shaders or particle systems (Bevy has these built-in)
- Performance is critical (real-time simulation of 1000+ agents)
- You're building a game-like experience (interactive manipulation, full 3D control)

**Trade-offs:**
- ✅ Everything is Rust — one language, one ecosystem, maximum type safety
- ✅ No JS runtime — Tauri ships the app as pure native binary
- ✅ Physics, particles, shaders built-in (3-year head start over hand-rolled)
- ✅ ECS architecture scales to complex scenes
- ❌ Lose Svelte reactivity — Bevy is event-driven, not reactive
- ❌ Lose rapid iteration — Compile times, less flexible UI
- ❌ Different mental model — entities/components/systems vs declarative rendering
- ❌ Harder to build reactive UX — control panels, data binding less ergonomic
- ❌ Learning curve — Bevy's ECS paradigm is unfamiliar to most web devs

**Example use case:** A physics-based agent simulator where agents have realistic movement, collision, and world interaction. Or a real-time multiplayer 3D experience.

**For our case:** Overkill. Agent flows are static positions + animated particles, not physics simulation. Bevy's complexity doesn't pay off.

---

## 4. Canvas 2D + D3.js (Go Backwards)

**How it would work:**
```javascript
const canvas = document.querySelector('canvas')
const ctx = canvas.getContext('2d')

// Draw agents as circles
agents.forEach(agent => {
  ctx.fillStyle = agentColor(agent.role)
  ctx.beginPath()
  ctx.arc(agent.x, agent.y, 20, 0, 2 * Math.PI)
  ctx.fill()
})

// Draw connections as lines
flows.forEach(flow => {
  ctx.strokeStyle = '#06b6d4'
  ctx.beginPath()
  ctx.moveTo(fromX, fromY)
  ctx.lineTo(toX, toY)
  ctx.stroke()
})
```

Or use D3.js for a force-directed graph layout.

**When it makes sense:**
- Your visualization is inherently 2D (org charts, flow diagrams, networks)
- You need compatibility with older browsers (Internet Explorer, etc.)
- Performance is critical and 3D overhead isn't justified
- You're building a static or slowly-updating visualization (not real-time animation)

**Trade-offs:**
- ✅ Simpler — no 3D math, smaller bundle
- ✅ D3 ecosystem is battle-tested for network/hierarchy visualization
- ✅ Canvas performance is very good for 2D
- ❌ Can't show Z-axis (time progression gets hidden or requires animation tricks)
- ❌ Spaghetti problem — agents + connections in 2D become visually tangled
- ❌ Less immersive — you lose the spatial intuition that 3D provides
- ❌ No depth — hard to see multiple overlapping data flows

**Example use case:** An org chart (hierarchical), a dependency graph (Converge → Wolfgang → components), a DAG visualization (task pipeline).

**For our case:** Doesn't work. We need 3D to show time as Z-axis. In 2D, you'd need animation or color gradients to show iteration progression, which is less intuitive.

---

## 5. Babylon.js (Three.js Alternative)

**How it would work:**
```javascript
const engine = new BABYLON.Engine(canvas)
const scene = new BABYLON.Scene(engine)
const mesh = BABYLON.MeshBuilder.CreateSphere("agent", {diameter: 1}, scene)
```

**When it makes sense:**
- You prefer Babylon's documentation or Babylon Playground
- You need WebGPU support (Babylon has it, Three.js is catching up)
- You're in a Microsoft ecosystem (Azure, Copilot, etc.)
- You want glTF loading (Babylon has better asset pipeline)

**Trade-offs:**
- ✅ Similar capabilities to Three.js
- ✅ Better playground for testing
- ✅ WebGPU support more mature
- ❌ Smaller ecosystem than Three.js
- ❌ Still requires Threlte equivalent for Svelte integration (doesn't exist)
- ❌ No Svelte wrapper — you'd use raw Babylon or build your own

**For our case:** Not worth it. We'd lose Threlte's Svelte integration, and gain what? Babylon is mature but not better enough to justify switching ecosystems.

---

## 6. Custom WebGL + Shaders

**How it would work:**
```glsl
// vertex shader
uniform mat4 projection;
uniform vec3 agentPos;

void main() {
  gl_Position = projection * vec4(agentPos, 1.0);
}
```

**When it makes sense:**
- You're building a graphics library itself
- You need custom shaders that existing engines won't support
- Performance is critical and you can hand-optimize
- You're a graphics PhD and want to own every detail

**Trade-offs:**
- ✅ Maximum performance and control
- ❌ 4-6 month implementation for something production-quality
- ❌ You own all bugs, all platform issues
- ❌ Not suitable for business projects with deadlines

**For our case:** Absolutely not. Over-engineered. Threlte's layer over Three.js is the right abstraction level.

---

## 7. Unreal Engine / Unity (Overkill)

**How it would work:**
Export agent flow data as JSON → Load in UE/Unity → View in engine → Export back to web.

**When it makes sense:**
- You're building a AAA game or photorealistic experience
- You need Hollywood-level graphics (volumetric lighting, ray-tracing, etc.)
- Offline rendering is okay (no real-time interaction required)
- You have a game dev team already

**Trade-offs:**
- ✅ Incredible visual quality
- ✅ Battle-tested for complex scenes
- ❌ Overkill — agent flows don't need photorealism
- ❌ Disconnected pipeline — data export/import, no live updates
- ❌ Huge bundle (1GB+)
- ❌ Can't embed in Tauri (you'd launch a separate game)
- ❌ Not a UI tool — it's a game engine, not designed for data apps

**For our case:** Completely wrong tool. Like using a space shuttle to drive to the grocery store.

---

## Decision Matrix

| Tool | Our Stack Fit | Reactivity | Bundle | Learning Curve | Best For |
|---|---|---|---|---|---|
| **Threlte** ✅ | Perfect | Native Svelte | 20KB | Low | Our project |
| Raw Three.js | Good | Manual | 30KB | Medium | Custom graphics |
| React Three Fiber | Bad | React hooks | 100KB+ | Medium | React-first projects |
| Bevy | Excellent | Event-driven | 5MB | High | Physics + game-like |
| Canvas 2D | Poor | Manual | 5KB | Low | 2D hierarchies |
| Babylon.js | Good | Manual | 40KB | Medium | Microsoft stack |
| Custom WebGL | Excellent | Manual | 10KB | Extreme | Graphics libraries |
| Unreal/Unity | Bad | N/A | 1GB+ | Extreme | AAA games |

---

## Why We Made The Right Call

Threlte was chosen because:

1. **Stack coherence:** Svelte → Threlte is one mental model
2. **Rapid iteration:** Reactive data binding, no boilerplate
3. **Type safety:** Rust structs → JSON → TypeScript mirrors
4. **Performance:** Lightweight, no runtime bloat
5. **Tauri compatibility:** Scales down bundle size
6. **Maintainability:** Less code, clearer intent

**If we needed something else:**
- More control over rendering: Raw Three.js
- React-first org: React Three Fiber
- Physics + simulation: Bevy
- Simple 2D graph: D3.js + Canvas
- Photorealism: Unreal/Unity (separate app)

The choice isn't "Threlte is best for everything" — it's "Threlte is best for this specific task in this specific stack."

See also: [[Threlte Visualization]], [[Bevy Deep Dive]]
