//! Hardware initialization for Embassy async example

use embassy_rp::{
    adc::{Adc, Async as AdcAsync, Channel as AdcChannel, Config as AdcConfig, InterruptHandler as AdcInterruptHandler},
    bind_interrupts,
    gpio::{Level, Output},
    Peri,
};

// Bind ADC interrupt handler
bind_interrupts!(pub struct AdcIrqs {
    ADC_IRQ_FIFO => AdcInterruptHandler;
});

// =============================================================================
// Hardware Initialization
// =============================================================================

/// Initialized hardware peripherals returned from setup
pub struct HardwareConfig {
    pub adc: Adc<'static, AdcAsync>,
    pub temp_channel: AdcChannel<'static>,
    pub led: Output<'static>,
}

/// Initialize hardware peripherals for Embassy async runtime
///
/// This function configures:
/// - ADC for temperature monitoring
/// - Temperature sensor channel
/// - LED on GP25
///
/// Takes only the peripherals it needs, allowing main to use the rest (UART, etc.)
pub fn init_hardware(
    adc: Peri<'static, embassy_rp::peripherals::ADC>,
    adc_temp_sensor: Peri<'static, embassy_rp::peripherals::ADC_TEMP_SENSOR>,
    led_pin: Peri<'static, embassy_rp::peripherals::PIN_25>,
) -> HardwareConfig {
    // Initialize ADC for temperature monitoring
    let adc = Adc::new(adc, AdcIrqs, AdcConfig::default());
    let temp_channel = AdcChannel::new_temp_sensor(adc_temp_sensor);

    // Set up onboard LED (GP25)
    let led = Output::new(led_pin, Level::Low);

    HardwareConfig {
        adc,
        temp_channel,
        led,
    }
}
