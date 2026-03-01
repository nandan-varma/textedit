use crate::error::Result;

pub trait Renderer {
    fn render(&mut self) -> Result<()>;
    fn resize(&mut self, width: u32, height: u32);
    fn update_geometry(&mut self /* params */) -> Result<()>;
    fn request_redraw(&self);
}
