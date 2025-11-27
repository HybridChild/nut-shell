//! SysTick timer for uptime tracking
//!
//! This module implements a simple millisecond counter using the SysTick timer.
//! The counter increments every 1ms via the SysTick interrupt handler.

// =============================================================================
// Global Millisecond Counter
// =============================================================================

/// Global millisecond counter, incremented by SysTick interrupt
/// Using static mut since Cortex-M0 doesn't have atomic operations
static mut MILLIS: u32 = 0;

// =============================================================================
// Public API
// =============================================================================

/// Get the current millisecond count since boot
///
/// This function can be called from any context (main code or interrupts)
/// and returns the current uptime in milliseconds.
///
/// # Safety
/// This is safe to call because:
/// - Reads are atomic on ARM Cortex-M (32-bit aligned)
/// - Only the interrupt increments, main code only reads
pub fn millis() -> u32 {
    unsafe { MILLIS }
}

/// Increment the millisecond counter (called from SysTick interrupt)
///
/// # Safety
/// This function should only be called from the SysTick interrupt handler.
pub fn increment_millis() {
    unsafe {
        MILLIS = MILLIS.wrapping_add(1);
    }
}
