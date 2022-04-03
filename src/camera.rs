use defmt::{write, Format, Formatter};
use embedded_hal::blocking::i2c::{Write, WriteRead};

const SCCB_SDDR: u8 = 0x60 >> 1;

const OV_PIDH_REG: u8 = 0x0a;
const OV_PIDL_REG: u8 = 0x0b;
const OV_CTRL0_REG: u8 = 0xc2;
const OV_COM2_REG: u8 = 0x09;
const OV_COM8_REG: u8 = 0x13;
const OV_IMAGE_MODE: u8 = 0xda;
const OV_BANK_SELECT: u8 = 0xff;

const OV_PIDH_VAL: u8 = 0x26;
const OV_PIDL_VAL: u8 = 0x42;
const OV_CTRL0_YUV422: u8 = 0x08;
const OV_CTRL0_YUV_EN: u8 = 0x04;
const OV_BANK_0: u8 = 0x00;
const OV_BANK_1: u8 = 0x01;

const OV_COM2_CAPABILITY_X2: u8 = 0x02;

const OV_COM8_EXPOSURE: u8 = 0x01;
const OV_COM8_AGC: u8 = 0x04;

pub enum Error<E> {
    I2c(E),
    UnknownChip,
}

impl<E> Format for Error<E> {
    fn format(&self, fmt: Formatter) {
        match self {
            Error::I2c(_) => write!(fmt, "I2C bus error"),
            Error::UnknownChip => write!(fmt, "Unknown chip"),
        }
    }
}
pub struct OvCam<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> OvCam<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
{
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            address: SCCB_SDDR,
        }
    }

    pub fn verify(&mut self) -> Result<(), Error<E>> {
        let pidh = self.read(OV_PIDH_REG)?;
        let pidl = self.read(OV_PIDL_REG)?;
        if pidh == OV_PIDH_VAL && pidl == OV_PIDL_VAL {
            Ok(())
        } else {
            Err(Error::UnknownChip)
        }
    }

    pub fn init(&mut self) -> Result<(), Error<E>> {
        self.write(OV_BANK_SELECT, OV_BANK_0)?;
        self.write(0x2c, 0xff)?;
        self.write(0x2e, 0xdf)?;

        self.write(OV_CTRL0_REG, OV_CTRL0_YUV_EN | OV_CTRL0_YUV422)?;

        self.write(OV_BANK_SELECT, OV_BANK_1)?;
        self.write(OV_COM2_REG, OV_COM2_CAPABILITY_X2)?;
        self.write(OV_COM8_REG, OV_COM8_EXPOSURE | OV_COM8_AGC)?;
        Ok(())
    }

    pub fn write(&mut self, reg: u8, val: u8) -> Result<(), Error<E>> {
        self.i2c
            .write(self.address, &[reg, val])
            .map_err(|e| Error::I2c(e))?;
        Ok(())
    }

    pub fn read(&mut self, reg: u8) -> Result<u8, Error<E>> {
        let mut buffer = [0; 1];
        self.i2c
            .write_read(self.address, &[reg], &mut buffer)
            .map_err(|e| Error::I2c(e))?;
        Ok(buffer[0])
    }
}
