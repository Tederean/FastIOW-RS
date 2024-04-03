use crate::spi::SPIError;
use std::marker::PhantomData;

#[cfg(feature = "embedded-hal")]
pub struct SPIDevice<B>
where
    B: embedded_hal::spi::SpiBus<u8>,
{
    phantom: PhantomData<B>,
}

#[cfg(not(feature = "embedded-hal"))]
pub struct SPIDevice {}

#[cfg(feature = "embedded-hal")]
impl<B: embedded_hal::spi::SpiBus<u8>> embedded_hal::spi::SpiDevice<u8> for SPIDevice<B> {
    fn transaction(
        &mut self,
        operations: &mut [embedded_hal::spi::Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        todo!()
    }
}

#[cfg(feature = "embedded-hal")]
impl<B: embedded_hal::spi::SpiBus<u8>> embedded_hal::spi::ErrorType for SPIDevice<B> {
    type Error = SPIError;
}

#[cfg(all(feature = "embedded-hal", feature = "embedded-hal-0"))]
impl<B> embedded_hal_0::blocking::spi::Transfer<u8> for SPIDevice<B>
where
    B: embedded_hal::spi::SpiBus<u8>,
{
    type Error = SPIError;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        todo!()
    }
}

#[cfg(all(not(feature = "embedded-hal"), feature = "embedded-hal-0"))]
impl embedded_hal_0::blocking::spi::Transfer<u8> for SPIDevice {
    type Error = SPIError;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        todo!()
    }
}

#[cfg(all(feature = "embedded-hal", feature = "embedded-hal-0"))]
impl<B> embedded_hal_0::blocking::spi::Write<u8> for SPIDevice<B>
where
    B: embedded_hal::spi::SpiBus<u8>,
{
    type Error = SPIError;

    fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        todo!()
    }
}

#[cfg(all(not(feature = "embedded-hal"), feature = "embedded-hal-0"))]
impl embedded_hal_0::blocking::spi::Write<u8> for SPIDevice {
    type Error = SPIError;

    fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        todo!()
    }
}

#[cfg(all(feature = "embedded-hal", feature = "embedded-hal-0"))]
impl<B> embedded_hal_0::blocking::spi::WriteIter<u8> for SPIDevice<B>
where
    B: embedded_hal::spi::SpiBus<u8>,
{
    type Error = SPIError;

    fn write_iter<WI>(&mut self, words: WI) -> Result<(), Self::Error>
    where
        WI: IntoIterator<Item = u8>,
    {
        todo!()
    }
}

#[cfg(all(not(feature = "embedded-hal"), feature = "embedded-hal-0"))]
impl embedded_hal_0::blocking::spi::WriteIter<u8> for SPIDevice {
    type Error = SPIError;

    fn write_iter<WI>(&mut self, words: WI) -> Result<(), Self::Error>
    where
        WI: IntoIterator<Item = u8>,
    {
        todo!()
    }
}
