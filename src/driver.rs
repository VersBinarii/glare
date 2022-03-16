use crate::command::AtCommand;
use heapless::{String, Vec};
use stm32f4xx_hal::serial::{Instance, Rx, Serial, Tx};

const MAX_RESP_LEN: usize = 256;

pub struct Esp01<USART> {
    tx: Tx<USART>,
    rx: Rx<USART>,
    rx_buf: Vec<u8, MAX_RESP_LEN>,
}

impl<USART> Esp01<USART> {
    pub fn new<PINS>(serial: Serial<USART, PINS, u8>) -> Self
    where
        USART: Instance,
    {
        let (tx, rx) = serial.split();

        Self {
            tx,
            rx,
            rx_buf: Vec::new(),
        }
    }

    pub fn send_command<'a, CMD: AtCommand<'a>>(cmd: CMD) -> Result<(), &'static str> {
        Ok(())
    }

    pub fn read_byte() -> Result<(), &'static str> {
        todo!()
    }

    pub fn get_response() -> Result<String<MAX_RESP_LEN>, &'static str> {
        todo!()
    }
}
