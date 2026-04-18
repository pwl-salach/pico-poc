use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

#[cfg(rp2350)]
use rp235x_hal as hal;
#[cfg(rp2350)]
use rp235x_hal::Clock;
#[cfg(rp2350)]
use rp235x_hal::fugit::RateExtU32;
#[cfg(rp2350)]
use rp235x_hal::uart::{DataBits, StopBits, UartConfig};

#[cfg(rp2040)]
use rp2040_hal as hal;
#[cfg(rp2040)]
use rp2040_hal::Clock;
#[cfg(rp2040)]
use rp2040_hal::fugit::RateExtU32;
#[cfg(rp2040)]
use rp2040_hal::uart::{DataBits, StopBits, UartConfig};

use crate::XTAL_FREQ_HZ;
use crate::pca9685::{Pca9685, ServoConfig};

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
        100.kHz(),
        &mut pac.RESETS,
        clocks.system_clock.freq(),
    );

    let mut servos = [None; 16];
    servos[1] = Some(ServoConfig {
        min_pulse: 1000,
        max_pulse: 2000,
        current_angle: 0.0,
    });

    let mut pca9685 = Pca9685::new_default(i2c, servos);
    pca9685.init().unwrap();

    let uart_pins = (
        pins.gpio0.into_function::<hal::gpio::FunctionUart>(),
        pins.gpio1.into_function::<hal::gpio::FunctionUart>(),
    );
    let uart = hal::uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();
    // let mut pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);
    // let pwm = &mut pwm_slices.pwm2;
    // pwm.set_ph_correct();
    // pwm.enable();
    // let pwm_pin = pins.gpio21.into_function::<hal::gpio::FunctionPwm>();
    // let channel = &mut pwm.channel_b;
    // channel.output_to(pwm_pin);

    program_loop(timer, led_pin, pca9685, uart);
}

fn program_loop<I2C, D, P>(
    mut timer: hal::Timer,
    mut led_pin: impl OutputPin,
    mut pca9685: Pca9685<I2C>,
    mut uart: hal::uart::UartPeripheral<hal::uart::Enabled, D, P>,
    // mut channel: &mut hal::pwm::Channel<
    //     hal::pwm::Slice<hal::pwm::Pwm2, hal::pwm::FreeRunning>,
    //     hal::pwm::B,
    // >,
) -> !
where
    I2C: embedded_hal::i2c::I2c,
    D: hal::uart::UartDevice,
    P: hal::uart::ValidUartPinout<D>,
{
    // const LOW: u16 = 0;
    // const HIGH: u16 = 25000;
    let mut buffer = [0u8; 32];

    loop {
        // Animate LED0 as before
        defmt::info!("on!");
        led_pin.set_high().unwrap();
        // pca9685.set_servo_angle(1, 45.0, 1000, 2000).unwrap();
        pca9685.set_pwm(0, 0, 200).unwrap();
        pca9685.set_pwm(1, 0, 200).unwrap();
        // timer.delay_ms(50);
        // for i in (LOW..=HIGH).step_by(25) {
        //     timer.delay_us(500);
        //     channel.set_duty(i);
        // }
        timer.delay_ms(500);

        defmt::info!("off!");
        led_pin.set_low().unwrap();
        // pca9685.set_servo_angle(1, 90.0, 1000, 2000).unwrap();
        pca9685.set_pwm(0, 0, 400).unwrap();
        pca9685.set_pwm(1, 0, 400).unwrap();
        // Ramp brightness down
        // for i in (LOW..=HIGH).rev().step_by(25) {
        //     timer.delay_us(50);
        //     channel.set_duty(i);
        // }
        timer.delay_ms(500);
        while let Ok(n) = uart.read_raw(&mut buffer) {
            // echo exactly what we got
            uart.write_full_blocking(&buffer[..n]);
        }
    }
}
