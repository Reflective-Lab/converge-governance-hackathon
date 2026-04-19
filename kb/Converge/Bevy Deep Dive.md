---
source: mixed
tags: [bevy, rust, visualization, architecture]
---

# Bevy Deep Dive: Architecture, Trade-offs, and Real Use Cases

## What Is Bevy?

Bevy is a Rust-native game engine and app framework built on an Entity-Component-System (ECS) architecture. It's not a graphics library like Three.js — it's a full application framework with rendering, physics, audio, asset loading, and event handling.

Think of it as: "If you wanted to write a game or 3D app entirely in Rust without touching JavaScript."

---

## ECS: The Core Paradigm

### Traditional OOP (what Three.js uses)

```rust
struct Agent {
    position: [f32; 3],
    velocity: [f32; 3],
    mesh_id: String,
}

impl Agent {
    fn update(&mut self) {
        self.position[0] += self.velocity[0];
    }
}
```

**Problem:** State is scattered. Objects own their data, methods modify it. Hard to parallelize, hard to reason about.

### ECS Approach

```rust
// Separate data from behavior
#[derive(Component)]
struct Position(Vec3);

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct AgentRole(String);

// Query: "Give me all entities with Position and Velocity"
fn move_agents(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.0 += vel.0
    }
}
```

**Benefits:**
- Data is separated from logic
- Easy to parallelize (Bevy runs this across CPU cores automatically)
- Easy to compose behavior (systems are orthogonal)
- Easy to reason about (no hidden state mutations)
- Easy to iterate (add a new component, write a system, done)

---

## Bevy Architecture

```
┌─────────────────────────────────────────────┐
│          Bevy Application                   │
├─────────────────────────────────────────────┤
│                                             │
│  World (Entity storage)                     │
│  ├─ Entity 1: Position, Velocity, Mesh     │
│  ├─ Entity 2: Position, Rotation, Mesh     │
│  └─ Entity 3: Position, Transform          │
│                                             │
│  Systems (Logic)                           │
│  ├─ move_system(Position, Velocity)        │
│  ├─ rotate_system(Rotation)                │
│  ├─ render_system(Mesh, Transform)         │
│  └─ user_input_system(Input)               │
│                                             │
│  Schedule (Execution order)                │
│  ├─ Input stage                            │
│  ├─ Update stage                           │
│  ├─ Render stage                           │
│  └─ Post-render stage                      │
└─────────────────────────────────────────────┘
```

Every frame:
1. Input stage: Collect keyboard/mouse events
2. Update stage: Run all systems that read/write components
3. Render stage: Draw the scene
4. Loop back

---

## Real Bevy Code Example: Agent Visualization

```rust
use bevy::prelude::*;

// Data
#[derive(Component)]
struct Agent {
    role: String,
}

#[derive(Component)]
struct Converging {
    target_pos: Vec3,
    speed: f32,
}

#[derive(Component)]
struct DataFlow {
    from: Entity,
    to: Entity,
    progress: f32,
}

// Startup: Create agents
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create Sonnet agent
    commands.spawn((
        Mesh3d(meshes.add(
            Mesh::try_from(shape::Icosphere { radius: 0.4, subdivisions: 4 }).unwrap()
        )),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.5, 1.0), // Blue
            emissive: LinearRgba::rgb(0.2, 0.5, 1.0),
            ..default()
        })),
        Transform::from_xyz(-2.0, 1.5, 0.0),
        Agent { role: "Analysis".into() },
        Converging {
            target_pos: Vec3::ZERO,
            speed: 0.5,
        },
    ));

    // Create KB Writer agent
    commands.spawn((
        Mesh3d(meshes.add(
            Mesh::try_from(shape::Icosphere { radius: 0.4, subdivisions: 4 }).unwrap()
        )),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.7, 0.5), // Emerald
            ..default()
        })),
        Transform::from_xyz(0.0, -1.5, 0.0),
        Agent { role: "Knowledge".into() },
        Converging {
            target_pos: Vec3::ZERO,
            speed: 0.5,
        },
    ));

    // Add camera
    commands.spawn(Camera3d::default());

    // Add lights
    commands.spawn(DirectionalLight {
        illuminance: 10000.0,
        ..default()
    });
}

// System: Move agents toward convergence point
fn converge_agents(
    mut query: Query<(&mut Transform, &Converging)>,
    time: Res<Time>,
) {
    for (mut transform, converging) in query.iter_mut() {
        let direction = (converging.target_pos - transform.translation).normalize();
        transform.translation += direction * converging.speed * time.delta_secs();
    }
}

// System: Animate data flow particles
fn animate_flows(
    mut query: Query<(&DataFlow, &mut Transform)>,
    agents: Query<&Transform, With<Agent>>,
    time: Res<Time>,
) {
    for (flow, mut transform) in query.iter_mut() {
        let from_pos = agents.get(flow.from).map(|t| t.translation).unwrap_or_default();
        let to_pos = agents.get(flow.to).map(|t| t.translation).unwrap_or_default();

        let new_progress = (flow.progress + time.delta_secs() * 0.5) % 1.0;
        transform.translation = from_pos.lerp(to_pos, new_progress);
    }
}

// System: Handle user input
fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Agent>>,
) {
    if keyboard.pressed(KeyCode::ArrowUp) {
        for mut transform in query.iter_mut() {
            transform.translation.y += 0.1;
        }
    }
}

// Main app
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (converge_agents, animate_flows, handle_input))
        .run()
}
```

**Key insights:**
- Setup: Spawn entities with components, assets
- Systems: Pure functions that query components and update them
- Time: Built-in Time resource for frame-rate-independent motion
- Queries: `Query<(&Transform, &Agent)>` automatically fetches matching entities
- No animation loop: Bevy manages scheduling; you just write systems

---

## Why Bevy for Agent Visualization (If We Chose It)

### Hypothetical Bevy Approach

```
┌─────────────────────────────────────┐
│  Rust Backend (Converge)            │
│  ├─ Agent state, decision logic     │
│  └─ Outputs ConvergenceLayer        │
└──────────────┬──────────────────────┘
               │ JSON over IPC
┌──────────────▼──────────────────────┐
│  Bevy App (Desktop)                 │
│  ├─ Deserialize ConvergenceLayer    │
│  ├─ Spawn entities for agents       │
│  ├─ Animate flows, convergence      │
│  ├─ Handle camera, input            │
│  └─ Render to screen                │
└─────────────────────────────────────┘
```

### Advantages

#### 1. All Rust

No JavaScript context switching. One language from backend to UI.

```rust
// Same types everywhere
let flow_data: AgentFlowVisualization = deserialize_from_converge()?;

// Build visualization from data
for layer in flow_data.layers {
    for agent in layer.agents {
        commands.spawn((
            Position(agent.start_position.into()),
            Agent { name: agent.name },
        ));
    }
}
```

#### 2. Physics-ready

If agents need realistic movement (collision, gravity, momentum):

```rust
#[derive(Component)]
struct RigidBody {
    velocity: Vec3,
    mass: f32,
}

// Bevy's physics plugin handles collisions automatically
commands
    .spawn(Agent { ... })
    .insert(RigidBody { velocity: Vec3::ZERO, mass: 1.0 });

// No code needed — physics system runs automatically
```

#### 3. Massive Performance

Bevy's ECS is heavily optimized for parallelism.

**Scenario:** You have 10,000 agents converging simultaneously.
- Threlte: Might struggle (JavaScript event loop, per-frame overhead)
- Bevy: Smooth 60fps (automatic multi-core parallelization)

```rust
// Bevy's scheduler automatically parallelizes this
fn update_1000_agents(mut query: Query<&mut Position, &Converging>) {
    query.par_iter_mut().for_each(|mut pos| {
        // Runs across multiple CPU cores simultaneously
    });
}
```

#### 4. Advanced Rendering

Built-in support for:
- Multiple render targets (screenshot, video export)
- Custom shaders (glow, outlines, post-processing)
- PBR materials (realistic lighting)
- Skeletal animation (if agents morph)

```rust
// Add a post-processing effect (bloom/glow)
let post_processing = commands.spawn_empty().id();
commands.entity(post_processing).insert(BloomSettings::default());
```

#### 5. Cross-platform

Ships the same Bevy binary to macOS, Windows, Linux, Web (via WebGPU).

---

## Disadvantages of Bevy for Agent Visualization

### 1. Learning Curve

ECS is unfamiliar to most web developers.

```rust
// This is not intuitive if you think in React/Svelte terms
#[derive(Component)]
struct Health(f32);

// "Where does the health value live?" Not in an object. In the World.
// "How do I update it?" Query for entities with that component.
```

### 2. No Reactive Binding

You can't do `let pos = $: calculatePos(time)` like in Svelte.

Instead, you write systems explicitly:
```rust
// Every frame, a system runs this
fn animate(mut query: Query<&mut Position>) {
    // You own the loop
}
```

### 3. UI is Secondary

Bevy is for games, not for data apps with side panels, buttons, forms.

If you need:
- A "Load Flow" button
- A sidebar showing agent details
- A data table with results

You'd need to build UI alongside Bevy, or use a separate UI framework (egui, etc.).

```rust
// UI code in Bevy is more verbose
commands.spawn((
    Button,
    Node {
        width: Val::Px(100.0),
        height: Val::Px(50.0),
        ..default()
    },
    BackgroundColor(Color::srgb(0.2, 0.5, 1.0)),
)).with_children(|parent| {
    parent.spawn(Text::new("Load Flow"));
});
```

### 4. Compile Times

Bevy projects take 1-2 minutes to compile (Rust + assets). Longer than Svelte hot-reload.

### 5. Fewer 3D Assets

Three.js has a massive ecosystem of pre-made models, shaders, effects. Bevy is catching up but smaller.

---

## When Bevy Actually Wins

### Scenario 1: Real-Time Physics Simulation

Agent behaviors need physics:
- Agents repel each other (collision avoidance)
- Gravity pulls agents downward as simulation progresses
- Momentum carries agents past convergence point
- Bouncing, settling into stable state

Bevy + Rapier physics engine:
```rust
#[derive(Component)]
struct AgentPhysics {
    collider: Collider,
    rigid_body: RigidBody,
}

// Bevy's physics plugin handles all collision/response
// No code needed
```

With Threlte: You'd manually calculate collisions, resolve overlaps, integrate velocities. Hours of work.

### Scenario 2: Thousands of Agents

Monterro with 1000 agents, each updating position, rotation, glow intensity every frame.

**Threlte:**
```javascript
// Loop through all 1000 agents
agents.forEach(agent => {
    agent.mesh.position.copy(agent.computedPosition)
    agent.mesh.rotation.x += 0.002
    agent.glow.material.opacity = Math.sin(time) * 0.5
})
// This is slow, runs on main thread
```

**Bevy:**
```rust
// Bevy's scheduler parallelizes across CPU cores
fn update_agents(mut query: Query<(&mut Transform, &mut GlowMaterial)>) {
    query.par_iter_mut().for_each(|(mut transform, mut glow)| {
        transform.translation = compute_position(time);
        glow.opacity = sin(time) * 0.5;
    });
}
// Runs on multiple cores simultaneously
```

### Scenario 3: Complex Event System

Agent A finishes analysis → Sends signal to Agent B
Agent B receives signal → Updates internal state
Agent B broadcasts result → Agent C and D react

Threlte: You'd manually manage event callbacks or use a state manager.

Bevy: Built-in event system:
```rust
#[derive(Event)]
struct AgentFinished(Entity);

// System: When Sonnet finishes analysis
fn sonnet_done(mut ev_finished: EventWriter<AgentFinished>) {
    ev_finished.send(AgentFinished(sonnet_entity));
}

// System: When any agent finishes, trigger dependent agents
fn on_agent_finished(
    mut events: EventReader<AgentFinished>,
    mut query: Query<&mut Status>,
) {
    for AgentFinished(entity) in events.read() {
        if let Ok(mut status) = query.get_mut(*entity) {
            status.0 = "Finished";
        }
    }
}
```

### Scenario 4: Save Visualization as Video

User clicks "Export"
→ Capture each frame at 60fps
→ Encode to MP4

Threlte: You'd use a separate library (ffmpeg, etc.) and manually capture canvas frames.

Bevy: Built-in render target capturing:
```rust
// Render to texture instead of screen
commands.spawn(Camera3d {
    target: RenderTarget::Image(image_handle.clone()),
    ..default()
});

// Every frame, the texture updates
// Save to disk: image.save("frame.png")
```

### Scenario 5: Offline Rendering

Converge computes a visualization
→ Bevy renders it at high quality
→ Saves as still image or video
→ No interactive window needed

Perfect for Bevy (headless mode):
```bash
cargo run --release --features headless

// Renders without opening a window, saves output
```

---

## Comparing Feature Completeness

| Feature | Threlte | Bevy |
|---|---|---|
| 3D rendering | ✅ (Three.js) | ✅ (wgpu) |
| Physics | ❌ (need third-party) | ✅ (Rapier built-in) |
| Audio | ❌ | ✅ (Bevy Audio) |
| Particles | ❌ (need custom) | ✅ (Bevy Particles) |
| Text rendering | ⚠️ (hard in 3D) | ✅ (TextMesh) |
| Input handling | ✅ | ✅ (more complete) |
| Asset pipeline | ❌ (manual) | ✅ (GLTF, PNG, etc.) |
| Networking | ❌ | ⚠️ (via plugins) |
| UI | ⚠️ (egui) | ✅ (Bevy UI) |
| Animation | ⚠️ (manual) | ✅ (Skeletal, keyframes) |
| Debugging | ⚠️ | ✅ (Inspector plugin) |
| Hot reload | ⚠️ (watch rebuild) | ✅ (Asset hot reload) |

---

## The Compile-Time Trade-off

**Threlte workflow:**
Edit .svelte → Save → Instant hot reload (500ms)

**Bevy workflow:**
Edit .rs → Save → Cargo rebuild (1-2 min) → Run

For rapid prototyping (like we're doing), Threlte wins.
For final product (stable feature set, optimization), Bevy wins.

---

## Hypothetical Bevy Version of Gamification

If we rewrote in Bevy:

**Gains:**
- ✅ All Rust, type-safe end-to-end
- ✅ Physics simulation (if agents collide)
- ✅ 10,000+ agent performance
- ✅ Native Windows/Mac/Linux/Web
- ✅ Built-in asset pipeline (load GLTF models, textures)
- ✅ Particle effects, post-processing
- ✅ Export as video

**Losses:**
- ❌ 1-2 minute recompile on changes
- ❌ Steeper learning curve (ECS)
- ❌ UI is more verbose (buttons, panels, forms)
- ❌ Less like "web dev" (more like game dev)
- ❌ Smaller ecosystem for 3D assets

**Verdict:** Better for a shipped game-like product with complex interactions. Worse for a data visualization tool with UI controls.

---

## When You'd Migrate from Threlte to Bevy

Red flags to watch:
1. **Performance:** "Our 1000 agents are dropping frames" → Bevy
2. **Physics:** "Agents need realistic collision" → Bevy
3. **Complexity:** "We have 50+ interconnected systems" → Bevy's ECS scales better
4. **Assets:** "We need pre-made 3D models, animations" → Bevy's asset pipeline
5. **Desktop:** "We're shipping a game-like app, not a web tool" → Bevy

**For the current project:** Stick with Threlte. We have:
- Simple static positions
- Smooth interpolation (no physics)
- 4-10 agents per layer (not thousands)
- Control panels and UI (Bevy is awkward)
- Rapid iteration needs

---

## Summary: Bevy Is...

A complete application framework for building interactive 3D apps, games, and simulations in pure Rust, with built-in physics, particles, audio, and a sophisticated scheduling system.

It's not "Three.js for Rust" — it's "Unreal Engine for indie developers."

**Use it when:**
- ✅ You need physics or complex behavior
- ✅ You're rendering 1000+ objects
- ✅ You're building a game-like experience
- ✅ All-Rust is a hard requirement
- ✅ You don't need a sophisticated UI

**Don't use it for:**
- ❌ Data visualization with UI panels
- ❌ Rapid prototyping (compile time)
- ❌ Team unfamiliar with Rust or ECS
- ❌ Web-first projects

See also: [[Threlte Visualization]], [[Visualization Alternatives]]
