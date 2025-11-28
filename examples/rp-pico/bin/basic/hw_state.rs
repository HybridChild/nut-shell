//! Global hardware state and control functions for basic example

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use embedded_hal_0_2::adc::OneShot;
use embedded_hal_0_2::digital::v2::OutputPin;
use rp2040_hal::{
    adc::Adc,
    gpio::{FunctionSio, Pin, PullDown, SioOutput},
};

// =============================================================================
// Type Aliases
// =============================================================================

pub type LedPin = Pin<rp2040_hal::gpio::bank0::Gpio25, FunctionSio<SioOutput>, PullDown>;

pub type TempSensor = rp2040_hal::adc::TempSense;
pub type AdcType = Adc;

// =============================================================================
// Global Hardware State
// =============================================================================

/// Global LED pin protected by a Mutex for safe access from command handlers
static LED_PIN: Mutex<RefCell<Option<LedPin>>> = Mutex::new(RefCell::new(None));

/// Global ADC and temperature sensor for reading temperature
static ADC: Mutex<RefCell<Option<AdcType>>> = Mutex::new(RefCell::new(None));
static TEMP_SENSOR: Mutex<RefCell<Option<TempSensor>>> = Mutex::new(RefCell::new(None));

// =============================================================================
// Initialization Functions
// =============================================================================

/// Initialize the LED pin (must be called once during startup)
pub(crate) fn init_led(led: LedPin) {
    cortex_m::interrupt::free(|cs| {
        LED_PIN.borrow(cs).replace(Some(led));
    });
}

/// Initialize the ADC and temperature sensor (must be called once during startup)
pub(crate) fn init_temp_sensor(adc: AdcType, sensor: TempSensor) {
    cortex_m::interrupt::free(|cs| {
        ADC.borrow(cs).replace(Some(adc));
        TEMP_SENSOR.borrow(cs).replace(Some(sensor));
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
        let mut sensor_ref = TEMP_SENSOR.borrow(cs).borrow_mut();

        if let (Some(adc), Some(sensor)) = (adc_ref.as_mut(), sensor_ref.as_mut()) {
            // Read temperature sensor (OneShot trait returns nb::Result)
            let read_result: nb::Result<u16, _> = adc.read(sensor);
            if let Ok(adc_value) = read_result {
                // Convert ADC value to temperature (RP2040 formula)
                // T = 27 - (ADC_voltage - 0.706) / 0.001721
                // ADC_voltage = (adc_value * 3.3) / 4096
                let adc_voltage = (adc_value as f32 * 3.3) / 4096.0;
                let temp_celsius = 27.0 - (adc_voltage - 0.706) / 0.001721;
                return temp_celsius;
            }
        }

        // Return a default value if reading fails
        0.0
    })
}
