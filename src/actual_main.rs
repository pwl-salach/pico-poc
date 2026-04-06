use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

#[cfg(rp2350)]
use rp235x_hal as hal;
#[cfg(rp2350)]
use rp235x_hal::fugit::RateExtU32;

#[cfg(rp2040)]
use rp2040_hal as hal;
#[cfg(rp2040)]
use rp2040_hal::fugit::RateExtU32;

use crate::XTAL_FREQ_HZ;
use crate::pca9685::Pca9685;

pub fn main(mut pac: hal::pac::Peripherals) -> ! {
    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    #[cfg(rp2040)]
    let mut timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    #[cfg(rp2350)]
    let mut timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure GPIO25 as an output
    let led_pin = pins.gpio25.into_push_pull_output();

    let sda = pins
        .gpio4
        .reconfigure::<hal::gpio::FunctionI2C, hal::gpio::PullUp>();
    let scl = pins
        .gpio5
        .reconfigure::<hal::gpio::FunctionI2C, hal::gpio::PullUp>();

    let i2c = hal::i2c::I2C::i2c0(
        pac.I2C0,
        sda,
        scl,
        100.kHz(), // 100 kHz
        &mut pac.RESETS,
        25.MHz(), // system clock
    );

    let mut pca9685 = Pca9685::new(i2c);
    pca9685.init().unwrap();
    pca9685.set_pwm_freq(50.0).unwrap();
    pca9685.set_servo_angle(1, 45.0, 1000, 2000).unwrap();

    program_loop(timer, led_pin);
}

fn program_loop(mut timer: hal::Timer, mut led_pin: impl OutputPin) -> ! {
    loop {
        // Animate LED0 as before
        defmt::info!("on!");
        led_pin.set_high().unwrap();
        timer.delay_ms(500);
        defmt::info!("off!");
        led_pin.set_low().unwrap();
        timer.delay_ms(500);
    }
}
