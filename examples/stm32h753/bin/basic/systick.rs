//! SysTick timer for uptime tracking
//!
//! Provides a millisecond counter driven by the Cortex-M7 SysTick interrupt.
//! Configured to fire at 1 kHz (reload = CPU_freq / 1000 - 1).
//!
//! AtomicU32 is used instead of static mut — Cortex-M7 has native 32-bit atomics.

use core::sync::atomic::{AtomicU32, Ordering};

static MILLIS: AtomicU32 = AtomicU32::new(0);

pub fn millis() -> u32 {
    MILLIS.load(Ordering::Relaxed)
}

pub fn increment_millis() {
    MILLIS.fetch_add(1, Ordering::Relaxed);
}
