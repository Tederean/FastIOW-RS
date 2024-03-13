use crate::iowkit::IOWarriorData;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub enum IOWarrior {
    IOWarrior40(IOWarrior40),
    IOWarrior24(IOWarrior24),
    IOWarrior28(IOWarrior28),
    IOWarrior28L(IOWarrior28L),
    IOWarrior28Dongle(IOWarrior28Dongle),
    IOWarrior56(IOWarrior56),
    IOWarrior56Dongle(IOWarrior56Dongle),
    IOWarrior56Beta(IOWarrior56Beta),
    IOWarrior100(IOWarrior100),
}

impl fmt::Display for IOWarrior {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum IOWarriorType {
    IOWarrior40,
    IOWarrior24,
    IOWarrior28,
    IOWarrior28L,
    IOWarrior56,
    IOWarrior100,
}

impl fmt::Display for IOWarriorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct IOWarrior40 {
    pub device_data: Rc<IOWarriorData>,
}

impl fmt::Display for IOWarrior40 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct IOWarrior24 {
    pub device_data: Rc<IOWarriorData>,
}

impl fmt::Display for IOWarrior24 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct IOWarrior28 {
    pub device_data: Rc<IOWarriorData>,
}

impl fmt::Display for IOWarrior28 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct IOWarrior28L {
    pub device_data: Rc<IOWarriorData>,
}

impl fmt::Display for IOWarrior28L {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct IOWarrior28Dongle {
    pub device_data: Rc<IOWarriorData>,
}

impl fmt::Display for IOWarrior28Dongle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct IOWarrior56 {
    pub device_data: Rc<IOWarriorData>,
}

impl fmt::Display for IOWarrior56 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct IOWarrior56Dongle {
    pub device_data: Rc<IOWarriorData>,
}

impl fmt::Display for IOWarrior56Dongle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct IOWarrior56Beta {
    pub device_data: Rc<IOWarriorData>,
}

impl fmt::Display for IOWarrior56Beta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct IOWarrior100 {
    pub device_data: Rc<IOWarriorData>,
}

impl fmt::Display for IOWarrior100 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
