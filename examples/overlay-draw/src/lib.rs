//! Example native plugin: drawing on the map view every frame (docs/abi.md, "Overlay", "Drawing
//! every frame"). It needs no capability. It draws two things in a match or a replay:
//!
//! - a small card pinned to the top-left corner of the map view, which uses every call of the
//!   `W2rOverlay` table once (it follows the screen, because it is placed from `view`);
//! - a ring with a caption around tile (8, 8), which stays on the map and scrolls with it.

use core::sync::atomic::{AtomicPtr, Ordering};

use w2r_abi::*;

static HOST: AtomicPtr<W2rHost> = AtomicPtr::new(core::ptr::null_mut());

/// Colours are `r | g << 8 | b << 16 | a << 24`.
const fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16) | ((a as u32) << 24)
}

const GOLD: u32 = rgba(255, 210, 90, 255);
const WHITE: u32 = rgba(255, 255, 255, 255);
const GREEN: u32 = rgba(60, 200, 80, 255);
const TRACK: u32 = rgba(0, 0, 0, 160);

fn s(text: &str) -> W2rStr {
    W2rStr { ptr: text.as_ptr().cast(), len: text.len() as u32 }
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> W2rRect {
    W2rRect { x, y, w, h }
}

/// Called by the host every frame of a match or a replay. Everything is in world pixels.
unsafe extern "C" fn draw(overlay: *const W2rOverlay) {
    let Some(o) = (unsafe { overlay.as_ref() }) else { return };
    if o.struct_size < W2R_OVERLAY_V1_SIZE {
        return;
    }
    unsafe {
        // ---- pinned to the view's corner -----------------------------------------------------
        let (x, y) = (o.view.x + 24.0, o.view.y + 48.0);
        let card = rect(x, y, 220.0, 100.0);
        (o.fill)(o.ctx, &card, TRACK, 6.0);
        (o.outline)(o.ctx, &card, GOLD, 1.0);
        (o.label)(o.ctx, &rect(x + 8.0, y + 6.0, 204.0, 16.0), s("Overlay drawing"), W2R_ALIGN_LEFT, GOLD);
        (o.line)(o.ctx, x + 8.0, y + 26.0, x + 212.0, y + 26.0, GOLD, 1.0);

        // A bar: a see-through track, then the filled part over it.
        (o.fill)(o.ctx, &rect(x + 8.0, y + 36.0, 120.0, 4.0), TRACK, 0.0);
        (o.fill)(o.ctx, &rect(x + 8.0, y + 36.0, 80.0, 4.0), GREEN, 0.0);

        // A ring and a filled circle.
        (o.circle)(o.ctx, x + 150.0, y + 38.0, 8.0, WHITE, 2.0);
        (o.circle)(o.ctx, x + 180.0, y + 38.0, 8.0, GREEN, 0.0);

        // `measure`: a box exactly as wide as the text in it. Colour 0 is the HUD's text colour.
        let text = "measured text";
        let width = (o.measure)(o.ctx, s(text), 0);
        (o.fill)(o.ctx, &rect(x + 8.0, y + 56.0, width, 16.0), TRACK, 0.0);
        (o.label)(o.ctx, &rect(x + 8.0, y + 56.0, width, 16.0), s(text), W2R_ALIGN_LEFT, 0);
        (o.label)(o.ctx, &rect(x + 8.0, y + 78.0, 204.0, 16.0), s("right"), W2R_ALIGN_RIGHT, WHITE);

        // ---- on the map: tile (8, 8), whose centre is tile * 32 + 16 ----------------------------
        let (cx, cy) = (8.0 * 32.0 + 16.0, 8.0 * 32.0 + 16.0);
        (o.circle)(o.ctx, cx, cy, 24.0, GOLD, 2.0);
        (o.label)(o.ctx, &rect(cx - 60.0, cy + 26.0, 120.0, 16.0), s("tile 8, 8"), W2R_ALIGN_CENTRE, GOLD);
    }
}

unsafe extern "C" fn on_load(host: *const W2rHost) -> W2rStatus {
    HOST.store(host.cast_mut(), Ordering::Release);
    let Some(h) = (unsafe { host.as_ref() }) else { return W2R_E_STATE };
    // An older host has no overlay drawing: fail the load so the card says why.
    if h.struct_size < W2R_HOST_OVERLAY_DRAW_SIZE {
        let msg = "this core has no overlay drawing (needs 2.7.0)";
        unsafe { (h.log)(h.token, W2R_LOG_ERROR, s(msg)) };
        return W2R_E_STATE;
    }
    unsafe { (h.overlay_draw_register)(h.token, Some(draw)) }
}

unsafe extern "C" fn on_unload() {
    if let Some(h) = unsafe { HOST.load(Ordering::Acquire).as_ref() } {
        unsafe { (h.overlay_draw_register)(h.token, None) };
    }
}

static TABLE: W2rPlugin = W2rPlugin {
    struct_size: W2R_PLUGIN_V1_SIZE,
    on_load: Some(on_load),
    on_event: None,
    on_ui: None,
    on_unload: Some(on_unload),
};

/// The one export every plugin provides.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn w2r_plugin_main(_host: *const W2rHost) -> *const W2rPlugin {
    &TABLE
}
