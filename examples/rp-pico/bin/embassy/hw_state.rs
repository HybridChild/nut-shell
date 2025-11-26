//! Global hardware state and control functions for Embassy example

use core::sync::atomic::{AtomicU32, Ordering};

// =============================================================================
// Global Hardware State
// =============================================================================

/// Cached temperature value (updated by background task, read by command)
static CACHED_TEMPERATURE: AtomicU32 = AtomicU32::new(0);

// =============================================================================
// Hardware Control Functions
// =============================================================================

/// Read the current temperature from the internal sensor
///
/// Returns the last temperature reading from the background monitor task.
/// Temperature is updated every 500ms by the temperature_monitor task.
pub fn read_temperature() -> f32 {
    let bits = CACHED_TEMPERATURE.load(Ordering::Relaxed);
    f32::from_bits(bits)
}

/// Update the cached temperature value (called from temperature_monitor task)
pub fn set_temperature(temp: f32) {
    CACHED_TEMPERATURE.store(temp.to_bits(), Ordering::Relaxed);
}
