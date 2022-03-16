use crate::command::AtCommand;
use core::fmt::Write;
use cortex_m::prelude::_embedded_hal_serial_Read;
use heapless::{String, Vec};
use stm32f4xx_hal::serial::{Instance, Rx, Serial, Tx};

pub const MAX_RESP_LEN: usize = 256;

pub struct Esp01<USART> {
    tx: Tx<USART>,
    rx: Rx<USART>,
    rx_buf: Vec<u8, MAX_RESP_LEN>,
}

impl<USART> Esp01<USART>
where
    USART: Instance,
{
    pub fn new<PINS>(serial: Serial<USART, PINS, u8>) -> Self {
        let (tx, rx) = serial.split();

        Self {
            tx,
            rx,
            rx_buf: Vec::new(),
        }
    }

    pub fn send_command<'a, CMD: AtCommand<'a>>(
        &mut self,
        cmd: CMD,
    ) -> Result<(), core::fmt::Error> {
        self.tx.write_str(cmd.cmd())?;
        if let Some(data) = cmd.data() {
            self.tx.write_str(data)?;
        }
        self.tx.write_str("\r\n")?;
        self.rx.listen();

        Ok(())
    }

    pub fn read_byte(&mut self) -> Result<(), &'static str> {
        self.rx.unlisten();

        if self.rx.is_rx_not_empty() {
            let byte = self.rx.read().map_err(|_| "Error reading from Rx buffer")?;
            self.rx_buf.push(byte).unwrap();
        }

        self.rx.listen();
        Ok(())
    }

    pub fn get_response(&mut self) -> Result<String<MAX_RESP_LEN>, &'static str> {
        self.rx.unlisten_idle();

        let mut response = String::new();
        self.rx_buf
            .iter()
            .for_each(|b| response.push(*b as char).unwrap());
        self.rx_buf.clear();
        self.rx.clear_idle_interrupt();
        self.rx.listen_idle();
        Ok(response)
    }

    pub fn is_response_ready(&self) -> bool {
        self.rx.is_idle()
    }
}
