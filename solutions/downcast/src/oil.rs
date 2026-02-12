use std::any::Any;
use crate::liquid::Liquid;

pub struct Oil;

impl Liquid for Oil {
    fn description(&self) -> &str {
        "oil"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
