#![no_std]
#![no_main]

use panic_semihosting as _;

use stm32f3xx_hal as _;

extern crate stm32f3xx_hal as hal;

use rtic::app;
use rtt_target::{rprintln, rtt_init_print};
use stm32f3xx_hal::gpio::{Output, PushPull, PE10};
use stm32f3xx_hal::prelude::*;
use systick_monotonic::{fugit::Duration, Systick};

use lsm303dlhc::Lsm303dlhc;

use l3gd20::L3gd20;

// Referenced https://github.com/rtic-rs/rtic-examples/tree/master/rtic_v1/stm32f3_blinky/src
#[app(device = stm32f3xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {
    use cortex_m_semihosting::hprintln;

    use super::*;

    #[shared]
    struct Shared {
        led: PE10<Output<PushPull>>,
    }

    #[local]
    struct Local {
        button: hal::gpio::Pin<hal::gpio::Gpioa, hal::gpio::U<0>, hal::gpio::Input>,
        state: bool,
        magnetometer_sensor: lsm303dlhc::Lsm303dlhc<
            hal::i2c::I2c<
                stm32f3xx_hal::pac::I2C1,
                (
                    hal::gpio::Pin<
                        hal::gpio::Gpiob,
                        hal::gpio::U<6>,
                        hal::gpio::Alternate<hal::gpio::OpenDrain, 4>,
                    >,
                    hal::gpio::Pin<
                        hal::gpio::Gpiob,
                        hal::gpio::U<7>,
                        hal::gpio::Alternate<hal::gpio::OpenDrain, 4>,
                    >,
                ),
            >,
        >,
        gyroscope_sensor: L3gd20<
            hal::spi::Spi<
                stm32f3xx_hal::pac::SPI1,
                (
                    hal::gpio::Pin<
                        hal::gpio::Gpioa,
                        hal::gpio::U<5>,
                        hal::gpio::Alternate<PushPull, 5>,
                    >,
                    hal::gpio::Pin<
                        hal::gpio::Gpioa,
                        hal::gpio::U<6>,
                        hal::gpio::Alternate<PushPull, 5>,
                    >,
                    hal::gpio::Pin<
                        hal::gpio::Gpioa,
                        hal::gpio::U<7>,
                        hal::gpio::Alternate<PushPull, 5>,
                    >,
                ),
            >,
            hal::gpio::Pin<hal::gpio::Gpioe, hal::gpio::U<3>, Output<PushPull>>,
        >,
    }

    #[monotonic(binds = SysTick, default = true)]
    type MonoTimer = Systick<1000>;

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Setup clocks
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();

        let mono = Systick::new(cx.core.SYST, 36_000_000);

        rtt_init_print!();
        rprintln!("init");

        let clocks = rcc
            .cfgr
            .use_hse(8.MHz())
            .sysclk(36.MHz())
            .pclk1(36.MHz())
            .freeze(&mut flash.acr);

        let mut gpioa = cx.device.GPIOA.split(&mut rcc.ahb);

        // Enable interrupts on button press
        let mut button = gpioa
            .pa0
            .into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr);
        button.trigger_on_edge(&mut cx.device.EXTI, hal::gpio::Edge::Rising);
        button.enable_interrupt(&mut cx.device.EXTI);

        // Setup LED
        let mut gpioe = cx.device.GPIOE.split(&mut rcc.ahb);
        let mut led = gpioe
            .pe10
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
        led.set_high().unwrap();

        // Setup mag
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.ahb);
        let scl =
            gpiob
                .pb6
                .into_af4_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
        // .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
        let sda =
            gpiob
                .pb7
                .into_af4_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
        let i2c = hal::i2c::I2c::new(
            cx.device.I2C1,
            (scl, sda),
            400_000.Hz(),
            clocks,
            &mut rcc.apb1,
        );
        let magnetometer_sensor = Lsm303dlhc::new(i2c).unwrap();

        let sck =
            gpioa
                .pa5
                .into_af5_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
        let miso =
            gpioa
                .pa6
                .into_af5_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
        let mosi =
            gpioa
                .pa7
                .into_af5_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
        let nss = gpioe
            .pe3
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
        let spi = hal::spi::Spi::new(
            cx.device.SPI1,
            (sck, miso, mosi),
            1.MHz(),
            clocks,
            &mut rcc.apb2,
        );
        let gyroscope_sensor = L3gd20::new(spi, nss).unwrap();

        // Schedule the blinking task
        blink::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();
        // Schedule magnetometer task
        magnetometer::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();
        // Schedule gyro task
        gyroscope::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();

        (
            Shared { led },
            Local {
                button,
                state: false,
                magnetometer_sensor,
                gyroscope_sensor,
            },
            init::Monotonics(mono),
        )
    }

    #[task(priority = 3, binds = EXTI0, local=[button], shared=[led])]
    fn button_press(mut cx: button_press::Context) {
        // https://lonesometraveler.github.io/2020/04/17/GPIO_interrupt.html
        // https://apollolabsblog.hashnode.dev/stm32f4-embedded-rust-at-the-hal-gpio-interrupts
        cx.local.button.clear_interrupt();
        cx.shared.led.lock(|led| {
            led.toggle().unwrap();
        });
        // let on = cx.local.button.is_high().unwrap();
        // cx.shared.led.lock(|led| {
        //     if on {
        //         (*led).set_high().unwrap();
        //     } else {
        //         (*led).set_low().unwrap();
        //     }
        // })
    }

    #[task(local = [state], shared = [led])]
    fn blink(cx: blink::Context) {
        rprintln!("blink");
        // if *cx.local.state {
        //     cx.local.led.set_high().unwrap();
        //     *cx.local.state = false;
        // } else {
        //     cx.local.led.set_low().unwrap();
        //     *cx.local.state = true;
        // }
        // blink::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();
    }

    #[task(local = [magnetometer_sensor])]
    fn magnetometer(cx: magnetometer::Context) {
        let lsm303dlhc::I16x3 { x, y, z } = cx.local.magnetometer_sensor.accel().unwrap();
        // hprintln!("{} {} {}", x, y, z);
        magnetometer::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();
    }

    #[task(local = [gyroscope_sensor])]
    fn gyroscope(cx: gyroscope::Context) {
        let l3gd20::I16x3 { x, y, z } = cx.local.gyroscope_sensor.gyro().unwrap();
        // hprintln!("{} {} {}", x, y, z);
        gyroscope::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();
    }
}
