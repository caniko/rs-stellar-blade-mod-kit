//! Stellar Blade Bouldy reconnaissance mod.
//!
//! This crate only discovers combat-related Unreal symbols. It does not patch
//! gameplay, register game-specific hooks, or assume offsets.

mod recon;

use std::sync::atomic::{AtomicBool, Ordering};

use bouldy_runtime::prelude::*;

static SCAN_ATTEMPTED: AtomicBool = AtomicBool::new(false);

#[unreal_mod]
#[derive(Default)]
struct StellarBladeReconMod;

impl Mod for StellarBladeReconMod {
    fn on_init(&mut self, ctx: &mut ModContext) {
        ctx.log("Stellar Blade Bouldy reconnaissance mod initialized");
        run_scan_once(ctx);
    }

    fn on_tick(&mut self, _delta: f32) {
        if !SCAN_ATTEMPTED.load(Ordering::Relaxed) {
            bouldy_runtime::log("Stellar Blade reconnaissance waiting for discovery API");
        }
    }

    fn on_shutdown(&mut self) {
        bouldy_runtime::log("Stellar Blade Bouldy reconnaissance mod shutting down");
    }
}

fn run_scan_once(ctx: &mut ModContext) {
    if SCAN_ATTEMPTED.swap(true, Ordering::AcqRel) {
        return;
    }

    let Some(discovery) = ctx.discovery() else {
        ctx.log("Bouldy discovery API unavailable; deploy with a V3 discovery shim");
        return;
    };

    let exported = recon::scan_combat_candidates(discovery);
    ctx.log(&format!(
        "Stellar Blade combat reconnaissance exported {exported} candidate records"
    ));
}
