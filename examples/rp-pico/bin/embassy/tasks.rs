//! Embassy async tasks for hardware control

use embassy_rp::{
    adc::{Adc, Async as AdcAsync, Channel as AdcChannel},
    gpio::Output,
};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};

use crate::handlers::LedCommand;
use crate::hw_state;

// =============================================================================
// Embassy Tasks
// =============================================================================

/// LED control task.
///
/// Receives LED commands from the command handler and controls the LED accordingly.
#[embassy_executor::task]
pub async fn led_task(
    mut led: Output<'static>,
    channel: &'static Channel<ThreadModeRawMutex, LedCommand, 1>,
) {
    loop {
        match channel.receive().await {
            LedCommand::On => led.set_high(),
            LedCommand::Off => led.set_low(),
        }
    }
}

/// Temperature monitoring task - periodically reads and caches temperature.
///
/// This background task reads the temperature sensor every 500ms and updates
/// the cached value that's returned by the `temp` command via `hw_state::read_temperature()`.
#[embassy_executor::task]
pub async fn temperature_monitor(
    mut adc: Adc<'static, AdcAsync>,
    mut temp_channel: AdcChannel<'static>,
) {
    loop {
        // Read temperature sensor
        let adc_value = adc.read(&mut temp_channel).await.unwrap();

        // Convert ADC value to temperature (RP2040 formula)
        // T = 27 - (ADC_voltage - 0.706) / 0.001721
        // ADC_voltage = (adc_value * 3.3) / 4096
        let adc_voltage = (adc_value as f32 * 3.3) / 4096.0;
        let temp_celsius = 27.0 - (adc_voltage - 0.706) / 0.001721;

        // Update cached temperature
        hw_state::set_temperature(temp_celsius);

        // Update every 500ms
        Timer::after(Duration::from_millis(500)).await;
    }
}
