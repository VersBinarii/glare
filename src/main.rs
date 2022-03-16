#![no_main]
#![no_std]

use glare as _;

#[rtic::app(device = stm32f4xx_hal::pac)]
mod app {
    use cortex_m::singleton;
    use glare::{
        command::CwModeQuery,
        driver::{Esp01, MAX_RESP_LEN},
    };
    use heapless::{
        spsc::{Consumer, Producer, Queue},
        String,
    };
    use stm32f4xx_hal::{
        pac::USART1,
        prelude::*,
        serial::{config::Config, Event, Serial},
        timer::SysDelay,
    };

    const QUEUE_LEN: usize = 8;

    #[shared]
    struct Shared {
        esp01: Esp01<USART1>,
    }

    #[local]
    struct Local {
        rx_prod: Producer<'static, String<MAX_RESP_LEN>, QUEUE_LEN>,
        rx_con: Consumer<'static, String<MAX_RESP_LEN>, QUEUE_LEN>,
        delay: SysDelay,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let dp = ctx.device;

        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(25.MHz()).freeze();
        let gpioa = dp.GPIOA.split();
        // define RX/TX pins
        let tx_pin = gpioa.pa9.into_alternate();
        let rx_pin = gpioa.pa10.into_alternate();

        // configure serial
        let mut serial: Serial<USART1, _, u8> = Serial::new(
            dp.USART1,
            (tx_pin, rx_pin),
            Config::default().baudrate(115200.bps()),
            &clocks,
        )
        .unwrap();

        //serial.listen(Event::Rxne);
        serial.listen(Event::Idle);
        // Make this Serial object use u16s instead of u8s

        let esp01 = Esp01::new(serial);

        let rx_queue = singleton!(:Queue<String<MAX_RESP_LEN>, QUEUE_LEN> = Queue::new()).unwrap();

        let (rx_prod, rx_con) = rx_queue.split();
        defmt::println!("Hello from init");

        let delay = ctx.core.SYST.delay(&clocks);
        (
            Shared { esp01 },
            Local {
                rx_con,
                rx_prod,
                delay,
            },
            init::Monotonics(),
        )
    }

    #[idle(shared = [esp01], local = [rx_con, delay])]
    fn idle(mut ctx: idle::Context) -> ! {
        // Locals in idle have lifetime 'static
        let delay = ctx.local.delay;
        defmt::println!("Hello Idle task");

        loop {
            ctx.shared
                .esp01
                .lock(|esp01| match esp01.send_command(CwModeQuery::default()) {
                    Ok(_) => defmt::println!("Success sending"),
                    Err(_) => defmt::println!("Error sending"),
                });
            if let Some(response) = ctx.local.rx_con.dequeue() {
                defmt::println!("Got response: {}", response.as_str());
            }
            delay.delay_ms(1000u32);
        }
    }

    #[task(binds = USART1, shared = [esp01],local = [rx_prod])]
    fn usart1(mut ctx: usart1::Context) {
        let rx_prod = ctx.local.rx_prod;
        ctx.shared.esp01.lock(|esp01| {
            if esp01.is_response_ready() {
                let response = esp01.get_response().unwrap();
                rx_prod.enqueue(response).unwrap();
            }

            esp01.read_byte().unwrap();
        });
    }
}
