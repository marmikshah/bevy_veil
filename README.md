# bevy_veil

A modal/overlay stack for native `bevy_ui`. It fixes one specific, recurring
bug — **stacked popups leak input** — and folds away the `children!` /
`SpawnWith` boilerplate while it's there. No retained widget framework, no
layout engine: it emits ordinary `bevy_ui` nodes.

> **Status: experimental (0.1, pre-release).** Built to be dogfooded across
> several games before it hits crates.io; the API will move. Consume via a path
> or git dependency for now.

## The bug it kills

Spawn a small popup over a larger one in raw `bevy_ui` and the larger popup's
buttons stay clickable around the edges of the smaller one — picking only
occludes where nodes overlap, and the small popup doesn't cover them. Worse, a
game that reads `Touches` / mouse **directly** for its own controls bypasses UI
picking entirely, so gameplay keeps responding *under* the popup.

`bevy_veil` closes both with two occlusion planes:

| Plane | Mechanism |
|-------|-----------|
| **UI → UI** | Every overlay owns a full-screen `Pickable` **scrim** that blocks all lower picks — regardless of the top panel's size. |
| **UI → gameplay** | A `UiCapturing` resource flips true while any overlay is open. Raw-input systems gate on it via the `ui_not_capturing` run condition. |

Layering is by spawn order (deterministic `GlobalZIndex` per depth), not entity
id or sibling position.

## Usage

```rust
use bevy::prelude::*;
use bevy_veil::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, VeilPlugin))
        // .insert_resource(Theme { /* your chrome + fonts */ })  // optional
        .run();
}

fn open_pause(mut commands: Commands) {
    overlay(&mut commands, "pause")
        .title("PAUSED")
        .button("Resume", |c| { c.queue(|w: &mut World| { /* resume */ }); })
        .button("Quit", |c| { c.queue(|w: &mut World| { /* to menu */ }); })
        .dismissable(true)   // tap scrim to close; Esc pops the top
        .push();
}
```

## The input-gate contract (read this)

The library cannot reach into your game's bespoke input reads. **You** must gate
every system that consumes raw input for gameplay:

```rust
app.add_systems(Update, rotate_player.run_if(ui_not_capturing));
```

Skip this and the UI→gameplay plane does nothing — your popups will look modal
but the game keeps playing underneath. `ui_not_capturing` defaults to *passing*
(capturing is false), so headless tests and fairness sims are never gated.

## Theming

Everything is driven by an injected `Theme` resource (colours, fonts, border
widths, button-state alphas). A neutral dark default is registered by
`VeilPlugin` so examples run with zero setup; insert your own to match your
game's chrome.

## Compatibility

| `bevy_veil` | `bevy` |
|-------------|--------|
| 0.1         | 0.18   |

Pre-release: consume via a path dependency (`bevy_veil = { path = "../bevy_veil" }`)
and dogfood before this hits crates.io.

## Example

```
cargo run --example stacked
```

Two stacked overlays; the spinning sprite freezes while either is open.

## Building overlays

Two tiers, pick per screen:

- **Built-in panel** — `overlay(c, id).title(..).body(..).button(label, on_click)`.
  veil builds the panel; good for simple dialogs.
- **Bespoke content** — `overlay(c, id).content(|parent| { /* your bevy_ui */ })`.
  veil owns the (scrimmed, stacked, gated) root; you fill it. For settings
  grids, icon rows, anything veil shouldn't try to model.

Lifecycle is yours to choose:

- **veil-driven** — `.dismissable(true)` (scrim tap) and/or `.escape(true)`
  (Esc) pop the overlay.
- **state-driven** — spawn on `OnEnter`, despawn the `Overlay` on `OnExit`;
  leave dismiss/escape **off** so the state machine stays authoritative.

## Limitations / roadmap

- **No focus trap or directional (keyboard/gamepad) nav yet.** Touch/mouse
  only. This is the main gap for full accessibility and is the next planned
  addition.
- Built-in panel is intentionally minimal (title / body / buttons). Richer
  layouts go through `.content()` — by design, not omission.
- No `dismiss_by_id` helper yet; despawn the `Overlay` entity directly.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at
your option.
