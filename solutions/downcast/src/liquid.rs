use std::any::Any;

pub trait Liquid {
    fn description(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}
