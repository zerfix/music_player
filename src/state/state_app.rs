use crate::state::state_interface::StateInterface;
use crate::state::state_library::LibraryTab;
use crate::state::state_library::StateLibrary;
use crate::state::state_playlist::StatePlaylist;
use crate::tasks::listener_tui::RenderDataCommon;
use crate::tasks::listener_tui::RenderDataView;
use crate::types::types_tui::TermSize;
use crate::ui::views::view_library::RenderDataViewLibrary;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
#[derive(Debug)]
pub struct AppState {
    interface  : StateInterface,
    library    : StateLibrary,
    playlist   : StatePlaylist,
    has_changed: bool,
    term_size  : TermSize,
}

impl AppState {
    pub fn init() -> AppState {
        AppState{
            has_changed: true,
            term_size: TermSize{width: 0, height: 0},
            interface: StateInterface::init(),
            library  : StateLibrary::init(),
            playlist : StatePlaylist::init(),
        }
    }

    pub fn mutate<F: FnOnce(
        &mut StateInterface,
        &mut StateLibrary,
        &mut StatePlaylist,
    )>(&mut self, mutate: F){
        mutate(
            &mut self.interface,
            &mut self.library,
            &mut self.playlist,
        );
        self.has_changed = true;
    }

    pub fn render_state(&mut self, force: bool, term_size: TermSize) -> Option<(RenderDataCommon, RenderDataView)> {
        if term_size != self.term_size {
            self.has_changed = true;
            self.term_size = term_size;
        }
        if force || !self.has_changed {
            return None
        }

        self.has_changed = false;

        let common = RenderDataCommon{
            is_scanning: self.interface.is_scanning,
            playlist   : self.playlist.clone(),
        };

        let (left ,  left_selected) = self.library.list_filter.view(self.term_size.height.saturating_sub(1));
        let (right, right_selected) = self.library.list_tracks.view(self.term_size.height.saturating_sub(1));

        let view = RenderDataView::Library(
            RenderDataViewLibrary{
                column_selected: self.library.selected_column,
                tabs: vec![LibraryTab::Artists],
                tab_selected: self.library.selected_tab,
                left,
                left_selected,
                right,
                right_selected,
            }
        );

        Some((
            common,
            view,
        ))
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
