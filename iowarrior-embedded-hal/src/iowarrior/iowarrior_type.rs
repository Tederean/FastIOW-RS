use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IOWarriorType {
    IOWarrior40,
    IOWarrior24,
    IOWarrior28,
    IOWarrior28Dongle,
    IOWarrior28L,
    IOWarrior56,
    IOWarrior56Dongle,
    IOWarrior100,
}

impl fmt::Display for IOWarriorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
