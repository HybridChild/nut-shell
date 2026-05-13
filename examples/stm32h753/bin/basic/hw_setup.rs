//! Hardware initialization for NUCLEO-H753ZI
//!
//! Configures:
//! - VOS1 power mode (default; avoids stm32h7xx-hal CSI startup issue with VOS0)
//! - System clock: HSI 64 MHz → PLL1 → 200 MHz
//! - USB kernel clock: HSI48 (48 MHz, enabled by freeze; selected via kernel_usb_clk_mux)
//! - GPIO: LEDs (PB0, PE1, PB14), USB power enable (PD10)
//! - USB OTG2_HS: PA11 (DM, AF10), PA12 (DP, AF10) → CN13
//! - SysTick: 1 ms interrupt at 200 MHz
//!
//! HSI is used instead of HSE because SB45 (ST-LINK MCO → HSE) is not guaranteed
//! to be closed or active on all NUCLEO-H753ZI board revisions.
//!
//! # Board wiring (NUCLEO-H753ZI default solder bridges)
//! - SB21 ON: PA11 connected to CN13 DM
//! - SB22 ON: PA12 connected to CN13 DP
//! - SB76 ON: PG7 overcurrent alarm connected
//! - SB77 ON: PD10 connected to U18 USB power switch
//!
//! # Warning
//! CN13 cannot power the board. Power via CN1 (ST-LINK USB) before connecting CN13.

use cortex_m::peripheral::syst::SystClkSource;
use static_cell::StaticCell;
use stm32h7xx_hal::{
    prelude::*,
    rcc::rec::UsbClkSel,
    usb_hs::{USB2, UsbBus},
};
use usb_device::{bus::UsbBusAllocator, device::StringDescriptors, prelude::*};
use usbd_serial::SerialPort;

use crate::hw_state;

// =============================================================================
// Hardware configuration output
// =============================================================================

pub struct HardwareConfig {}

// =============================================================================
// Initialization
// =============================================================================

/// Initialize all hardware peripherals.
pub fn init_hardware(
    dp: stm32h7xx_hal::pac::Peripherals,
    mut core: cortex_m::Peripherals,
) -> HardwareConfig {
    // ------------------------------------------------------------------
    // 1. Power — VOS1 default (Scale 1).
    //    VOS0 (480 MHz) causes a CSI oscillator hang in stm32h7xx-hal 0.16
    //    freeze() on this board; VOS1 is stable and sufficient for USB CDC.
    // ------------------------------------------------------------------
    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();

    // ------------------------------------------------------------------
    // 2. Clocks
    //    Source: HSI 64 MHz (no HSE — SB45 is not reliable on all board revisions).
    //    VOS1 allows up to 400 MHz; 200 MHz chosen for safe margin.
    //    HSI48 is enabled inside freeze() and is always available after it.
    // ------------------------------------------------------------------
    let mut ccdr = dp
        .RCC
        .constrain()
        .sys_ck(200.MHz())
        .freeze(pwrcfg, &dp.SYSCFG);

    // Verify HSI48 is running and select it as USB kernel clock (RCC_D2CCIP2R.USBSEL = HSI48)
    let _ = ccdr.clocks.hsi48_ck().expect("HSI48 must run");
    ccdr.peripheral.kernel_usb_clk_mux(UsbClkSel::Hsi48);

    // ------------------------------------------------------------------
    // 3. GPIO
    // ------------------------------------------------------------------
    let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);
    let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
    let gpiod = dp.GPIOD.split(ccdr.peripheral.GPIOD);
    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);

    // User LEDs: LD1 green (PB0), LD2 yellow (PE1), LD3 red (PB14)
    let led1 = gpiob.pb0.into_push_pull_output();
    let led2 = gpioe.pe1.into_push_pull_output();
    let led3 = gpiob.pb14.into_push_pull_output();
    hw_state::init_leds(led1.erase(), led2.erase(), led3.erase());

    // USB power enable: PD10 → U18 power switch (SB77 ON = default)
    let mut usb_pwr_en = gpiod.pd10.into_push_pull_output();
    usb_pwr_en.set_high();

    // ------------------------------------------------------------------
    // 4. USB OTG2_HS
    //    PA11 → DM (AF10), PA12 → DP (AF10); type-inferred from USB2::new signature.
    //    Note: EP_MEMORY must be in AXI SRAM (0x24000000) — accessible by USB DMA.
    //    DTCM (0x20000000) is CPU-only and cannot be used.
    // ------------------------------------------------------------------
    let pin_dm = gpioa.pa11.into_alternate();
    let pin_dp = gpioa.pa12.into_alternate();

    let usb2 = USB2::new(
        dp.OTG2_HS_GLOBAL,
        dp.OTG2_HS_DEVICE,
        dp.OTG2_HS_PWRCLK,
        pin_dm,
        pin_dp,
        ccdr.peripheral.USB2OTG,
        &ccdr.clocks,
    );

    // Endpoint memory — placed in AXI SRAM by default linker script (RAM section).
    // Use addr_of_mut! to get a raw pointer without creating a `&mut static` reference
    // (which is forbidden by the `static_mut_refs` lint in Rust 2024 edition).
    static mut EP_MEMORY: [u32; 1024] = [0u32; 1024];
    let ep_mem: &'static mut [u32] = unsafe { &mut *core::ptr::addr_of_mut!(EP_MEMORY) };

    // UsbBus::new returns UsbBusAllocator<UsbBus<USB2>> directly
    static USB_BUS: StaticCell<UsbBusAllocator<UsbBus<USB2>>> = StaticCell::new();
    let usb_bus = USB_BUS.init(UsbBus::new(usb2, ep_mem));

    // ------------------------------------------------------------------
    // 5. USB CDC-ACM serial device
    // ------------------------------------------------------------------
    let serial = SerialPort::new(usb_bus);

    let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("nut-shell")
            .product("NUCLEO-H753ZI")
            .serial_number("basic")])
        .unwrap()
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

    crate::io::init_usb(usb_dev, serial);

    // ------------------------------------------------------------------
    // 6. SysTick at 1 ms — reload derived from actual sys_ck so it stays
    //    correct if the clock configuration changes.
    // ------------------------------------------------------------------
    let reload = ccdr.clocks.sys_ck().raw() / 1_000 - 1;
    core.SYST.set_clock_source(SystClkSource::Core);
    core.SYST.set_reload(reload);
    core.SYST.clear_current();
    core.SYST.enable_counter();
    core.SYST.enable_interrupt();

    HardwareConfig {}
}
