//! Hardware initialization for NUCLEO-F072RB

use cortex_m::delay::Delay;
use stm32f0xx_hal::{
    pac,
    prelude::*,
    serial::Serial,
};

use crate::io::{UartTx, UartRx};

// =============================================================================
// Hardware Initialization
// =============================================================================

/// Initialized hardware peripherals returned from setup
pub struct HardwareConfig {
    pub uart_tx: UartTx,
    pub uart_rx: UartRx,
    pub delay: Delay,
}

/// Initialize all hardware peripherals
///
/// This function configures:
/// - System clocks (48 MHz from internal HSI48)
/// - GPIO pins
/// - USART2 on PA2/PA3 at 115200 baud (connected to ST-LINK VCP)
/// - LED on PA5
pub fn init_hardware(
    mut pac: pac::Peripherals,
    core: pac::CorePeripherals,
) -> HardwareConfig {
    // Configure clocks
    let mut rcc = pac.RCC.configure().sysclk(48.mhz()).freeze(&mut pac.FLASH);

    // Set up delay provider
    let delay = Delay::new(core.SYST, rcc.clocks.sysclk().0);

    // Get GPIO ports
    let gpioa = pac.GPIOA.split(&mut rcc);

    // Configure LED (PA5) - user LED on NUCLEO-F072RB
    let led = cortex_m::interrupt::free(|cs| gpioa.pa5.into_push_pull_output(cs));
    crate::hw_state::init_led(led);

    // Configure USART2 pins (PA2=TX, PA3=RX)
    let tx_pin = cortex_m::interrupt::free(|cs| gpioa.pa2.into_alternate_af1(cs));
    let rx_pin = cortex_m::interrupt::free(|cs| gpioa.pa3.into_alternate_af1(cs));

    // Initialize USART2 at 115200 baud
    let serial = Serial::usart2(pac.USART2, (tx_pin, rx_pin), 115_200.bps(), &mut rcc);

    // Split into TX and RX
    let (uart_tx, uart_rx) = serial.split();

    HardwareConfig {
        uart_tx,
        uart_rx,
        delay,
    }
}
