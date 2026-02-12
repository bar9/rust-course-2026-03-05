use std::any::Any;
use crate::liquid::Liquid;

pub struct Water;

impl Liquid for Water {
    fn description(&self) -> &str {
        "water"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
