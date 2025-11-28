//! Global hardware state and control functions for NUCLEO-F072RB

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use stm32f0xx_hal::{
    adc::{Adc, VTemp},
    gpio::{Output, PushPull, gpioa::PA5},
    prelude::*,
};

// =============================================================================
// Type Aliases
// =============================================================================

pub type LedPin = PA5<Output<PushPull>>;
pub type AdcType = Adc;

// =============================================================================
// Global Hardware State
// =============================================================================

/// Global LED pin protected by a Mutex for safe access from command handlers
static LED_PIN: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));

/// Global ADC for reading temperature sensor
static ADC: Mutex<RefCell<Option<AdcType>>> = Mutex::new(RefCell::new(None));

// =============================================================================
// Initialization Functions
// =============================================================================

/// Initialize the LED pin (must be called once during startup)
pub(crate) fn init_led(led: LedPin) {
    cortex_m::interrupt::free(|cs| {
        LED_PIN.borrow(cs).replace(Some(led));
    });
}

/// Initialize the ADC (must be called once during startup)
pub(crate) fn init_adc(adc: AdcType) {
    cortex_m::interrupt::free(|cs| {
        ADC.borrow(cs).replace(Some(adc));
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

/// Read the current temperature from the internal sensor
pub fn read_temperature() -> f32 {
    cortex_m::interrupt::free(|cs| {
        let mut adc_ref = ADC.borrow(cs).borrow_mut();

        if let Some(adc) = adc_ref.as_mut() {
            // Read temperature sensor using VTemp (ADC Channel 16)
            // Returns temperature in 10ths of a degree Celsius (e.g., 235 = 23.5°C)
            let temp_tenths: i16 = VTemp::read(adc, None);

            // Convert from tenths to actual degrees Celsius
            let temp_celsius = temp_tenths as f32 / 10.0;

            return temp_celsius;
        }

        // Return a default value if reading fails
        0.0
    })
}
