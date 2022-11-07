#![no_std]
#![no_main]

use panic_semihosting as _;

use stm32f3xx_hal as hal;
use ws2812_spi as ws2812;

use cortex_m_rt::entry;

use cortex_m_semihosting::hprintln;

use smart_leds::colors;
use smart_leds::RGB;

use hal::flash::FlashExt;
use hal::gpio::marker::Gpio;
use hal::gpio::marker::Index;
use hal::gpio::GpioExt;
use hal::gpio::Output;
use hal::gpio::Pin;
use hal::gpio::PushPull;
use hal::pac::Peripherals;
use hal::prelude::*;
use hal::rcc::RccExt;
use hal::spi::Spi;

use ws2812::Ws2812;

use smart_leds::SmartLedsWrite;

// struct Pins {
//     pub north: Pin<Gpio, Index, Output<PushPull>>,
// }

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut acr = dp.FLASH.constrain().acr;
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .pclk2(24.MHz())
        .freeze(&mut acr);

    // Set LEDs

    let sck =
        gpiob
            .pb3
            .into_af_push_pull::<5>(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let miso =
        gpiob
            .pb4
            .into_af_push_pull::<5>(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let mosi =
        gpiob
            .pb5
            .into_af_push_pull::<5>(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    // let sck =
    //     gpioa
    //         .pa5
    //         .into_af_push_pull::<5>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    // let miso =
    //     gpioa
    //         .pa6
    //         .into_af_push_pull::<5>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    // let mosi =
    //     gpioa
    //         .pa7
    //         .into_af_push_pull::<5>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);

    let spi = Spi::new(dp.SPI1, (sck, miso, mosi), 3.MHz(), clocks, &mut rcc.apb2);

    let mut ws = Ws2812::new(spi);
    let mut data = [colors::WHITE; 60];
    for i in 0..data.len() {
        data[i] = RGB { r: 0, g: 0, b: 0 }
        // data[i] = hsv2rgb(Hsv {
        //     hue: (i as u8) * 32,
        //     sat: 255,
        //     val: 32,
        // });
    }
    ws.write(data.iter().cloned()).unwrap();
    hprintln!("Written");

    let button = gpioa.pa0.into_input(&mut gpioa.moder);

    let mut north = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut north_east = gpioe
        .pe10
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut east = gpioe
        .pe11
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut south_east = gpioe
        .pe12
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut south = gpioe
        .pe13
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut south_west = gpioe
        .pe14
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut west = gpioe
        .pe15
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut north_west = gpioe
        .pe8
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    let mut leds = (
        north, north_east, east, south_east, south, south_west, west, north_west,
    );

    let mut show = false;
    let mut i = 0;
    loop {
        if button.is_high().unwrap() {
            show = !show;
        }

        if show {
            leds.0.set_low().unwrap();
            leds.1.set_low().unwrap();
            leds.2.set_low().unwrap();
            leds.3.set_low().unwrap();
            leds.4.set_low().unwrap();
            leds.5.set_low().unwrap();
            leds.6.set_low().unwrap();
            leds.7.set_low().unwrap();

            match i {
                0 => leds.0.set_high().unwrap(),
                1 => leds.1.set_high().unwrap(),
                2 => leds.2.set_high().unwrap(),
                3 => leds.3.set_high().unwrap(),
                4 => leds.4.set_high().unwrap(),
                5 => leds.5.set_high().unwrap(),
                6 => leds.6.set_high().unwrap(),
                7 => leds.7.set_high().unwrap(),
                _ => panic!("Invalid pin"),
            }
            if i == 7 {
                i = 0;
            } else {
                i += 1;
            }
            cortex_m::asm::delay(8_000_000);
        }
        // if button.is_high().unwrap() {
        //     east.set_high().unwrap();
        //     cortex_m::asm::delay(8_000_000);
        // } else {
        //     east.set_low().unwrap();
        // }
        // cortex_m::asm::delay(8_000_000);

        // led.toggle().unwrap();
    }

    // loop {
    //     leds.ld3.toggle().ok();
    //     delay.delay_ms(1000u16);
    //     leds.ld3.toggle().ok();
    //     delay.delay_ms(1000u16);

    //     //explicit on/off
    //     leds.ld4.on().ok();
    //     delay.delay_ms(1000u16);
    //     leds.ld4.off().ok();
    //     delay.delay_ms(1000u16);
    // }
}
