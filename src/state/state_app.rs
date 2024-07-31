use crate::state::state_interface::StateInterface;
use crate::state::state_library::LibraryTab;
use crate::state::state_library::StateLibrary;
use crate::state::state_playlist::StatePlaylist;
use crate::tasks::listener_tui::RenderDataCommon;
use crate::tasks::listener_tui::RenderDataView;
use crate::ui::views::view_library::RenderDataViewLibrary;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
#[derive(Debug)]
pub struct AppState {
    pub interface: StateInterface,
    pub library  : StateLibrary,
    pub playlist : StatePlaylist,
    pub has_changed: bool,
    pub term_height: usize,
}

impl AppState {
    pub fn init() -> AppState {
        AppState{
            has_changed: true,
            term_height: 0,
            interface: StateInterface::init(),
            library  : StateLibrary::init(),
            playlist : StatePlaylist::init(),
        }
    }

    pub fn mut_interface<F: FnOnce(&mut StateInterface)>(&mut self, mutate: F){
        mutate(&mut self.interface);
        self.has_changed = true;
    }

    pub fn mut_library<F: FnOnce(&mut StateLibrary)>(&mut self, mutate: F) {
        mutate(&mut self.library);
        self.has_changed = true;
    }

    pub fn render_state(&mut self, force: bool) -> Option<(RenderDataCommon, RenderDataView)> {
        if force || !self.has_changed {
            return None
        }

        self.has_changed = false;

        let common = RenderDataCommon{
            is_scanning: self.interface.is_scanning,
            term_height: self.term_height
        };

        let (left ,  left_selected) = self.library.list_filter.view(self.term_height);
        let (right, right_selected) = self.library.list_tracks.view(self.term_height);
        let right = right.into_iter().map(|e| e.to_track_entry()).collect::<Vec<_>>();

        let view = RenderDataView::Library(
            RenderDataViewLibrary{
                column_selected: self.library.selected_column,
                tabs: vec![LibraryTab::Artists],
                tabs_selected: LibraryTab::Artists,
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
