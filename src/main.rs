#![allow(unsafe_code)]
#![allow(warnings)]
#![allow(missing_docs)]
#![allow(unused_variables)]
#![no_main]
#![no_std]

#[rtic::app(device = stm32g4xx_hal::stm32g4::stm32g474, peripherals = true, dispatchers = [FDCAN3_INTR0, FDCAN3_INTR1])]
mod app {
    use stm32g4xx_hal::flash::{FlashExt, FlashSize, FlashWriter, Parts};
    use stm32g4xx_hal::prelude::*;
    use stm32g4xx_hal::rcc::{PllConfig, RccExt};

    use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics

    // Resources shared between tasks
    #[shared]
    struct Shared {}

    // Local resources to specific tasks (cannot be shared)
    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // let dp = Peripherals::take().unwrap();
        // let cp = cortex_m::Peripherals::take().expect("cannot take core peripherals");

        let dp = cx.device;
        let cp = cx.core;

        let rcc = dp.RCC.constrain();
        let mut pll_config = stm32g4xx_hal::rcc::PllConfig::default();

        // Sysclock is based on PLL_R
        pll_config.mux = stm32g4xx_hal::rcc::PLLSrc::HSI; // 16MHz
        pll_config.n = stm32g4xx_hal::rcc::PllNMul::MUL_64;
        pll_config.m = stm32g4xx_hal::rcc::PllMDiv::DIV_2; // f(vco) = 16MHz*64/2 = 512MHz
        pll_config.r = Some(stm32g4xx_hal::rcc::PllRDiv::DIV_4); // f(sysclock) = 512MHz/4 = 128MHz

        // Note to future self: The AHB clock runs the timers, among other things.
        // Please refer to the Clock Tree manual to determine if it is worth
        // changing to a lower speed for battery life savings.
        let mut clock_config = stm32g4xx_hal::rcc::Config::default()
            .pll_cfg(pll_config)
            .clock_src(stm32g4xx_hal::rcc::SysClockSrc::PLL);

        // After clock configuration, the following should be true:
        // Sysclock is 128MHz
        // AHB clock is 128MHz
        // APB1 clock is 128MHz
        // APB2 clock is 128MHz
        // The ADC will ultimately be put into synchronous mode and will derive
        // its clock from the AHB bus clock, with a prescalar of 2 or 4.

        let mut rcc = rcc.freeze(clock_config);

        // *** FLASH Memory ***
        let mut data = [0xBE, 0xEF, 0xCA, 0xFE];
        let mut flash = dp.FLASH.constrain();
        let mut flash_writer = flash.writer(FlashSize::Sz128K);
        let write_results = flash_writer.write(0x1FC00 as u32, &data);
        write_results.unwrap();
        let mut memory_read_results = flash_writer.read(0x1FC00 as u32, data.len());
        let memory_bytes = memory_read_results.unwrap();

        //adc.start_conversion();
        (
            // Initialization of shared resources
            Shared {},
            // Initialization of task local resources
            Local {},
            // Move the monotonic timer to the RTIC run-time, this enables
            // scheduling
            init::Monotonics(),
        )
    }

    // Background task, runs whenever no other tasks are running
    #[idle]
    fn idle(mut cx: idle::Context) -> ! {
        loop {
            // Sleep until next interrupt
            cortex_m::asm::wfi();
        }
    }
}
