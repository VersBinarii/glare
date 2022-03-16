#![no_main]
#![no_std]

use glare as _;

#[rtic::app(device = stm32f4xx_hal::pac)]
mod app {
    use cortex_m::singleton;
    use glare::driver::Esp01;
    use heapless::spsc::{Consumer, Producer, Queue};
    use stm32f4xx_hal::{
        pac::USART1,
        prelude::*,
        serial::{config::Config, Event, Rx, Serial, Tx},
    };

    const QUEUE_LEN: usize = 8;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        esp01: Esp01<USART1>,
        tx_prod: Producer<'static, u8, QUEUE_LEN>,
        tx_con: Consumer<'static, u8, QUEUE_LEN>,
        rx_prod: Producer<'static, u8, QUEUE_LEN>,
        rx_con: Consumer<'static, u8, QUEUE_LEN>,
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
        let serial: Serial<USART1, _, u8> = Serial::new(
            dp.USART1,
            (tx_pin, rx_pin),
            Config::default().baudrate(115200.bps()),
            &clocks,
        )
        .unwrap();

        serial.listen(Event::Rxne);
        serial.listen(Event::Idle);
        // Make this Serial object use u16s instead of u8s

        let esp01 = Esp01::new(serial);

        let rx_queue = singleton!(:Queue<u8, QUEUE_LEN> = Queue::new()).unwrap();
        let tx_queue = singleton!(:Queue<u8, QUEUE_LEN> = Queue::new()).unwrap();

        let (rx_prod, rx_con) = rx_queue.split();
        let (tx_prod, tx_con) = tx_queue.split();
        defmt::println!("Hello from init");

        (
            Shared {},
            Local {
                tx_con,
                tx_prod,
                rx_con,
                rx_prod,
            },
            init::Monotonics(),
        )
    }

    #[idle(local = [rx_con, tx_prod])]
    fn idle(cx: idle::Context) -> ! {
        // Locals in idle have lifetime 'static

        defmt::println!("Hello Idle task");

        loop {
            if let Some(byte) = cx.local.rx_con.dequeue() {
                defmt::println!("Got byte: {}", byte);
                let _ = cx.local.tx_prod.enqueue(byte);
            }
        }
    }

    #[task(binds = USART1, local = [tx, rx, rx_prod, tx_con])]
    fn usart1(ctx: usart1::Context) {
        let tx = ctx.local.tx;
        let rx = ctx.local.rx;

        rx.unlisten();

        if tx.is_tx_empty() {
            defmt::println!("TX interrupt");
            if let Some(byte) = ctx.local.tx_con.dequeue() {
                let _ = tx.write(byte);
            }
            tx.unlisten();
        }

        if rx.is_rx_not_empty() {
            let byte = rx.read().unwrap();

            let _ = ctx.local.rx_prod.enqueue(byte);
            tx.listen();
        }

        rx.listen();
    }
}
