//! Hardware initialization for basic (non-async) example

use cortex_m::delay::Delay;
use fugit::HertzU32;
use rp2040_hal::{
    Sio,
    adc::Adc,
    clocks::{Clock, init_clocks_and_plls},
    gpio::{FunctionUart, Pins},
    pac,
    uart::{DataBits, StopBits, UartConfig, UartPeripheral},
    watchdog::Watchdog,
};

use crate::{hw_state, io::UartType};

// =============================================================================
// Hardware Initialization
// =============================================================================

/// Initialized hardware peripherals returned from setup
pub struct HardwareConfig {
    pub uart: UartType,
    pub delay: Delay,
}

/// Initialize all hardware peripherals
///
/// This function configures:
/// - System clocks (125 MHz from 12 MHz crystal)
/// - GPIO pins
/// - UART0 on GP0/GP1 at 115200 baud
/// - ADC and temperature sensor
/// - LED on GP25
pub fn init_hardware(
    mut pac: pac::Peripherals,
    core: pac::CorePeripherals,
) -> HardwareConfig {
    // Set up watchdog
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    // Configure clocks (125 MHz system clock from 12 MHz crystal)
    let xosc_crystal_freq = 12_000_000;
    let clocks = init_clocks_and_plls(
        xosc_crystal_freq,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Set up delay provider
    let delay = Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Set up GPIO
    let sio = Sio::new(pac.SIO);
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure LED (GP25)
    let led = pins.gpio25.into_push_pull_output();
    hw_state::init_led(led);

    // Configure UART on GP0 (TX) and GP1 (RX)
    let uart_pins = (
        pins.gpio0.into_function::<FunctionUart>(),
        pins.gpio1.into_function::<FunctionUart>(),
    );

    let uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(
                HertzU32::from_raw(115200),
                DataBits::Eight,
                None,
                StopBits::One,
            ),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    // Initialize ADC and temperature sensor
    let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);
    let temp_sensor = adc.take_temp_sensor().unwrap();
    hw_state::init_temp_sensor(adc, temp_sensor);

    HardwareConfig { uart, delay }
}
