use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardResource {
    Microbe,
    Plant,
    Animal,
    Science,
    Fighter,
}

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resource {
    Megacredits,
    Steel,
    Titanium,
    Plants,
    Energy,
    Heat,
}

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaymentCost {
    Megacredits(usize),
    Space(usize),
    Building(usize),
    SpaceOrBuilding(usize),
    Steel(usize),
    Titanium(usize),
    Plants(usize),
    Energy(usize),
    Heat(usize),
}
