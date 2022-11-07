#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;

use stm32f3_discovery::stm32f3xx_hal::delay::Delay;
use stm32f3_discovery::stm32f3xx_hal::pac;
use stm32f3_discovery::stm32f3xx_hal::prelude::*;

use stm32f3_discovery::leds::Leds;
use stm32f3_discovery::switch_hal::{OutputSwitch, ToggleableOutputSwitch};

#[entry]
fn main() -> ! {
    let device_periphs = pac::Peripherals::take().unwrap();
    let core_preiphs = cortex_m::Peripherals::take().unwrap();

    // RCC is a peripheral on MCU. It's like I2C peripheral, which is embedded on a MCU.
    // RCC stands for Reset clock control and which controls the internal clocks to each peripheral on MCU.
    // For detailed information you can open any one of STM32 datasheet see it in Clock configuration tree diagram, you will get some idea.
    let mut reset_and_clock_control = device_periphs.RCC.constrain();
    let mut flash = device_periphs.FLASH.constrain();
    let clocks = reset_and_clock_control.cfgr.freeze(&mut flash.acr);
    let mut delay = Delay::new(core_preiphs.SYST, clocks);

    let mut gpioe = device_periphs.GPIOE.split(&mut reset_and_clock_control.ahb);
    // https://embeddedbucket.wordpress.com/2017/02/19/configuring-gpios-with-hal-and-cmsis-part-1/
    let mut leds = Leds::new(
        gpioe.pe8,
        gpioe.pe9,
        gpioe.pe10,
        gpioe.pe11,
        gpioe.pe12,
        gpioe.pe13,
        gpioe.pe14,
        gpioe.pe15,
        &mut gpioe.moder,
        &mut gpioe.otyper,
    );

    // loop {}

    loop {
        leds.ld3.toggle().ok();
        delay.delay_ms(1000u16);
        leds.ld3.toggle().ok();
        delay.delay_ms(1000u16);

        //explicit on/off
        leds.ld4.on().ok();
        delay.delay_ms(1000u16);
        leds.ld4.off().ok();
        delay.delay_ms(1000u16);
    }
}
