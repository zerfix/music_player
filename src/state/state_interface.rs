use crate::types::types_tui::TermSize;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Copy)]
#[derive(Debug)]
pub struct StateInterface {
    pub interval: u8,
    pub term_size: TermSize,
    pub is_scanning: bool,
    pub current_view: CurrentView,
}

#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum CurrentView {
    Library,
}

impl StateInterface {
    pub fn init() -> StateInterface {
        StateInterface{
            interval: 0,
            term_size: TermSize::new().unwrap(),
            is_scanning: false,
            current_view: CurrentView::Library,
        }
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
