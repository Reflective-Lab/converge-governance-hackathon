---
tags: [development]
---
# Streaming Callbacks

Implement `StreamingCallback` to watch convergence in real time. This is how you build a live dashboard in the Tauri desktop app.

```rust
struct UiCallback;

impl StreamingCallback for UiCallback {
    fn on_cycle_start(&self, cycle: u32) {
        println!("--- Cycle {cycle} ---");
    }

    fn on_fact(&self, cycle: u32, fact: &Fact) {
        println!("  [cycle {cycle}] new fact: {}", fact.id);
    }

    fn on_cycle_end(&self, cycle: u32, facts_added: usize) {
        println!("  cycle {cycle} complete: {facts_added} facts added");
    }
}
```

The callback fires *as the engine runs*, not after. Wire it into the Tauri command layer to push updates to the Svelte frontend.

See also: [[Architecture/Convergence Loop]], [[Converge/Building Blocks]]
