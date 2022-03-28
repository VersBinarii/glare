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
        gpio::{Edge, Output, Pin, PushPull},
        i2c::{DutyCycle, I2c1, Mode},
        pac::{GPIOA, TIM2, USART6},
        prelude::*,
        serial::{config::Config, Event, Serial},
        timer::{CounterHz, Event as TimEvent, SysDelay},
    };

    const QUEUE_LEN: usize = 8;

    #[shared]
    struct Shared {
        esp01: Esp01<USART6>,
    }

    #[local]
    struct Local {
        rx_prod: Producer<'static, String<MAX_RESP_LEN>, QUEUE_LEN>,
        rx_con: Consumer<'static, String<MAX_RESP_LEN>, QUEUE_LEN>,
        delay: SysDelay,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut dp = ctx.device;

        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(100.MHz()).freeze();

        let mut syscfg = dp.SYSCFG.constrain();
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();

        // Port A pixel data port
        let _pd0 = gpioa.pa0.into_pull_down_input();
        let _pd1 = gpioa.pa1.into_pull_down_input();
        let _pd2 = gpioa.pa2.into_pull_down_input();
        let _pd3 = gpioa.pa3.into_pull_down_input();
        let _pd4 = gpioa.pa4.into_pull_down_input();
        let _pd5 = gpioa.pa5.into_pull_down_input();
        let _pd6 = gpioa.pa6.into_pull_down_input();
        let _pd7 = gpioa.pa7.into_pull_down_input();
        let _pd8 = gpioa.pa8.into_pull_down_input();
        let _pd9 = gpioa.pa9.into_pull_down_input();

        // Timer2 output CH1
        //
        unsafe {
            let raw_gpioa = &*<GPIOA>::ptr();
            raw_gpioa.afrh.modify(|_, w| w.afrh15().af1());
        }
        let mut timer2 = dp.TIM2.counter_hz(&clocks);
        unsafe {
            let tim_raw = &*<TIM2>::ptr();
            tim_raw.ccmr1_output().modify(|_, w| w.oc1m().toggle());
            tim_raw.ccer.modify(|_, w| w.cc1e().set_bit());
        }
        let _ = timer2.start(6.MHz());
        //timer2.listen(TimEvent::Update);

        // I2c pb6 - SCL pb7 - SDA
        let i2c_scl = gpiob.pb6.into_alternate_open_drain();
        let i2c_sda = gpiob.pb7.into_alternate_open_drain();

        let _cam_i2c = I2c1::new(
            dp.I2C1,
            (i2c_scl, i2c_sda),
            Mode::Fast {
                frequency: 400000.Hz(),
                duty_cycle: DutyCycle::Ratio2to1,
            },
            &clocks,
        );
        // cam control pins
        let mut pclk = gpiob.pb5.into_pull_down_input();
        pclk.make_interrupt_source(&mut syscfg);
        pclk.trigger_on_edge(&mut dp.EXTI, Edge::Falling);
        let mut href = gpiob.pb8.into_pull_down_input();
        href.make_interrupt_source(&mut syscfg);
        href.trigger_on_edge(&mut dp.EXTI, Edge::Falling);
        let mut vsync = gpiob.pb9.into_pull_down_input();
        vsync.make_interrupt_source(&mut syscfg);
        vsync.trigger_on_edge(&mut dp.EXTI, Edge::Falling);

        // define RX/TX pins
        let tx_pin = gpioa.pa11.into_alternate();
        let rx_pin = gpioa.pa12.into_alternate();

        // configure serial
        let mut serial: Serial<USART6, _, u8> = Serial::new(
            dp.USART6,
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

    #[task(binds = EXTI9_5 )]
    fn exti95(_ctx: exti95::Context) {
        defmt::println!("Exti interrupt triggered");
    }
    /*
        #[task(binds = TIM2, local = [tim2_ch1, timer2] )]
        fn tim2(ctx: tim2::Context) {
            let tim = ctx.local.timer2;
            tim.unlisten(TimEvent::Update);
            ctx.local.tim2_ch1.toggle();
            tim.clear_interrupt(TimEvent::Update);
            tim.listen(TimEvent::Update);
        }
    */
}
