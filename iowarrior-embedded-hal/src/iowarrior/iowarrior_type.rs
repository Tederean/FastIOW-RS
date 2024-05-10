use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IOWarriorType {
    IOWarrior40,
    IOWarrior24,
    IOWarrior24PowerVampire,
    IOWarrior56,
    IOWarrior56Dongle,
    IOWarrior28,
    IOWarrior28Dongle,
    IOWarrior28L,
    IOWarrior100,
}

impl IOWarriorType {
    pub fn from_device_product_id(device_product_id: u16) -> Option<IOWarriorType> {
        match device_product_id {
            5376 => Some(IOWarriorType::IOWarrior40),
            5377 => Some(IOWarriorType::IOWarrior24),
            5393 | 5394 => Some(IOWarriorType::IOWarrior24PowerVampire),
            5379 => Some(IOWarriorType::IOWarrior56),
            5380 => Some(IOWarriorType::IOWarrior28),
            5381 => Some(IOWarriorType::IOWarrior28L),
            5382 => Some(IOWarriorType::IOWarrior100),
            _ => None,
        }
    }
}

impl fmt::Display for IOWarriorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
