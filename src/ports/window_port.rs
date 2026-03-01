use crate::error::Result;

pub trait Window {
    fn request_redraw(&self);
    fn inner_size(&self) -> (u32, u32);
    fn scale_factor(&self) -> f64;
}
