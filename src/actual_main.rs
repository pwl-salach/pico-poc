use crate::XTAL_FREQ_HZ;
use crate::controls::{
    ControlCommand, InputDevice, bt_controls::HC05, buttons_controls::ButtonsControls,
};
use crate::messengers::lcd::LcdMessenger;
use crate::messengers::{MessageLevel, broadcaster::Broadcaster, logger::Logger};
use crate::pca9685::{Pca9685, ServoConfig};
use crate::{hal, hal::Clock, hal::fugit::RateExtU32};
use core::fmt::Write;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use heapless::String;

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
    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    #[cfg(rp2350)]
    let timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    const I2C_FREQ_HZ: u32 = 100_000;

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
        I2C_FREQ_HZ.kHz(),
        &mut pac.RESETS,
        clocks.system_clock.freq(),
    );

    let mut servos = [None; 16];
    servos[0] = Some(ServoConfig {
        min_pulse_ms: 0,
        max_pulse_ms: 20_000,
        current_angle: 0.0,
    });
    servos[1] = Some(ServoConfig {
        min_pulse_ms: 1000,
        max_pulse_ms: 2000,
        current_angle: 0.0,
    });

    let mut pca9685 = Pca9685::new_default(i2c, servos);
    pca9685.init().unwrap();

    let uart_pins = (
        pins.gpio0.into_function::<hal::gpio::FunctionUart>(),
        pins.gpio1.into_function::<hal::gpio::FunctionUart>(),
    );
    // let uart = hal::uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
    //     .enable(
    //         UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
    //         clocks.peripheral_clock.freq(),
    //     )
    //     .unwrap();
    // let hc_05 = HC05::new(uart);

    // let mut pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);
    // let pwm = &mut pwm_slices.pwm2;
    // pwm.set_ph_correct();
    // pwm.enable();
    // let pwm_pin = pins.gpio21.into_function::<hal::gpio::FunctionPwm>();
    // let channel = &mut pwm.channel_b;
    // channel.output_to(pwm_pin);
    let buttons_contr = ButtonsControls::new(
        pins.gpio6.into_pull_up_input(),
        pins.gpio7.into_pull_up_input(),
        pins.gpio8.into_pull_up_input(),
        pins.gpio9.into_pull_up_input(),
        pins.gpio10.into_pull_up_input(),
        pins.gpio11.into_pull_up_input(),
        pins.gpio12.into_pull_up_input(),
    );

    let sda = pins
        .gpio18
        .reconfigure::<hal::gpio::FunctionI2C, hal::gpio::PullUp>();
    let scl = pins
        .gpio19
        .reconfigure::<hal::gpio::FunctionI2C, hal::gpio::PullUp>();

    let i2c = hal::i2c::I2C::i2c1(
        pac.I2C1,
        sda,
        scl,
        I2C_FREQ_HZ.kHz(),
        &mut pac.RESETS,
        clocks.system_clock.freq(),
    );

    let mut broadcaster = Broadcaster::new();
    let mut logger = Logger;
    let _ = broadcaster.add_messenger(&mut logger); // This is the 1st messenger, so it should never fail

    let mut lcd = LcdMessenger::new(i2c, timer);
    if let Err(e) = broadcaster.add_messenger(&mut lcd) {
        broadcaster.broadcast(e, MessageLevel::Error);
    }

    program_loop(timer, pca9685, buttons_contr, broadcaster);
}

fn program_loop<I2C>(
    mut timer: hal::Timer,
    mut pca9685: Pca9685<I2C>,
    mut input_device: impl InputDevice,
    mut broadcaster: Broadcaster,
    // uart: hal::uart::UartPeripheral<hal::uart::Enabled, D, P>,
    // mut channel: &mut hal::pwm::Channel<
    //     hal::pwm::Slice<hal::pwm::Pwm2, hal::pwm::FreeRunning>,
    //     hal::pwm::B,
    // >,
) -> !
where
    I2C: embedded_hal::i2c::I2c,
{
    loop {
        let input = input_device.read_input().unwrap();
        match input {
            ControlCommand::Servo(cmd) => {
                match pca9685.move_servo_by_step(cmd.servo_index, cmd.step) {
                    Ok(_) => {
                        let mut message: String<32> = String::new();
                        if write!(
                            &mut message,
                            "Moved servo {} by step {}",
                            cmd.servo_index, cmd.step
                        )
                        .is_ok()
                        {
                            broadcaster.broadcast(message.as_str(), MessageLevel::Debug);
                        } else {
                            broadcaster.broadcast(
                                "Moved servo, but failed to format log message",
                                MessageLevel::Warning,
                            );
                        }
                    }
                    Err(e) => broadcaster.broadcast(&e.message(), MessageLevel::Error),
                }
            }
            ControlCommand::Effector(cmd) => {
                // Handle effector command
            }
            ControlCommand::Config(cmd) => {
                // Handle configuration command
            }
        }
        pca9685.update_all_servos().unwrap();
        timer.delay_ms(100);
    }
}
