//! Hardware initialization for basic (non-async) example

use cortex_m::delay::Delay;
use rp2040_hal::{
    Sio,
    adc::Adc,
    clocks::{Clock, init_clocks_and_plls},
    gpio::Pins,
    pac,
    usb::UsbBus,
    watchdog::Watchdog,
};
use usb_device::{class_prelude::UsbBusAllocator, prelude::*};
use usbd_serial::SerialPort;
use static_cell::StaticCell;

use crate::hw_state;

// =============================================================================
// Hardware Initialization
// =============================================================================

/// Initialized hardware peripherals returned from setup
pub struct HardwareConfig {
    pub delay: Delay,
}

/// Initialize all hardware peripherals
///
/// This function configures:
/// - System clocks (125 MHz from 12 MHz crystal)
/// - GPIO pins
/// - USB device (CDC serial)
/// - ADC and temperature sensor
/// - LED on GP25
///
/// Returns the hardware configuration struct.
/// USB device and serial are initialized globally via `io::init_usb()`.
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

    // Initialize ADC and temperature sensor
    let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);
    let temp_sensor = adc.take_temp_sensor().unwrap();
    hw_state::init_temp_sensor(adc, temp_sensor);

    // Set up USB bus allocator (must be static)
    static USB_BUS: StaticCell<UsbBusAllocator<UsbBus>> = StaticCell::new();
    let usb_bus = USB_BUS.init(UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    )));

    // Create USB serial device
    let serial = SerialPort::new(usb_bus);

    // Create USB device
    let usb_device = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("Raspberry Pi")
            .product("Pico")
            .serial_number("nut-shell")])
        .unwrap()
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

    // Initialize USB globals
    crate::io::init_usb(usb_device, serial);

    HardwareConfig { delay }
}
