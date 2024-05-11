#![no_std]
#![no_main]

extern crate k1921vk035_pac;

use core::cell::RefCell;

use cortex_m::{
    asm::nop,
    interrupt::Mutex,
};
use cortex_m_rt::{self, entry};
use k1921vk035_pac as pac;
use pac::{interrupt, Peripherals};
use panic_halt as _;

static G_PER: Mutex<RefCell<Option<Peripherals>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let dp: Peripherals = Peripherals::take().unwrap();

    config_led_pin(&dp);

    config_interrupt_user_btn(&dp);

    //check via led if pin6 isn't jtag
    if dp.gpioa.altfuncset().read().pin6().bit_is_clear() {
        toggle_led(&dp);
        toggle_led(&dp);
    }

    unsafe { cortex_m::peripheral::NVIC::unmask(interrupt::GPIOA) };
    
    //check via led if nvic is enabled
    if cortex_m::peripheral::NVIC::is_enabled(interrupt::GPIOA) {
        toggle_led(&dp);
        toggle_led(&dp);
        toggle_led(&dp);
    }

    cortex_m::interrupt::free(|cs| {
        G_PER.borrow(cs).replace(Some(dp));
    });

    loop {}
}

fn toggle_led(periph: &Peripherals) {
    periph.gpiob.dataoutclr().write(|w| w.pin5().set_bit());
    delay();

    periph.gpiob.dataoutset().write(|w| w.pin5().set_bit());
    delay();
}

fn config_led_pin(periph: &Peripherals) {
    periph.rcu.hclkcfg().modify(|_r, w| w.gpioben().set_bit());
    periph.rcu.hrstcfg().modify(|_r, w| w.gpioben().set_bit());

    periph.gpiob.dataoutset().write(|w| w.pin5().set_bit());
    periph.gpiob.outmode().write(|w| w.pin5().pp());
    periph.gpiob.outenset().write(|w| w.pin5().set_bit());
    periph.gpiob.denset().write(|w| w.pin5().set_bit());

    periph.gpiob.drivemode().write(|w| w.pin5().hf());
}

fn delay() {
    for _n in 0..20_000 {
        nop();
    }
}

fn config_interrupt_user_btn(periph: &Peripherals) {
    periph.rcu.hclkcfg().modify(|_r, w| w.gpioaen().set_bit());
    periph.rcu.hrstcfg().modify(|_r, w| w.gpioaen().set_bit());

    periph
        .gpioa
        .lockkey()
        .write(|w| unsafe { w.bits(0xADEADBEE) });
    // delay for wait unlock gpioa pin6
    for _n in 0..1000 {
        nop();
    }
    periph.gpioa.lockclr().write(|w| w.pin6().set_bit());
    for _n in 0..1000 {
        nop();
    }

    periph.gpioa.altfuncclr().write(|w| w.pin6().set_bit());
    periph.gpioa.inmode().write(|w| w.pin6().cmos());
    periph.gpioa.denset().write(|w| w.pin6().set_bit());

    periph.gpioa.inttypeset().write(|w| w.pin6().set_bit());
    periph.gpioa.intpolset().write(|w| w.pin6().set_bit());
    periph.gpioa.intedgeset().write(|w| w.pin6().set_bit());
    periph.gpioa.intenset().write(|w| w.pin6().set_bit());
}

#[interrupt]
fn GPIOA() {
    // Start a Critical Section
    cortex_m::interrupt::free(|cs| {
        // Obtain Access to Peripherals Global Data
        let mut dp = G_PER.borrow(cs).borrow_mut();
        // Check if PA6 caused the interrupt
        if dp.as_mut().unwrap().gpioa.intstatus().read().pin6().bit() {
            // Toggle Output LED
            dp.as_mut()
                .unwrap()
                .gpioa
                .intstatus()
                .write(|w| w.pin6().set_bit());
            // Clear Interrupt Flag for Button
            dp.as_mut()
                .unwrap()
                .gpiob
                .dataouttgl()
                .write(|w| w.pin5().set_bit());
        }
    });
}
