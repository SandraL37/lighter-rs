# TODOs (prioritized)

## [CANONICAL A++ ROADMAP - INTERACTION STATES + TRANSITIONS]

### Phase 0 - Baseline and Safety

- [x] P0.1 create a dedicated branch for the state/interaction refactor.
- [x] P0.2 define baseline behavior snapshots for hover, click, and resize sample apps (video/gif + expected logs).
- [x] P0.3 add a short architecture note documenting current runtime data ownership across `NodeData`, `NodeLayout`, and renderer commands.
- [x] P0.4 add compile-time guards/tests for no panic paths in interaction code.
- [x] P0.5 add a minimal benchmark scene for many hoverable nodes to track regressions during refactor.

### Phase 1 - Runtime Data Model Hardening

- [x] P1.1 keep runtime split: `NodeStyle` (shared), element style (`DivStyle`/`TextStyle`), and `LayoutStyle` in `NodeLayout`.
- [x] P1.2 explicitly separate build-time style containers from runtime style state types (no deferred bindings in runtime structs).
- [x] P1.3 replace `unreachable!` downcasts in `NodeKind` with safe typed accessors that return `Result`.
- [x] P1.4 add `NodeStateFlags` helper methods for hover/active/focus/disabled checks to reduce bitflag boilerplate.
- [x] P1.5 add node-level state metadata needed later for focus and transitions (example: focusable, disabled source, transition config handle).

### Phase 2 - Event Pipeline Simplification

- [x] P2.1 refactor `Engine::dispatch_event` into small methods: pointer move, pointer down, pointer up, focus in, focus out, resize, dpi changed.
- [x] P2.2 centralize hover path diffing into one helper that returns entering/leaving/common paths.
- [x] P2.3 define deterministic propagation semantics for pointer events (target-first/bubble order, stop propagation support plan).
- [x] P2.4 introduce `EventContext` for callbacks (node id, pointer position, key modifiers, phase, stop flag).
- [x] P2.5 upgrade event handler signatures from `Fn()` to context-aware callbacks while preserving ergonomic builder API.
- [x] P2.6 add focus events at platform boundary (`WM_SETFOCUS`, `WM_KILLFOCUS`) and translate them to `EngineEvent`.

### Phase 3 - Generic Stateful Style Resolution

- [x] P3.1 add a generic state patch model (`base + hover + active + focus + disabled`) for all relevant style domains.
- [x] P3.2 define property classification table for dirty invalidation (`PAINT` vs `LAYOUT`) per style field.
- [x] P3.3 implement resolver with strict precedence: `disabled > active > hover > focus > base`.
- [x] P3.4 implement change diffing between previous effective style and next effective style.
- [x] P3.5 apply only changed fields to runtime node data and mark precise dirty flags.
- [x] P3.6 route all interaction-state mutations through one `apply_interaction_state` entrypoint.

### ~~ Phase 4 - Builder API Unification (Tailwind-like DX) ~~

- [x] P4.1 make `when(state, f)` the primitive state API.
- [x] P4.2 implement sugar methods on top: `.hover(...)`, `.active(...)`, `.focus_visible(...)`, `.disabled(...)`.
- [x] P4.3 ensure state builders exist for Div and Text from day one (no Div-only special case).
- [x] P4.4 make style patch builders property-complete for current API surface (node props, div props, text props, layout props).
- [x] P4.5 preserve existing fluent base-style methods unchanged to avoid migration pain.
- [x] P4.6 add compile examples validating DSL ergonomics in docs/tests.~~

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
- [x] fix impl trait system for elements/components so public API stays ergonomic while allowing reusable, typed component composition.
- [x] add stop event propagation in event system (capture/bubble control): allow handlers to stop bubbling for click/hover flows to avoid conflicting parent handlers.
- [ ] active / focus / hover styles: wire pseudo-state style resolution to events and invalidation so state changes update paint/layout predictably.
- [ ] bubble hover behavior: define and implement consistent hover enter/leave bubbling semantics across nested elements.
- [ ] add dcomp integration path (DirectComposition) for better composition/presentation control and future visual effects pipeline.
- [x] remove debug I/O from render loop (`src/core/app/engine.rs`): `println!("\rrendered frame")` executes each frame and can heavily degrade performance in interactive scenes.

## [LESS IMPORTANT]

- [x] deduplicate layout style builder code (`src/core/layout.rs`): `ContainerStylePropsImpl` and `LeafStylePropsExt` duplicate many setters (`w/h/size/max_* /m`), increasing maintenance cost and inconsistency risk; extract shared helper/macro.
- [ ] reduce clone pressure during layout (`src/core/layout/engine.rs`): `node_kind.clone()` and `style.clone()` are done in `compute_child_layout`; borrow where possible or split read paths to avoid repeated cloning.
- [x] harden type-safe node downcasts (`src/core/arena/node.rs`): replace `unreachable!("Not a div/text")` in `as_div_mut`/`as_text_mut` with safe error-returning APIs to avoid undefined behavior patterns during future element expansion.
- [ ] debug devtools: expand diagnostics for layout tree, dirty flags, event paths, and cache stats to reduce time spent debugging framework internals.
- [ ] add default styling/theme baseline (tokens + defaults for text/background/spacing) so elements are usable without verbose style setup.
- [x] `src/core/app/window.rs` remove (or manage) the silent error in the WM_PAINT branch.

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
use my_ui::*;

fn app(
    is_error:    Signal<bool>,
    is_loading:  Signal<bool>,
    accent:      Signal<Color>,
    font_scale:  Signal<f32>,
    count:       Signal<i32>,
) -> impl Element {


    // ─── 1. BARE VALUES — simplest case ────────────────────────────────────────

    div()
        .bg(RED)
        .opacity(1.0f32)
        .rounding(px(8.0))
        .width(px(200.0))
        .height(px(48.0))
        .font_size(rem(1.0))


    // ─── 2. STATES — instant snap, no transitions ───────────────────────────────

    div()
        .bg(BLUE_500.hover(BLUE_400).active(BLUE_600).disabled(GRAY_300))
        .opacity(1.0f32.hover(0.9f32).active(0.8f32).disabled(0.4f32))
        .rounding(px(8.0).hover(px(12.0)))
        .border_width(px(1.0).hover(px(2.0)))
        .border_color(BLUE_600.hover(BLUE_300).disabled(GRAY_400))
        .text_color(WHITE.disabled(GRAY_500))
        .cursor(Cursor::Pointer.disabled(Cursor::NotAllowed))


    // ─── 3. TRANSITIONS — uniform enter/exit ────────────────────────────────────

    div()
        .bg(BLUE_500
            .hover(BLUE_400).transition(150.ms(), EASE_IN_OUT)
            .active(BLUE_600).transition(80.ms(), EASE_IN_OUT)
            .disabled(GRAY_300).transition(200.ms(), EASE_OUT)
        )
        .opacity(1.0f32
            .hover(0.9f32).transition(150.ms(), EASE_OUT)
            .disabled(0.4f32).transition(200.ms(), EASE_IN_OUT)
        )
        .rounding(px(8.0)
            .hover(px(12.0)).transition(200.ms(), EASE_OUT)
        )


    // ─── 4. TRANSITIONS — asymmetric enter/exit ─────────────────────────────────

    div()
        .bg(RED
            .hover(GREEN).enter(100.ms(), EASE_IN).exit(250.ms(), EASE_OUT)
            .active(BLUE).enter(50.ms(),  EASE_IN).exit(150.ms(), EASE_OUT)
        )
        .shadow(Shadow::sm()
            .hover(Shadow::lg()).enter(200.ms(), EASE_OUT).exit(300.ms(), EASE_IN)
            .active(Shadow::none()).enter(60.ms(), EASE_IN).exit(100.ms(), EASE_OUT)
        )
        .translate_y(px(0.0)
            .hover(px(-4.0)).enter(200.ms(), EASE_OUT).exit(300.ms(), EASE_IN)
        )


    // ─── 5. SIGNALS — reactive values, no states ────────────────────────────────

    div()
        // Signal<Color> coerces directly
        .bg(accent)

        // Signal<f32> coerces directly
        .opacity(is_loading.map(|l| if l { 0.5 } else { 1.0 }))

        // Signal<Px> for layout
        .width(count.map(|c| px(c as f32 * 40.0)))

        // Signal<bool> driving a style choice
        .border_color(is_error.map(|e| if e { RED_500 } else { GRAY_300 }))

        // Derived signal combining two signals
        .font_size(font_scale.map(|s| rem(s)))


    // ─── 6. SIGNALS + STATES — reactive property with interactive states ─────────

    div()
        // The entire Property is driven by a signal
        // hover/active colors derive from the same signal
        .bg(accent.map(|c| {
            c.hover(c.lighten(0.1))
             .active(c.darken(0.1))
             .disabled(c.desaturate(0.8))
        }))

        // Error state drives both value AND states
        .border_color(is_error.map(|e| {
            if e { RED_500.hover(RED_400).active(RED_600) }
            else { GRAY_300.hover(GRAY_400).active(GRAY_500) }
        }))

        // Loading drives opacity including its hover behavior
        .opacity(is_loading.map(|l| {
            if l { 0.6f32.hover(0.6f32) }   // no hover effect while loading
            else { 1.0f32.hover(0.9f32) }
        }))

        // Cursor reacts to multiple signals
        .cursor(is_loading.map(|l| {
            if l { Cursor::Wait }
            else { Cursor::Pointer.disabled(Cursor::NotAllowed) }
        }))


    // ─── 7. SIGNALS + STATES + TRANSITIONS ──────────────────────────────────────

    div()
        .bg(accent.map(|c| {
            c
            .hover(c.lighten(0.1)).enter(100.ms(), EASE_OUT).exit(200.ms(), EASE_IN)
            .active(c.darken(0.1)).transition(60.ms(), EASE_IN)
        }))
        .shadow(is_error.map(|e| {
            if e {
                Shadow::error()
                    .hover(Shadow::error_lg()).transition(150.ms(), EASE_OUT)
            } else {
                Shadow::sm()
                    .hover(Shadow::lg()).enter(200.ms(), EASE_OUT).exit(300.ms(), EASE_IN)
            }
        }))


    // ─── 8. VISUAL PERCENT — resolved against own size post-layout ──────────────

    // Circle — percent rounding resolved against own min side
    div()
        .width(px(48.0))
        .height(px(48.0))
        .rounding(percent(50.0))
        .bg(BLUE_500.hover(BLUE_400).transition(150.ms(), EASE_OUT))

    // Font size relative to parent
    div()
        .font_size(em(1.2))
        .line_height(em(1.5))
        .letter_spacing(em(0.02))

    // Font size relative to root, also reactive
    div()
        .font_size(font_scale.map(|s| rem(s)))


    // ─── 9. LAYOUT — taffy properties ───────────────────────────────────────────

    div()
        .width(px(200.0))
        .width(percent(50.0))
        .width(auto())
        .height(px(48.0))
        .min_width(px(100.0))
        .max_width(px(600.0))
        .padding(px(16.0))
        .padding_x(px(24.0))
        .padding_y(px(12.0))
        .margin(px(8.0))
        .margin_top(auto())
        .gap(px(8.0))
        .flex_grow(1.0)
        .flex_shrink(0.0)
        .flex_basis(percent(50.0))


    // ─── 10. MIXED — some props animated, some instant, some reactive ────────────

    div()
        .bg(RED
            .hover(GREEN).transition(200.ms(), EASE_OUT)   // animated
        )
        .rounding(px(8.0)
            .hover(px(16.0))                               // instant snap
        )
        .cursor(Cursor::Pointer)                           // never animates
        .opacity(is_loading.map(|l| {                      // reactive + animated
            if l { 0.5f32 }
            else { 1.0f32 }
        }))
        .border_color(is_error.map(|e| {                   // reactive + states + transition
            if e { RED_500.hover(RED_400).transition(100.ms(), EASE_OUT) }
            else { GRAY_300.hover(GRAY_400).transition(100.ms(), EASE_OUT) }
        }))


    // ─── 11. REAL WORLD — primary button ────────────────────────────────────────

    div()
        .width(px(160.0))
        .height(px(44.0))
        .padding_x(px(20.0))
        .padding_y(px(10.0))
        .rounding(px(8.0)
            .hover(px(12.0)).transition(200.ms(), EASE_OUT)
        )
        .bg(accent.map(|c| {
            c
            .hover(c.lighten(0.1)).enter(100.ms(), EASE_OUT).exit(200.ms(), EASE_IN)
            .active(c.darken(0.15)).transition(60.ms(), EASE_IN)
            .disabled(GRAY_300)
        }))
        .opacity(is_loading.map(|l| {
            if l { 0.7f32 } else { 1.0f32 }
        }))
        .shadow(Shadow::md()
            .hover(Shadow::lg()).enter(200.ms(), EASE_OUT).exit(300.ms(), EASE_IN)
            .active(Shadow::none()).transition(60.ms(), EASE_IN)
            .disabled(Shadow::none())
        )
        .border_width(px(1.0))
        .border_color(accent.map(|c| {
            c.darken(0.1)
             .hover(c.lighten(0.1)).transition(100.ms(), EASE_OUT)
             .disabled(GRAY_400)
        }))
        .cursor(is_loading.map(|l| {
            if l { Cursor::Wait }
            else { Cursor::Pointer.disabled(Cursor::NotAllowed) }
        }))
        .text_color(WHITE.disabled(GRAY_500))
        .font_size(rem(0.875))
        .font_weight(FontWeight::Medium)
        .translate_y(px(0.0)
            .hover(px(-1.0)).enter(100.ms(), EASE_OUT).exit(150.ms(), EASE_IN)
            .active(px(1.0)).transition(60.ms(), EASE_IN)
        )


    // ─── 12. REAL WORLD — input field ────────────────────────────────────────────

    div()
        .width(percent(100.0))
        .height(px(40.0))
        .padding_x(px(12.0))
        .rounding(px(6.0))
        .bg(WHITE.disabled(GRAY_50))
        .border_width(px(1.0)
            .focused(px(2.0)).transition(100.ms(), EASE_OUT)
        )
        .border_color(is_error.map(|e| {
            if e {
                RED_500
                    .hover(RED_400).transition(150.ms(), EASE_OUT)
                    .focused(RED_500).transition(100.ms(), EASE_OUT)
            } else {
                GRAY_300
                    .hover(GRAY_400).transition(150.ms(), EASE_OUT)
                    .focused(BLUE_500).transition(100.ms(), EASE_OUT)
                    .disabled(GRAY_200)
            }
        }))
        .shadow(Shadow::none()
            .focused(Shadow::focus_ring(BLUE_500)).transition(100.ms(), EASE_OUT)
        )
        .text_color(GRAY_900.disabled(GRAY_400))
        .font_size(rem(0.875))


    // ─── 13. REAL WORLD — card with hover lift ───────────────────────────────────

    div()
        .width(px(320.0))
        .padding(px(24.0))
        .gap(px(16.0))
        .rounding(px(12.0))
        .bg(WHITE
            .hover(GRAY_50).transition(200.ms(), EASE_OUT)
        )
        .border_width(px(1.0))
        .border_color(GRAY_200
            .hover(GRAY_300).transition(200.ms(), EASE_OUT)
        )
        .shadow(Shadow::sm()
            .hover(Shadow::xl()).enter(200.ms(), EASE_OUT).exit(300.ms(), EASE_IN)
        )
        .translate_y(px(0.0)
            .hover(px(-4.0)).enter(200.ms(), EASE_OUT).exit(300.ms(), EASE_IN)
        )
        .cursor(Cursor::Pointer)
}
```

BUG WHEN CLICKING IF THE HOVER EXIST AND THE ACTIVE NOT HOVER SHOULD PASS BUT THIS DOESN'T HAPPEN
property along with the state should hold the runtime value
