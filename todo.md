# TODOs (prioritized)

## [CRITICAL]

- [x] fix text layout cache correctness in `src/core/render/d2d/cache.rs`: `TextLayoutKey` currently ignores `font_family`, `font_size`, and `font_weight`, which can return wrong cached layouts for different text styles sharing the same content and bounds.
- [x] remove panic paths in window event dispatch (`src/core/app/window.rs`): replace `.unwrap()` calls in `WM_SIZE`, `WM_MOUSEMOVE`, `WM_LBUTTONDOWN`, `WM_LBUTTONUP` with explicit error handling to avoid hard crashes from runtime/renderer errors.
- [x] fix app shutdown behavior for multi-window support (`src/core/app/window.rs`): `WM_DESTROY` always calls `PostQuitMessage(0)`, which kills the whole app when any window closes; track remaining windows and quit only when last window is destroyed.
- [x] fix the padding, margin, gap problem. ~~Add a new LayoutStyle that is bound to the frameworks api and then is converted during the building phase to taffy Layout~~

## [URGENT]

- [x] fix signal system, avoid the use of map mainly in the layout.rs file.
- [ ] prevent reactive subscription leaks (`src/core/reactive/runtime.rs` + `src/core/reactive/bind.rs`): signals only support `subscribe` (no unsubscribe), so deleted nodes keep receiving updates forever; add subscriber IDs + unsubscribe on node/subtree deletion.
- [ ] avoid panics in layout engine (`src/core/layout/engine.rs`): remove `expect(...)` paths (ex: text measurement failure, missing nodes) and propagate structured `Result` errors through layout computation.
- [ ] optimize hit testing hot path (`src/core/event/hit_test.rs`): stop cloning children vectors on each recursive call (`c.clone()`), iterate by reference/reversed index to reduce per-mouse-move allocations.
- [x] fix dpi (window moved to monitor with different DPI): handle `WM_DPICHANGED`, refresh renderer DPI, recompute scale/transform, and trigger full layout + repaint with new logical size.
- [ ] support transforms end-to-end: currently transform values exist in API/commands but are not applied during draw/hit-test/layout interactions consistently.

## [IMPORTANT]

- [ ] reduce per-frame work in `Engine::frame` (`src/core/app/engine.rs`): avoid rebuilding + sorting full render command list when only a small subtree is dirty; introduce dirty-node render list updates or retained draw list invalidation by dirty flags.
- [ ] apply render properties consistently in D2D backend (`src/core/render/d2d.rs`): `RenderCommand` carries `opacity` and `transform` but draw calls ignore them; implement per-command transform/opacity push-pop to match API behavior.
- [ ] MANAGE RAM CONSUMPTION - cache: cap/evict `D2DCache` entries (text layout/format/brush) and invalidate on device/context recreation to prevent unbounded growth and stale resources.
- [ ] benchmark: add repeatable frame-time + layout + text-measure benchmarks (scene sizes, update rates, warm/cold cache) to guide optimization work.
- [ ] fix impl trait system for elements/components so public API stays ergonomic while allowing reusable, typed component composition.
- [ ] add stop event propagation in event system (capture/bubble control): allow handlers to stop bubbling for click/hover flows to avoid conflicting parent handlers.
- [ ] active / focus / hover styles: wire pseudo-state style resolution to events and invalidation so state changes update paint/layout predictably.
- [ ] bubble hover behavior: define and implement consistent hover enter/leave bubbling semantics across nested elements.
- [ ] add dcomp integration path (DirectComposition) for better composition/presentation control and future visual effects pipeline.
- [x] remove debug I/O from render loop (`src/core/app/engine.rs`): `println!("\rrendered frame")` executes each frame and can heavily degrade performance in interactive scenes.

## [LESS IMPORTANT]

- [ ] deduplicate layout style builder code (`src/core/layout.rs`): `ContainerStylePropsImpl` and `LeafStylePropsExt` duplicate many setters (`w/h/size/max_* /m`), increasing maintenance cost and inconsistency risk; extract shared helper/macro.
- [ ] reduce clone pressure during layout (`src/core/layout/engine.rs`): `node_kind.clone()` and `style.clone()` are done in `compute_child_layout`; borrow where possible or split read paths to avoid repeated cloning.
- [ ] harden type-safe node downcasts (`src/core/arena/node.rs`): replace `unreachable!("Not a div/text")` in `as_div_mut`/`as_text_mut` with safe error-returning APIs to avoid undefined behavior patterns during future element expansion.
- [ ] debug devtools: expand diagnostics for layout tree, dirty flags, event paths, and cache stats to reduce time spent debugging framework internals.
- [ ] add default styling/theme baseline (tokens + defaults for text/background/spacing) so elements are usable without verbose style setup.
- [ ] `src/core/app/window.rs` remove (or manage) the silent error in the WM_PAINT branch.

## [NICE TO HAVE]

- [ ] add borders (including per-side width/color/style and radius-aware rendering).
- [ ] add shadows (box/text where applicable), with clear perf budget and cache strategy.
- [ ] add blur effects with graceful fallback when effect pipeline/device support is missing.
- [ ] add ANIMATION primitives (timing, interpolation, invalidation hooks) integrated with reactive updates.
- [ ] add first-class component contract in core (`src/elements` + `src/core`): define a `Component` abstraction (props + child slots + reactive local state) so custom elements can be composed without directly manipulating low-level `NodeArena` patterns.

## [DX REFERENCE - STATE + ANIMATION API SKETCH]

- [ ] keep core fluent setters as default: `.bg(...) .opacity(...) .rounded(...)`.
- [ ] add state sugar on top of generic predicate/state API:
  - `.hover(|s| s.bg(...))`
  - `.active(|s| s.scale(...))`
  - `.focus_visible(|s| s.ring(...))`
  - `.disabled(|s| s.opacity(...))`
- [ ] make `when(...)` the primitive and implement sugar via it:
  - `hover(f) => when(State::Hover, f)`
  - `active(f) => when(State::Active, f)`
- [ ] define state precedence for deterministic resolution:
  - `disabled > active > hover > focus > base`
- [ ] transition ergonomics baseline:
  - `.transition(t().color(120).transform(90).ease(Ease::OutCubic))`

Minimal examples to keep in mind during implementation:

```rust
div()
  .bg(palette::PRIMARY)
  .opacity(1.0)
  .hover(|s| s.bg(palette::PRIMARY.with_alpha(0.70)))
  .active(|s| s.scale(0.98))
  .focus_visible(|s| s.ring(2.0, palette::ACCENT))
  .transition(t().color(120).transform(90).ease(Ease::OutCubic));
```

```rust
// Optional grouping for larger style blocks; fluent setters stay primary.
div()
  .style(|s| s.bg(palette::PRIMARY).rounded(12.0).px(px(16.0)).py(px(10.0)))
  .hover(|s| s.bg(palette::PRIMARY.with_alpha(0.85)));
```

```rust
// Reactive text DX target: closure sugar + explicit derived remains available.
let counter = signal(0);
text(|| format!("Counter: {}", counter.get()));
// also valid:
text(derived(move || format!("Counter: {}", counter.get())));
```

```rust
// Signal API preference: object-style as default.
let s = signal(0);
s.get();
s.set(1);
s.update(|v| *v += 1);
```

address hit testing translation problem
