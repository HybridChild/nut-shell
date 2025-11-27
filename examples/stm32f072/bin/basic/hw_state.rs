//! Global hardware state and control functions for NUCLEO-F072RB

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use stm32f0xx_hal::{
    gpio::{
        gpioa::PA5,
        Output, PushPull,
    },
    prelude::*,
};

// =============================================================================
// Type Aliases
// =============================================================================

pub type LedPin = PA5<Output<PushPull>>;

// =============================================================================
// Global Hardware State
// =============================================================================

/// Global LED pin protected by a Mutex for safe access from command handlers
static LED_PIN: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));

// =============================================================================
// Initialization Functions
// =============================================================================

/// Initialize the LED pin (must be called once during startup)
pub(crate) fn init_led(led: LedPin) {
    cortex_m::interrupt::free(|cs| {
        LED_PIN.borrow(cs).replace(Some(led));
    });
}

// =============================================================================
// Hardware Control Functions
// =============================================================================

/// Set the LED state (on = true, off = false)
pub fn set_led(on: bool) {
    cortex_m::interrupt::free(|cs| {
        if let Some(led) = LED_PIN.borrow(cs).borrow_mut().as_mut() {
            if on {
                let _ = led.set_high();
            } else {
                let _ = led.set_low();
            }
        }
    });
}
