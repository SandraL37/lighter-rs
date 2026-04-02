# TODOs (prioritized)

## [CANONICAL A++ ROADMAP - INTERACTION STATES + TRANSITIONS]

### Phase 0 - Baseline and Safety

- [ ] P0.1 create a dedicated branch for the state/interaction refactor.
- [ ] P0.2 define baseline behavior snapshots for hover, click, and resize sample apps (video/gif + expected logs).
- [ ] P0.3 add a short architecture note documenting current runtime data ownership across `NodeData`, `NodeLayout`, and renderer commands.
- [ ] P0.4 add compile-time guards/tests for no panic paths in interaction code.
- [ ] P0.5 add a minimal benchmark scene for many hoverable nodes to track regressions during refactor.

### Phase 1 - Runtime Data Model Hardening

- [ ] P1.1 keep runtime split: `NodeStyle` (shared), element style (`DivStyle`/`TextStyle`), and `LayoutStyle` in `NodeLayout`.
- [ ] P1.2 explicitly separate build-time style containers from runtime style state types (no deferred bindings in runtime structs).
- [ ] P1.3 replace `unreachable!` downcasts in `NodeKind` with safe typed accessors that return `Result`.
- [ ] P1.4 add `NodeStateFlags` helper methods for hover/active/focus/disabled checks to reduce bitflag boilerplate.
- [ ] P1.5 add node-level state metadata needed later for focus and transitions (example: focusable, disabled source, transition config handle).

### Phase 2 - Event Pipeline Simplification

- [ ] P2.1 refactor `Engine::dispatch_event` into small methods: pointer move, pointer down, pointer up, focus in, focus out, resize, dpi changed.
- [ ] P2.2 centralize hover path diffing into one helper that returns entering/leaving/common paths.
- [ ] P2.3 define deterministic propagation semantics for pointer events (target-first/bubble order, stop propagation support plan).
- [ ] P2.4 introduce `EventContext` for callbacks (node id, pointer position, key modifiers, phase, stop flag).
- [ ] P2.5 upgrade event handler signatures from `Fn()` to context-aware callbacks while preserving ergonomic builder API.
- [ ] P2.6 add focus events at platform boundary (`WM_SETFOCUS`, `WM_KILLFOCUS`) and translate them to `EngineEvent`.

### Phase 3 - Generic Stateful Style Resolution

- [ ] P3.1 add a generic state patch model (`base + hover + active + focus + disabled`) for all relevant style domains.
- [ ] P3.2 define property classification table for dirty invalidation (`PAINT` vs `LAYOUT`) per style field.
- [ ] P3.3 implement resolver with strict precedence: `disabled > active > hover > focus > base`.
- [ ] P3.4 implement change diffing between previous effective style and next effective style.
- [ ] P3.5 apply only changed fields to runtime node data and mark precise dirty flags.
- [ ] P3.6 route all interaction-state mutations through one `apply_interaction_state` entrypoint.

### Phase 4 - Builder API Unification (Tailwind-like DX)

- [ ] P4.1 make `when(state, f)` the primitive state API.
- [ ] P4.2 implement sugar methods on top: `.hover(...)`, `.active(...)`, `.focus_visible(...)`, `.disabled(...)`.
- [ ] P4.3 ensure state builders exist for Div and Text from day one (no Div-only special case).
- [ ] P4.4 make style patch builders property-complete for current API surface (node props, div props, text props, layout props).
- [ ] P4.5 preserve existing fluent base-style methods unchanged to avoid migration pain.
- [ ] P4.6 add compile examples validating DSL ergonomics in docs/tests.

### Phase 5 - Transition and Animation Foundation

- [ ] P5.1 add transition spec type (duration, easing, delay, property groups).
- [ ] P5.2 attach transition config per node in runtime state.
- [ ] P5.3 on state changes, compute target style and create/retarget property animations.
- [ ] P5.4 add animation tick step integrated with frame scheduling.
- [ ] P5.5 update dirty flags from animated property groups per frame.
- [ ] P5.6 guarantee completion behavior (snap to final value, remove finished tracks).
- [ ] P5.7 handle interruption semantics (hover->active->hover quickly) without visual glitches.

### Phase 6 - Focus and Accessibility-Ready Behavior

- [ ] P6.1 add engine focus owner tracking and focus path updates.
- [ ] P6.2 define keyboard focus navigation strategy (initially tab order by tree traversal).
- [ ] P6.3 support focus-visible semantics separated from pointer focus where needed.
- [ ] P6.4 ensure focus changes trigger state resolution and transitions consistently.
- [ ] P6.5 define disabled interaction semantics affecting hit testing and event dispatch.

### Phase 7 - Renderer and Hit-Test Consistency

- [ ] P7.1 ensure render backend fully applies `transform` and `opacity` from effective styles.
- [ ] P7.2 ensure hit testing uses the same coordinate/transformation model as rendering.
- [ ] P7.3 define and implement hover semantics for overlapping nodes and z-index ordering.
- [ ] P7.4 keep interaction coordinate handling DPI-correct in all pointer/focus paths.

### Phase 8 - Performance and Memory

- [ ] P8.1 remove avoidable allocations in hover path diff and hit testing hot paths.
- [ ] P8.2 profile state resolution cost in dense trees and cache where it is safe.
- [ ] P8.3 avoid full-tree rebuilds when state changes are local and only paint-dirty.
- [ ] P8.4 verify transition engine does not leak subscriptions or animation tracks.
- [ ] P8.5 cap caches and verify cache invalidation on renderer/device resets.

### Phase 9 - Tests, Tooling, and Documentation

- [ ] P9.1 add unit tests for precedence resolution and style patch merge behavior.
- [ ] P9.2 add unit tests for dirty-flag mapping by changed property.
- [ ] P9.3 add integration tests for enter/leave/active/focus sequences including nested elements.
- [ ] P9.4 add transition tests for retargeting and interruption determinism.
- [ ] P9.5 add event propagation tests for stop-propagation scenarios.
- [ ] P9.6 update README and architecture docs with final interaction + transition model.

### Phase 10 - CustomNode Readiness

- [ ] P10.1 define a production-safe custom element contract without `Box<impl Style>` runtime erasure pitfalls.
- [ ] P10.2 add typed trait(s) for style patch resolution and render command emission for custom nodes.
- [ ] P10.3 ensure custom nodes participate in the same state resolver and transition pipeline.
- [ ] P10.4 add one real custom node example proving parity with Div/Text state behavior.

### Exit Criteria (Definition of Done)

- [ ] E1 hover/active/focus/disabled all work with deterministic precedence for Div and Text.
- [ ] E2 transitions run for configured properties and remain stable under rapid state changes.
- [ ] E3 no panic paths remain in state/event runtime code.
- [ ] E4 tests and benchmarks pass with no regressions versus baseline.
- [ ] E5 code paths are modular enough that adding one new element type does not require state-system redesign.

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
  .transition(|t| t.color(120).transform(90).ease(Ease::OutCubic));
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
