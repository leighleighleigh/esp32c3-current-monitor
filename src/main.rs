//! embassy hello world
//!
//! This is an example of running the embassy executor with multiple tasks
//! concurrently.

//% CHIPS: esp32 esp32c2 esp32c3 esp32c6 esp32h2 esp32s2 esp32s3
//% FEATURES: embassy embassy-time-timg0 embassy-generic-timers

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(const_trait_impl)]

const INA3221_I2C_ADDR: u8 = 0x41;
// const SHUNT_RESISTANCE: f32 = 0.1f32; // 0.1 Ohm

extern crate alloc;
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use core::cell::RefCell;
use embedded_hal_bus::i2c;

use ina3221::{AveragingMode, Current, Voltage, INA3221};

use embedded_graphics::{
    geometry::AnchorY, image::Image, mock_display::ColorMapping, mono_font::{
        ascii::{FONT_10X20, FONT_4X6, FONT_5X7, FONT_6X10, FONT_6X13_BOLD, FONT_7X13, FONT_9X15_BOLD, FONT_9X18_BOLD},
        MonoTextStyleBuilder,
    }, pixelcolor::{BinaryColor, Rgb565}, prelude::*, primitives::Rectangle, text::{Alignment, Baseline, LineHeight, Text, TextStyleBuilder}
};

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Ticker, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal as hal;

use hal::{
    analog::adc::{Adc, AdcConfig, AdcPin, Attenuation},
    clock::{ClockControl,CpuClock},
    gpio::{AnalogPin, AnyInput, Io},
    i2c::I2C,
    peripherals::{Peripherals, I2C0},
    prelude::*,
    system::SystemControl,
    timer::timg::TimerGroup,
    timer::systimer::{SystemTimer, Target},
};

use core::mem::MaybeUninit;

use ssd1306_i2c::{prelude::*, Builder}; // was use sh1106:: ...

//#[global_allocator]
//static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();
// We actually have a heap! This is used for string formatting
//fn init_heap() {
//    const HEAP_SIZE: usize = 32 * 1024;
//    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();
//
//    unsafe {
//        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
//    }
//}

#[main]
async fn main(spawner: Spawner) {
    esp_println::println!("Init!");
    esp_println::logger::init_logger(log::LevelFilter::Info);

    //init_heap();
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::configure(system.clock_control,CpuClock::Clock80MHz).freeze();

    esp_alloc::heap_allocator!(72 * 1024);
  
    //let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    //esp_hal_embassy::init(&clocks, timers);
    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    esp_hal_embassy::init(&clocks, timg0.timer0);

    //let systimer = SystemTimer::new(peripherals.SYSTIMER).split::<Target>();
    //esp_hal_embassy::init(&clocks, systimer.alarm0);

    // hello world printer
    // spawner.spawn(run()).ok();
    let mut ticker = Ticker::every(Duration::from_millis(100));

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // get a random inptu to use for adc
    let analog_pin = io.pins.gpio3;
    let mut adc1_config = AdcConfig::new();
    let mut adc1_pin = adc1_config.enable_pin(analog_pin, Attenuation::Attenuation11dB);
    let mut adc1 = Adc::new(peripherals.ADC1, adc1_config);

    // setup boot gpio0 pin as input
    let mode_pin = AnyInput::new(io.pins.gpio9, hal::gpio::Pull::None);

    let i2c0 = I2C::new(
        peripherals.I2C0,
        io.pins.gpio6,
        io.pins.gpio7,
        400.kHz(),
        &clocks,
    );

    let i2c_ref_cell = RefCell::new(i2c0);

    // create and ssd1306-i2c instance using builder
    let mut display: GraphicsMode<_> = Builder::new()
        .with_size(DisplaySize::Display128x64NoOffset)
        .with_i2c_addr(0x3c) // your LCD may used 0x3c the primary address
        .with_rotation(DisplayRotation::Rotate0)
        .connect_i2c(i2c::RefCellDevice::new(&i2c_ref_cell))
        .into();

    let text_style_title = MonoTextStyleBuilder::new()
        .font(&FONT_9X18_BOLD)
        .text_color(BinaryColor::Off)
        .build();

    let text_style = TextStyleBuilder::new()
        .alignment(Alignment::Center)
        .line_height(LineHeight::Percent(75))
        .baseline(Baseline::Middle)
        .build();

    let busVoltageTextStyle = TextStyleBuilder::new()
        .alignment(Alignment::Left)
        .line_height(LineHeight::Percent(75))
        .baseline(Baseline::Top)
        .build();

    let clampCurrentTextStyle = TextStyleBuilder::new()
        .alignment(Alignment::Left)
        .line_height(LineHeight::Percent(75))
        .baseline(Baseline::Bottom)
        .build();

    let text_style_small = MonoTextStyleBuilder::new()
        .font(&FONT_9X15_BOLD)
        .text_color(BinaryColor::On)
        .build();

    let text_style_nano = MonoTextStyleBuilder::new()
        .font(&FONT_5X7)
        .text_color(BinaryColor::On)
        .build();

    display.init().unwrap();
    display.flush().unwrap();
    display.clear();
    display.set_contrast(1).unwrap();

    ticker.next().await;

    // make some text
    let title_text = " TRANS\nRIGHTS!\n  :3";

    let y_offset = -18;
    let x_offset = -48;
    let letter_spacing = 11;
    let line_spacing = 15;

    let mut x = 0;
    let mut y = 0;

    let mut wiggle_seed = 7;

    let loop_count = 20;

    // print the text with a 'typewriter' effect
    // each character will shake a little bit independently of each other
    for i in 0..loop_count {
        let k = i.min(title_text.len());

        display.clear();
        display
            .fill_solid(&Rectangle::from(display.bounding_box()), BinaryColor::On)
            .unwrap();
        x = x_offset;
        y = y_offset;

        for l in 0..k {
            // if the character is '\n' then add  to the y position
            if title_text[l..l + 1].to_string() == "\n" {
                y += line_spacing;
                x = x_offset;
                continue;
            }

            // add letter spacing
            x += letter_spacing;

            // x and y wiggle is based on a modulo of the character index, so each character will wiggle independently
            let mut letter_x = x;
            let mut letter_y = y;

            let letter_seed: i32 = (l as i32) * wiggle_seed;

            // if it's odd, then make it negative
            letter_x = match (letter_seed % 5) {
                0 => letter_x,
                1 => letter_x - 1,
                _ => letter_x + 1,
            };

            letter_y = match (letter_seed % 7) {
                0 => letter_y,
                1 => letter_y - 1,
                _ => letter_y + 1,
            };

            Text::with_text_style(
                &title_text[l..l + 1],
                display.bounding_box().center() + Point::new(letter_x, letter_y),
                text_style_title,
                text_style,
            )
            .draw(&mut display)
            .unwrap();

            // Update wiggle seed
            let pin_value: u16 = nb::block!(adc1.read_oneshot(&mut adc1_pin)).unwrap();
            wiggle_seed += pin_value as i32;
        }

        display.flush().unwrap();

        // Text::with_text_style(&title_text[..i + 1],(display.bounding_box().center() + Point::new(0,-16)), text_style_title, text_style).draw(&mut display).unwrap();
        // ticker.next().await;
        ticker.next().await;
    }

    Timer::after_millis(100).await;

    // Current monitor
    let mut ina = INA3221::new(i2c::RefCellDevice::new(&i2c_ref_cell), INA3221_I2C_ADDR);
    Timer::after_millis(100).await;
    ina.set_averaging_mode(ina3221::AveragingMode::Samples4).unwrap();
    Timer::after_millis(100).await;
    let channel = 0;

    // data channels, rolling bar graph
    let mut busvolt: Vec<i32> = vec![0; 128];
    let mut shuntvolt: Vec<i32> = vec![0; 128];

    let mut max_busvolt = 0;
    let mut max_shuntvolt = 0;
    let mut min_busvolt = 0;
    let mut min_shuntvolt = 0;

    let mut avg_mode = AveragingMode::Samples1;

    loop {
        Timer::after_millis(100).await;

        // if button is pressed, clear the vecs
        if mode_pin.is_low() {
            busvolt = vec![0; 128];
            shuntvolt = vec![0; 128];
            // change sampling mode
            avg_mode = next_mode(avg_mode);
            ina.set_averaging_mode(avg_mode).unwrap();
            Timer::after_millis(500).await;
        }

        let sampling_time = Instant::now().as_micros();
        let shunt_voltage = ina.get_shunt_voltage(channel);
        let bus_voltage = ina.get_bus_voltage(channel);

        match (shunt_voltage, bus_voltage) {
            (Ok(shunt_voltage), Ok(bus_voltage)) => {
                esp_println::println!(
                    "{} us, {} mV, {} mV",
                    sampling_time,
                    bus_voltage.milli_volts(),
                    shunt_voltage.milli_volts()
                );


                // append to the rolling bar graph
                busvolt.remove(0);
                busvolt.push(bus_voltage.micro_volts());
                shuntvolt.remove(0);
                shuntvolt.push(shunt_voltage.micro_volts());

                // update our min/max bounds
                max_busvolt = busvolt.iter().cloned().fold(0, i32::max);
                max_shuntvolt = shuntvolt.iter().cloned().fold(0, i32::max);
                min_busvolt = busvolt.iter().cloned().fold(i32::MAX, i32::min);
                min_shuntvolt = shuntvolt.iter().cloned().fold(i32::MAX, i32::min);

                display.clear();
                // display.fill_solid(&Rectangle::from(display.bounding_box()), BinaryColor::On).unwrap();

                // two bar graphs on the screen, both 128 px wide, and each 32px tall.
                // each datapoint gets a single 1px column.
                let bar_height = 32;
                let padding = 1;
                let mut voltage_bar_data = busvolt.clone();
                let mut shunt_bar_data = shuntvolt.clone();
                let mut busdiv = max_busvolt - min_busvolt;
                let mut shuntdiv = max_shuntvolt - min_shuntvolt;

                if busdiv == 0 {
                    busdiv = 1;
                }
                if shuntdiv == 0 {
                    shuntdiv = 1;
                }

                // normalise
                for i in 0..128 {
                    voltage_bar_data[i] = ((voltage_bar_data[i] - min_busvolt) * bar_height
                        / busdiv)
                        .max(padding)
                        .min(bar_height - padding);
                    shunt_bar_data[i] = ((shunt_bar_data[i] - min_shuntvolt) * bar_height
                        / shuntdiv)
                        .max(padding)
                        .min(bar_height - padding);
                }

                //draw lines
                for i in 0..128 {
                    for height in 0..voltage_bar_data[i] {
                        display.set_pixel(i as u32, 32 - height as u32, 1);
                    }
                    for height in 0..shunt_bar_data[i] {
                        display.set_pixel(i as u32, 64 - height as u32, 1);
                    }
                    // display.set_pixel(i as u32, 32 - voltage_bar_data[i] as u32, 1);
                    // display.set_pixel(i as u32, 64 - shunt_bar_data[i] as u32, 1);
                }

                let leftPad = 8; 
                let textPad = 0; 

                let fmtBusText = format!(
                    "{:.0}mV",
                    bus_voltage.milli_volts(),
                );
                let busVtext = Text::with_text_style(
                    &fmtBusText,
                    Point::new(leftPad, textPad),
                    text_style_small,
                    busVoltageTextStyle,
                );
                display.fill_solid(&busVtext.bounding_box(), BinaryColor::Off).unwrap();
                busVtext.draw(&mut display).unwrap();

                let max_busvolt_mv = max_busvolt / 1000;
                let min_busvolt_mv = min_busvolt / 1000;

                let shunt_mA = shunt_voltage.micro_volts() as f64 / 100.0;
                let max_shunt_ma = max_shuntvolt as f64 / 100.0;
                let min_shunt_ma = min_shuntvolt as f64 / 100.0;

                // show max/min voltage next to the realtime one, with half heights, in a column
                let fmtMaxBusText = format!(
                    "{:.0}",
                    max_busvolt_mv,
                );
                let maxBusVtext = Text::with_text_style(
                    &fmtMaxBusText,
                    Point::new(leftPad,busVtext.bounding_box().anchor_y(AnchorY::Bottom) + textPad),
                    text_style_nano,
                    busVoltageTextStyle,
                );
                display.fill_solid(&maxBusVtext.bounding_box(), BinaryColor::Off).unwrap();
                maxBusVtext.draw(&mut display).unwrap();

                // and min
                let fmtMinBusText = format!(
                    "{:.0}",
                    min_busvolt_mv,
                );
                let minBusVtext = Text::with_text_style(
                    &fmtMinBusText,
                    Point::new(leftPad,maxBusVtext.bounding_box().anchor_y(AnchorY::Bottom) + textPad),
                    text_style_nano,
                    busVoltageTextStyle,
                );
                display.fill_solid(&minBusVtext.bounding_box(), BinaryColor::Off).unwrap();
                minBusVtext.draw(&mut display).unwrap();




                let fmtShuntText = format!(
                    "{:.1}mA",
                    shunt_mA,
                );
                let shuntVtext = Text::with_text_style(
                    &fmtShuntText,
                    Point::new(leftPad, 64-textPad),
                    text_style_small,
                    clampCurrentTextStyle,
                );
                display.fill_solid(&shuntVtext.bounding_box(), BinaryColor::Off).unwrap();
                shuntVtext.draw(&mut display).unwrap();

                // repeat min/max for shunt
                let fmtMinShuntText = format!(
                    "{:.0}",
                    min_shunt_ma,
                );
                let minShuntVtext = Text::with_text_style(
                    &fmtMinShuntText,
                    Point::new(leftPad, shuntVtext.bounding_box().anchor_y(AnchorY::Top)-textPad),
                    text_style_nano,
                    clampCurrentTextStyle,
                );
                display.fill_solid(&minShuntVtext.bounding_box(), BinaryColor::Off).unwrap();
                minShuntVtext.draw(&mut display).unwrap();

                let fmtMaxShuntText = format!(
                    "{:.0}",
                    max_shunt_ma,
                );
                let maxShuntVtext = Text::with_text_style(
                    &fmtMaxShuntText,
                    Point::new(leftPad, minShuntVtext.bounding_box().anchor_y(AnchorY::Top)-textPad),
                    text_style_nano,
                    clampCurrentTextStyle,
                );
                display.fill_solid(&maxShuntVtext.bounding_box(), BinaryColor::Off).unwrap();
                maxShuntVtext.draw(&mut display).unwrap();

                display.flush().unwrap();

            }
            (Err(e), _) => {
                log::error!("Error reading shunt voltage: {:?}", e);
                continue;
            }
            (_, Err(e)) => {
                log::error!("Error reading bus voltage: {:?}", e);
                continue;
            }
        }
    }
}

fn next_mode(mode : AveragingMode) -> AveragingMode {
    match mode {
        AveragingMode::Samples1 => AveragingMode::Samples4,
        AveragingMode::Samples4 => AveragingMode::Samples16,
        AveragingMode::Samples16 => AveragingMode::Samples64,
        AveragingMode::Samples64 => AveragingMode::Samples128,
        AveragingMode::Samples128 => AveragingMode::Samples256,
        AveragingMode::Samples256 => AveragingMode::Samples512,
        AveragingMode::Samples512 => AveragingMode::Samples1024,
        AveragingMode::Samples1024 => AveragingMode::Samples1,
    }
}
