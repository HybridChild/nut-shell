//! Global hardware state for NUCLEO-H753ZI
//!
//! Holds the three user LEDs behind a single Mutex as an array of erased pins.

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use stm32h7xx_hal::gpio::{ErasedPin, Output, PushPull};

static LEDS: Mutex<RefCell<Option<[ErasedPin<Output<PushPull>>; 3]>>> =
    Mutex::new(RefCell::new(None));

pub(crate) fn init_leds(
    led1: ErasedPin<Output<PushPull>>,
    led2: ErasedPin<Output<PushPull>>,
    led3: ErasedPin<Output<PushPull>>,
) {
    cortex_m::interrupt::free(|cs| {
        LEDS.borrow(cs).replace(Some([led1, led2, led3]));
    });
}

/// Set LED by 1-based index (1 = LD1 green, 2 = LD2 yellow, 3 = LD3 red).
pub fn set_led(n: u8, on: bool) {
    let idx = (n as usize).wrapping_sub(1);
    cortex_m::interrupt::free(|cs| {
        if let Some(leds) = LEDS.borrow(cs).borrow_mut().as_mut() {
            if let Some(led) = leds.get_mut(idx) {
                if on {
                    led.set_high();
                } else {
                    led.set_low();
                }
            }
        }
    });
}

/// Toggle LED by 1-based index.
pub fn toggle_led(n: u8) {
    let idx = (n as usize).wrapping_sub(1);
    cortex_m::interrupt::free(|cs| {
        if let Some(leds) = LEDS.borrow(cs).borrow_mut().as_mut() {
            if let Some(led) = leds.get_mut(idx) {
                led.toggle();
            }
        }
    });
}
