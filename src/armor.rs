#[derive(Copy, Clone)]
pub struct Armor {
    pub max_life: u32,
    pub life: u32,
}

impl Armor {
    pub fn new(max_life: u32) -> Armor {
        Armor {
            max_life,
            life: max_life,
        }
    }
}
