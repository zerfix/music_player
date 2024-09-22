use tui::Frame;
use tui::backend::Backend;
use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Style;
use tui::widgets::List;
use tui::widgets::ListItem;
use crate::state::state_library::LibraryColumn;
use crate::state::state_library::LibraryTab;
use crate::tasks::listener_tui::RenderDataCommon;
use crate::traits::trait_listable::Listable;
use crate::traits::trait_listable::ListRenderable;
use crate::types::types_library_entry::LibraryFilterEntry;
use crate::types::types_library_entry::LibraryTrackEntry;


//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct RenderDataViewLibrary {
    pub column_selected: LibraryColumn,
    pub tabs: Vec<LibraryTab>,
    pub tabs_selected: LibraryTab,
    pub left: Vec<LibraryFilterEntry>,
    pub left_selected: usize,
    pub right: Vec<LibraryTrackEntry>,
    pub right_selected: usize,
}

pub fn draw_library_view<B: Backend>(
    frame: &mut Frame<B>,
    area: Rect,
    _common: RenderDataCommon,
    view: RenderDataViewLibrary,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30),
            Constraint::Length(1),
            Constraint::Min(10),
        ])
        .split(area);

    // --- Left List ---
    render_list(
        frame,
        chunks[0],
        view.column_selected == LibraryColumn::Filter,
        view.left,
        view.left_selected,
    );

    // --- Separator ---
    frame.render_widget(
        List::new(
            vec![
                ListItem::new("┃").style(Style::default().fg(Color::DarkGray));
                area.height as usize
            ]
        ),
        chunks[1]
    );

    // --- Right List ---
    render_list(
        frame,
        chunks[2],
        view.column_selected == LibraryColumn::Tracks,
        view.right,
        view.right_selected,
    );

    // state.list_filter.render(frame, chunks[0], state.selected_column == LibraryColumn::Filter);
    // frame.render_widget(List::new(vec![ListItem::new("|"); area.height as usize]), chunks[1]);
    // state.list_tracks.render(frame, chunks[2], state.selected_column == LibraryColumn::Tracks);
}

fn render_list<'a, B: Backend, T: Listable + ListRenderable>(frame: &mut Frame<B>, area: Rect, active: bool, list: Vec<T>, selected: usize) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(1); area.height as usize])
        .split(area)
        .into_iter();

    list.into_iter()
        .zip(layout)
        .enumerate()
        .for_each(|(i, (element, chunk))| {
            element.render(frame, chunk, active, i == selected)            
        })
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
