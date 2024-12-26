#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(const_trait_impl)]

use embassy_futures::block_on;
use esp_println::println;
mod vcdprint;
use vcdprint::*;

extern crate alloc;
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use core::{cell::RefCell, error::Error};
use embedded_hal_bus::i2c;

use ina3221::{AveragingMode, Current, Voltage, INA3221};

use embedded_graphics::{
    geometry::AnchorY,
    image::Image,
    mock_display::ColorMapping,
    mono_font::{
        ascii::{
            FONT_10X20, FONT_4X6, FONT_5X7, FONT_6X10, FONT_6X13_BOLD, FONT_7X13, FONT_9X15_BOLD,
            FONT_9X18_BOLD,
        },
        MonoTextStyleBuilder,
    },
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
    primitives::Rectangle,
    text::{Alignment, Baseline, LineHeight, Text, TextStyleBuilder},
};

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Ticker, TimeoutError, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{self as hal, gpio::Io, peripherals};

use hal::{
    analog::adc::{Adc, AdcConfig, AdcPin, Attenuation},
    clock::CpuClock,
    gpio::{AnalogPin, Flex, Input, Level, Output, Pin, Pull},
    i2c::master::I2c,
    peripherals::{Peripherals, I2C0},
    prelude::*,
    spi::{
        master::{Config as SpiConfig, Spi},
        SpiMode,
    },
    timer::systimer::{SystemTimer, Target},
    timer::timg::TimerGroup,
};

use quadrature_encoder::{
    Async, Blocking, FullStep, HalfStep, IncrementalEncoder, InputPinError, PollMode, QuadStep,
    Rotary, RotaryMovement,
};
use ssd1306_i2c::{prelude::*, Builder}; // was use sh1106:: ...

type RotaryEncoder<Clk, Dt, Steps = FullStep, T = i32, PM = Blocking> =
    IncrementalEncoder<Rotary, Clk, Dt, Steps, T, PM>;

#[embassy_executor::task]
async fn encoder_task(
    mut rot_clk: Flex<'static>,
    mut rot_dat: Flex<'static>,
    mut rot_sw: Flex<'static>,
) {
    rot_clk.set_as_input(Pull::Up);
    rot_dat.set_as_input(Pull::Up);
    rot_sw.set_as_input(Pull::Up);

    // let mut rot: RotaryEncoder<_, _, FullStep, i32, Async> = RotaryEncoder::new(rot_clk, rot_dat).into_async();
    let mut rot: RotaryEncoder<_, _, FullStep, i32, Async> =
        RotaryEncoder::new(rot_clk, rot_dat).into_async().reversed();

    let t0 = Instant::now();
    let u = TimeUnit::Us;
    let s = TimeScale::_1;
    let mut vcd = VcdFile::default();
    vcd.set_version("LTT alpha").set_timescale(s, u);
    vcd.root_scope().add_var("btn", "b", VarType::Wire, 1);
    vcd.root_scope()
        .add_var("state", "s", VarType::Reg, 2)
        .set_reference("state[1:0]");
    vcd.root_scope()
        .add_var("position", "p", VarType::Integer, 32);
    vcd.root_scope().add_var("error", "e", VarType::Event, 1);
    vcd.root_scope().add_var("proj", "l", VarType::Integer, 32);

    println!("{}", vcd.to_string());

    let c = vcd.root_scope_ref().get_var("state").unwrap();
    let p = vcd.root_scope_ref().get_var("position").unwrap();
    // let btn = vcd.root_scope_ref().get_var("btn").unwrap();
    let er = vcd.root_scope_ref().get_var("error").unwrap();
    let lp = vcd.root_scope_ref().get_var("proj").unwrap();

    use embassy_futures::select::{select, Either};

    // print starting conditions (dumpvars)
    println!("{}", VcdFile::print_timestamp(t0, u, s));
    println!("{}", &c.print_data(rot.raw_state() as u32));
    println!("{}", &p.print_data(rot.position() as u32));
    // println!("{}", &btn.print_data(0));
    println!("{}", &er.print_data(0));

    // store the time instant of the last N position updates
    let mut last_direction = RotaryMovement::Clockwise;
    let mut last_positions: Vec<(Instant, i32)> = Vec::new();

    fn linear_speed_avg(last_positions: &Vec<(Instant, i32)>) -> f32 {
        // calculate deltas between each position update
        // for all positions within the past 100 ms elapsed
        let mut deltas: i32 = 0;
        let mut xs: u32 = 0;

        let (t0, p0) = last_positions[0];
        let (t1, p1) = last_positions[last_positions.len() - 1];
        let dt = t1.duration_since(t0).as_micros();
        if dt < 100 {
            // if its a huge delta, dont bother
            if (p1 - p0).abs() < 0xFFFF {
                deltas += p1 - p0;
                xs += dt as u32;
            }
        }

        // calculate the average speed per ms
        if xs > 0 {
            deltas as f32 / xs as f32
        } else {
            0.0
        }
    }

    // using the speed, and the time since the last update, we can estimate the position
    // this is useful for the case where the encoder is spinning too fast for the polling rate
    fn linear_position_projection(
        last_positions: &Vec<(Instant, i32)>,
    ) -> Result<i32, TimeoutError> {
        if last_positions.len() < 2 {
            return Err(TimeoutError);
        }
        let avg_speed = linear_speed_avg(last_positions);
        let (t0, p0) = last_positions[last_positions.len() - 1];
        let dt = t0.elapsed().as_micros();
        Ok(p0 + (avg_speed * dt as f32) as i32)
    }

    loop {
        match select(rot.poll(), rot_sw.wait_for_any_edge()).await {
            Either::First(movement) => match movement {
                Ok(Some(movement)) => {
                    println!("{}", VcdFile::print_timestamp(t0, u, s));
                    match movement {
                        RotaryMovement::Clockwise => {
                            esp_println::println!(
                                "$comment rot CW position {} $end",
                                rot.position()
                            );
                            println!("{}", &p.print_data(rot.position() as u32));
                        }
                        RotaryMovement::CounterClockwise => {
                            esp_println::println!(
                                "$comment rot C-CW position {} $end",
                                rot.position()
                            );
                            println!("{}", &p.print_data(rot.position() as u32));
                        }
                    }

                    if movement != last_direction {
                        last_direction = movement;
                        last_positions.clear();
                    }

                    let statebyte = rot.raw_state();
                    println!("{}", &c.print_data((statebyte & 0b11) as u32));

                    // print the linearly projected position
                    match linear_position_projection(&last_positions) {
                        Ok(proj) => {
                            println!("{}", &lp.print_data(proj as u32));
                        }
                        Err(_) => {}
                    };

                    // store the time instant of the last N position updates
                    last_positions.push((Instant::now(), rot.position()));
                    if last_positions.len() > 10 {
                        last_positions.remove(0);
                    }
                }
                Ok(_) => {
                    println!("{}", VcdFile::print_timestamp(t0, u, s));
                    let statebyte = rot.raw_state();
                    println!("{}", &c.print_data((statebyte & 0b11) as u32));
                }
                Err(e) => {
                    esp_println::println!("$comment error {:?} $end", e);
                    println!("{}", &er.print_data(1));
                }
            },
            Either::Second(_) => {
                // read level and print
                // let level = rot_sw.level();
                // println!("{}", VcdFile::print_timestamp(t0, u, s));
                // println!("{}", &btn.print_data(level as u32));
            }
        }
    }
}

#[embassy_executor::task]
async fn busy_task() {
    let mut ticker = Ticker::every(Duration::from_millis(500));

    loop {
        esp_println::println!("$comment blocking... end");
        block_on(Timer::after_millis(1));
        ticker.next().await;
    }
}

#[main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger(log::LevelFilter::Info);

    // print a line that signals where to cut the stdout data for the VCD FILE
    println!("--- VCD FILE START ---");

    //init_heap();
    //let peripherals = Peripherals::take();
    //let system = SystemControl::new(peripherals.SYSTEM);
    //let clocks = ClockControl::configure(system.clock_control,CpuClock::Clock80MHz).freeze();
    let peripherals = esp_hal::init(esp_hal::Config::default());
    esp_alloc::heap_allocator!(72 * 1024);

    //let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    //esp_hal_embassy::init(&clocks, timers);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // // setup rotary encoder
    let rot_clk = Flex::new(peripherals.GPIO5);
    let rot_dat = Flex::new(peripherals.GPIO4);
    let rot_switch = Flex::new(peripherals.GPIO3);
    spawner.must_spawn(encoder_task(rot_clk, rot_dat, rot_switch));

    // add a busy task which wastes time
    spawner.must_spawn(busy_task());
}
