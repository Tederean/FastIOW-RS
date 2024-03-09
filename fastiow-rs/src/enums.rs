use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum IOWarriorType {
    Unknown,
    IOWarrior40,
    IOWarrior24,
    IOWarrior56,
    IOWarrior28,
    IOWarrior28L,
    IOWarrior100,
}

#[derive(Debug, Copy, Clone)]
pub enum Pipe {
    IOPins,
    SpecialMode,
    I2CMode,
    ADCMode,
}

#[derive(Debug, Copy, Clone)]
pub enum DigitalPinState {
    Low,
    High,
}

impl Pipe {
    fn get_value(&self) -> u8 {
        match self {
            Pipe::IOPins => 0,
            Pipe::SpecialMode => 1,
            Pipe::I2CMode => 2,
            Pipe::ADCMode => 3,
        }
    }
}

impl fmt::Display for IOWarriorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Pipe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for DigitalPinState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
