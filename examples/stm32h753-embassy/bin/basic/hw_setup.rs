//! Clock and RCC configuration for NUCLEO-H753ZI Embassy example
//!
//! Clock configuration:
//! - HSI: 64 MHz (internal; HSE/SB45 not required)
//! - SYSCLK: 200 MHz (PLL1, VOS1) — HSI/4 × 50 / 4
//! - USB kernel clock: 48 MHz (PLL3Q) — HSI/4 × 12 / 4
//!
//! VOS1 (Scale1) selected for stability; VOS0 allows 480 MHz but is unnecessary here.
//! HSI48 is not used for USB — PLL3Q avoids the internal HSI48 oscillator entirely.

use embassy_stm32::Config;
use embassy_stm32::rcc::*;

/// Build the embassy-stm32 Config with the desired clock tree.
///
/// Call this before `embassy_stm32::init()`.
pub fn rcc_config() -> Config {
    let mut config = Config::default();
    {
        let rcc = &mut config.rcc;

        // HSI at full speed (64 MHz, no prescaler)
        rcc.hsi = Some(HSIPrescaler::DIV1);
        rcc.csi = false;
        rcc.hsi48 = None; // Not used — USB clock comes from PLL3Q

        // PLL1: 200 MHz SYSCLK
        // HSI(64) / 4 = 16 MHz → × 50 = 800 MHz VCO → / 4 = 200 MHz
        rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV4), // 200 MHz SYSCLK
            divq: None,
            divr: None,
        });

        // PLL3: 48 MHz for USB kernel clock
        // HSI(64) / 4 = 16 MHz → × 12 = 192 MHz VCO → / 4 = 48 MHz
        rcc.pll3 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL12,
            divp: None,
            divq: Some(PllDiv::DIV4), // 48 MHz → USB
            divr: None,
        });

        rcc.sys = Sysclk::PLL1_P; // 200 MHz
        rcc.ahb_pre = AHBPrescaler::DIV1; // 200 MHz (within VOS1 limit)
        rcc.apb1_pre = APBPrescaler::DIV2; // 100 MHz
        rcc.apb2_pre = APBPrescaler::DIV2; // 100 MHz
        rcc.apb3_pre = APBPrescaler::DIV2; // 100 MHz
        rcc.apb4_pre = APBPrescaler::DIV2; // 100 MHz

        // VOS1: allows CPU up to 400 MHz, AHB up to 200 MHz
        rcc.voltage_scale = VoltageScale::Scale1;

        // Route PLL3Q (48 MHz) to USB kernel clock
        // Note: Usbsel::HSI48 must NOT be used on H753ZI — it panics if HSI48 is not running.
        rcc.mux.usbsel = mux::Usbsel::PLL3_Q;
    }
    config
}
