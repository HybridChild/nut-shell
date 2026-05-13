//! Global hardware state for NUCLEO-H753ZI Embassy example
//!
//! Stores the three user LEDs behind an embassy-sync blocking mutex.

use core::cell::RefCell;
use embassy_stm32::gpio::Output;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

static LEDS: Mutex<CriticalSectionRawMutex, RefCell<Option<[Output<'static>; 3]>>> =
    Mutex::new(RefCell::new(None));

pub fn init_leds(led1: Output<'static>, led2: Output<'static>, led3: Output<'static>) {
    LEDS.lock(|cell| {
        cell.replace(Some([led1, led2, led3]));
    });
}

/// Set LED by 1-based index (1 = LD1 green, 2 = LD2 yellow, 3 = LD3 red).
pub fn set_led(n: u8, on: bool) {
    let idx = (n as usize).wrapping_sub(1);
    LEDS.lock(|cell| {
        if let Some(leds) = cell.borrow_mut().as_mut() {
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
    LEDS.lock(|cell| {
        if let Some(leds) = cell.borrow_mut().as_mut() {
            if let Some(led) = leds.get_mut(idx) {
                led.toggle();
            }
        }
    });
}
