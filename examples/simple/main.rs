#![no_main]
#![no_std]

//! Example for using the BME680 sensor.
//!
//! ## STM32F411RE - Nucleo 64
//!
//! Connect the BME680 sensor to the I2C port #1 using pins PB8/PB9.
//!
//! ## STM32F723E - DISCOVERY
//!
//! Connect the BME680 to the STM32F723E DISCOVERY board via the Grove I2C connector
//! on the extension board.

#[cfg(feature = "display")]
mod display;

#[cfg(feature = "display")]
use display::Display;

use panic_rtt_target as _;
use rtt_target::rtt_init_print;

#[cfg(any(feature = "stm32f7xx", feature = "stm32f4xx"))]
use cortex_m_rt::entry;

#[cfg(feature = "stm32f4xx")]
use stm32f4 as _;
#[cfg(feature = "stm32f4xx")]
use stm32f4::stm32f411 as stm32;
#[cfg(feature = "stm32f4xx")]
use stm32f4xx_hal as hal;
#[cfg(feature = "stm32f4xx")]
use stm32f4xx_hal::stm32::Peripherals as DevicePeripherals;

#[cfg(feature = "stm32f4xx")]
use hal::i2c::I2c;

#[cfg(feature = "stm32f7xx")]
use stm32f7 as _;
#[cfg(feature = "stm32f7xx")]
use stm32f7xx_hal as hal;
#[cfg(feature = "stm32f7xx")]
use stm32f7xx_hal::device::Peripherals as DevicePeripherals;

#[cfg(feature = "stm32f7xx")]
use hal::i2c::{BlockingI2c, Mode};

#[cfg(all(feature = "stm32f4xx", feature = "display"))]
use stm32f4xx_hal::{
    gpio::{
        gpioa::{PA5, PA6, PA7},
        gpioc::{PC4, PC5},
        AF5,
    },
    spi::Spi,
};

use hal::interrupt;

#[cfg(all(feature = "stm32f4xx", feature = "display"))]
use stm32f4::stm32f411::SPI1;

#[cfg(feature = "display")]
use embedded_hal::digital::v2::OutputPin;
#[cfg(feature = "display")]
use embedded_hal::spi::MODE_0;
#[cfg(feature = "display")]
use hal::gpio::{Alternate, Output, PushPull};

use hal::{delay::Delay, gpio::GpioExt, rcc::RccExt, time::U32Ext};

use drogue_bme680::{
    Address, Bme680Controller, Bme680Sensor, Configuration, DelayMsWrapper, Filter, Oversampling,
    StaticProvider,
};

use log::LevelFilter;
use rtt_logger::RTTLogger;

use drogue_bsec::{Bsec, Inputs, SampleRate};
use drogue_embedded_timer::{MillisecondsClock1, MillisecondsTicker1};
use embedded_time::duration::Milliseconds;

use core::cell::RefCell;
use core::convert::TryInto;
use core::ops::DerefMut;
use cortex_m::interrupt::{free, Mutex};
use embedded_time::{Clock, Instant};
use hal::timer::Timer;
use stm32f4xx_hal::timer::Event;

static LOGGER: RTTLogger = RTTLogger::new(LevelFilter::Info);

static CLOCK: MillisecondsClock1 = MillisecondsClock1::new();

#[cfg(feature = "stm32f4xx")]
type TIM = stm32f4::stm32f411::TIM2;

type ClockType = MillisecondsClock1;
type TimerType = hal::timer::Timer<TIM>;
type TickerType = MillisecondsTicker1<'static, ClockType, TimerType, fn(&mut TimerType)>;

static TICKER: Mutex<RefCell<Option<TickerType>>> = Mutex::new(RefCell::new(None));

#[allow(non_snake_case)]
#[interrupt]
fn TIM2() {
    free(|cs| {
        if let Some(ref mut ticker) = TICKER.borrow(cs).borrow_mut().deref_mut() {
            ticker.tick()
        }
    });
}

fn get_now() -> Milliseconds {
    CLOCK
        .try_now()
        .unwrap()
        .duration_since_epoch()
        .try_into()
        .unwrap()
}

#[entry]
fn main() -> ! {
    rtt_init_print!(NoBlockSkip, 4096);
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
    log::info!("Starting up...");

    let p = DevicePeripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    #[cfg(feature = "stm32f4xx")]
    let rcc = p.RCC.constrain();
    #[cfg(feature = "stm32f7xx")]
    let mut rcc = p.RCC.constrain();

    #[cfg(feature = "stm32f4xx")]
    let clocks = rcc.cfgr.sysclk(100.mhz()).freeze();
    #[cfg(feature = "stm32f7xx")]
    let clocks = rcc.cfgr.sysclk(216.mhz()).freeze();

    #[cfg(all(feature = "stm32f4xx", feature = "display"))]
    let gpioa = p.GPIOA.split();
    #[cfg(feature = "stm32f4xx")]
    let gpiob = p.GPIOB.split();
    #[cfg(all(feature = "stm32f4xx", feature = "display"))]
    let gpioc = p.GPIOC.split();
    #[cfg(feature = "stm32f7xx")]
    let gpioh = p.GPIOH.split();

    // delay implementation

    let delay = Delay::new(cp.SYST, clocks);
    let mut delay = DelayMsWrapper::new(delay);

    let mut tim = Timer::tim2(p.TIM2, 1000.hz(), clocks);
    tim.listen(Event::TimeOut);

    free(|cs| {
        let mut ticker = CLOCK.ticker(
            tim,
            (|t| {
                t.clear_interrupt(Event::TimeOut);
            }) as fn(&mut TimerType),
        );
        ticker.tick();
        TICKER.borrow(cs).replace(Some(ticker));
    });
    unsafe {
        stm32::NVIC::unmask(hal::stm32::Interrupt::TIM2);
    }

    // display
    #[cfg(feature = "display")]
    let mut display = if cfg!(feature = "display") {
        log::info!("Init display");

        // init display
        let mosi = gpioa.pa7.into_alternate_af5();
        let miso = gpioa.pa6.into_alternate_af5();
        let sck = gpioa.pa5.into_alternate_af5();

        let mut cs = gpioc.pc13.into_push_pull_output();
        let dc = gpioc.pc5.into_push_pull_output();
        let rst = gpioc.pc4.into_push_pull_output();

        cs.set_low().unwrap();

        let spi = Spi::spi1(p.SPI1, (sck, miso, mosi), MODE_0, 1000.khz().into(), clocks);
        let mut display = Display::new(spi, dc, rst);
        display.init(&mut delay).unwrap();
        Some(display)
    } else {
        None
    };

    // init

    #[cfg(feature = "stm32f4xx")]
    let (sda, scl) = {
        let sda = gpiob.pb9.into_alternate_af4_open_drain();
        let scl = gpiob.pb8.into_alternate_af4_open_drain();
        (sda, scl)
    };
    #[cfg(feature = "stm32f7xx")]
    let (sda, scl) = {
        let sda = gpioh.ph5.into_alternate_af4();
        let scl = gpioh.ph4.into_alternate_af4();
        (sda, scl)
    };

    // Initialize I2C

    #[cfg(feature = "stm32f4xx")]
    let i2c = I2c::i2c1(p.I2C1, (scl, sda), 100.khz(), clocks);
    #[cfg(feature = "stm32f7xx")]
    let i2c = BlockingI2c::i2c2(
        p.I2C2,
        (scl, sda),
        Mode::standard(100_000.hz()),
        clocks,
        &mut rcc.apb1,
        100,
    );

    let bme680 = Bme680Sensor::from(i2c, Address::Secondary).unwrap();

    let mut controller =
        Bme680Controller::new(bme680, delay, Configuration::standard(), StaticProvider(25))
            .unwrap();

    let mut cnt = 0;

    let mut bsec = Bsec::new().unwrap().unwrap();

    bsec.update_subscription(
        SampleRate::LowPower,
        drogue_bsec::bsec_virtual_sensor_t::ALL,
    )
    .unwrap();

    let mut next = Milliseconds(0u32);
    loop {
        let now = get_now();

        if now > next {
            log::info!("start timestamp: {}", now);

            let control = bsec.sensor_control(now).unwrap();
            next = control.next_call;

            // process "control"
            let config = Configuration {
                filter: Filter::Coefficient3,
                temperature: control.temperature_oversampling,
                pressure: control.pressure_oversampling,
                humidity: control.humidity_oversampling,
                heater_temperature: control.heater_temperature,
                heater_duration: control.heating_duration,
                run_gas: control.run_gas,
            };

            controller.update_configuration(config).unwrap();

            if control.trigger_measurement {
                let data = controller.measure_default().unwrap().unwrap();

                let gas_resistance = if control.run_gas {
                    Some(data.gas_resistance)
                } else {
                    None
                };

                let inputs = Inputs {
                    temperature: Some(data.temperature),
                    humidity: Some(data.humidity),
                    pressure: data.pressure,
                    gas_resistance,
                };

                log::info!("Processing inputs: {:?}", inputs);

                let outputs = bsec.process_data(get_now(), &inputs).unwrap();

                if let Some(display) = &mut display {
                    display.set_state(&data, &outputs).unwrap();
                }

                log::info!("end timestamp - {}", get_now());
            }
        }

        // sleep

        let now = get_now();
        let diff = next - now;
        if next > now {
            if diff > Milliseconds(0u32) {
                log::info!("Sleeping for: {} ms", diff);
                controller.delay(diff);
            }
        }
    }
}
