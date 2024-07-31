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
    frame.render_widget(
        List::new(
            assemble_list(
                chunks[0],
                view.column_selected == LibraryColumn::Filter,
                view.left,
                view.left_selected,
            )
        ),
        chunks[0],
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
    frame.render_widget(
        List::new(
            assemble_list(
                chunks[2],
                view.column_selected == LibraryColumn::Tracks,
                view.right,
                view.right_selected
            )
        ),
        chunks[2]
    );

    // state.list_filter.render(frame, chunks[0], state.selected_column == LibraryColumn::Filter);
    // frame.render_widget(List::new(vec![ListItem::new("|"); area.height as usize]), chunks[1]);
    // state.list_tracks.render(frame, chunks[2], state.selected_column == LibraryColumn::Tracks);
}

fn assemble_list<'a, T: Listable>(area: Rect, active: bool, list: Vec<T>, selected: usize) -> Vec<ListItem<'a>> {
    // todo: do style globally somewhere else
    let selected_style = match active {
        true => Style::default()
            .fg(Color::White)
            .bg(Color::Rgb(20, 100, 20)),
        false => Style::default()
            .fg(Color::White)
            .bg(Color::DarkGray),
    };
    let unselectable_style = Style::default()
        .fg(Color::White)
        .bg(Color::Rgb(20, 20, 100));

    list.into_iter()
        .map(|element| (
            element.is_selectable(),
            element.to_list_item(area.width as usize),
        ))
        .enumerate()
        .map(|(i, (selectable, element))| match (selectable, i == selected) {
            (true , true ) => element.style(selected_style),
            (true , false) => element,
            (false, _    ) => element.style(unselectable_style)
        })
        .take(area.height as usize)
        .collect::<Vec<ListItem>>()
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
