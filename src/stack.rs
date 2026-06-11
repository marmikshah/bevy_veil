//! The overlay stack: who's open, in what order, and therefore who draws on top
//! and who owns input. Spawn order — not entity id or sibling position —
//! decides layering, so it's deterministic across frames and respawns.

use bevy::prelude::*;

use crate::gate::UiCapturing;

/// The `GlobalZIndex` floor for overlays; each depth adds [`Z_STEP`]. Pick a
/// base well above ordinary HUD z so overlays always win.
pub(crate) const Z_BASE: i32 = 1000;
pub(crate) const Z_STEP: i32 = 10;

/// Marks an overlay root. The string is the caller's id (for find/dismiss).
#[derive(Component, Debug, Clone)]
pub struct Overlay {
    pub id: String,
    /// Whether Escape pops this overlay while it's on top (see the builder's
    /// `escape`). Off for state-driven overlays.
    pub pop_on_escape: bool,
}

/// The live stack of overlay root entities, bottom-to-top. The builder pushes;
/// despawning an overlay root prunes it (see [`prune_despawned_overlays`]).
#[derive(Resource, Default, Debug)]
pub struct OverlayStack {
    pub roots: Vec<Entity>,
}

impl OverlayStack {
    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }

    pub fn depth(&self) -> usize {
        self.roots.len()
    }

    /// The top-most (input-owning) overlay, if any.
    pub fn top(&self) -> Option<Entity> {
        self.roots.last().copied()
    }
}

/// Called from the spawn command (which holds `&mut World`): registers a new
/// root on top, hands back its depth so the caller can stamp `GlobalZIndex`,
/// and arms the capture gate.
pub(crate) fn push_root(world: &mut World, root: Entity) -> usize {
    let mut stack = world.resource_mut::<OverlayStack>();
    let depth = stack.roots.len();
    stack.roots.push(root);
    world.resource_mut::<UiCapturing>().0 = true;
    depth
}

/// Despawning an overlay root removes its `Overlay`; reconcile the stack and
/// release the gate when the last one closes. Removal-driven so callers just
/// `despawn()` the root (recursive — scrim and content go with it) and the
/// bookkeeping follows.
pub(crate) fn prune_despawned_overlays(
    mut removed: RemovedComponents<Overlay>,
    mut stack: ResMut<OverlayStack>,
    mut capturing: ResMut<UiCapturing>,
) {
    let mut changed = false;
    for entity in removed.read() {
        let before = stack.roots.len();
        stack.roots.retain(|&r| r != entity);
        changed |= stack.roots.len() != before;
    }
    if changed {
        capturing.0 = !stack.roots.is_empty();
    }
}

/// Desktop affordance: Escape pops only the top overlay (a stack, not a
/// blanket close), and only if that overlay opted in via `escape(true)`. Touch
/// builds dismiss via the scrim tap instead.
pub(crate) fn escape_pops_top(
    keys: Res<ButtonInput<KeyCode>>,
    stack: Res<OverlayStack>,
    overlays: Query<&Overlay>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::Escape)
        && let Some(top) = stack.top()
        && let Ok(overlay) = overlays.get(top)
        && overlay.pop_on_escape
    {
        commands.entity(top).despawn();
    }
}
