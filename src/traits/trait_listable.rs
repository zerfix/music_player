use tui::Frame;
use tui::backend::Backend;
use tui::layout::Rect;

//-//////////////////////////////////////////////////////////////////
pub trait Listable {
    fn is_selectable(&self) -> bool;
}

pub trait ListRenderable {
    fn render<B: Backend>(self, frame: &mut Frame<B>, area: Rect, is_active: bool, is_selected: bool);
}
//-//////////////////////////////////////////////////////////////////
