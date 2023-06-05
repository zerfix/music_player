use crate::state::state_library::LibraryColumn;
use crate::state::state_library::StateLibrary;
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;
use tui::layout::Rect;
use tui::widgets::List;
use tui::widgets::ListItem;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub fn draw_library_view<B: Backend>(frame: &mut Frame<B>, area: Rect, state: &mut StateLibrary) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30),
            Constraint::Length(1),
            Constraint::Min(10),
        ])
        .split(area);

    state.list_filter.render(frame, chunks[0], state.selected_column == LibraryColumn::Filter);
    frame.render_widget(List::new([ListItem::new("|")]), chunks[1]);
    state.list_tracks.render(frame, chunks[2], state.selected_column == LibraryColumn::Tracks);
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
