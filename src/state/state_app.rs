use crate::state::state_interface::StateInterface;
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
    interface  : StateInterface,
    library    : StateLibrary,
    playlist   : StatePlaylist,
    has_changed: bool,
}

impl AppState {
    pub fn init() -> AppState {
        AppState{
            has_changed: true,
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

    pub fn render_state(&mut self) -> Option<(RenderDataCommon, RenderDataView)> {
        self.has_changed = false;

        let common = RenderDataCommon{
            interval   : self.interface.interval,
            is_scanning: self.interface.is_scanning,
            playlist   : self.playlist.clone(),
        };

        let (left ,  left_selected) = self.library.list_filter.view(self.interface.term_size.height.saturating_sub(1));
        let (right, right_selected) = self.library.list_tracks.view(self.interface.term_size.height.saturating_sub(1));

        let view = RenderDataView::Library(RenderDataViewLibrary{
            column_selected: self.library.selected_column,
            tab_selected: self.library.selected_tab,
            track_select_mode: self.library.select_mode,
            left,
            left_selected,
            right,
            right_selected,
        });

        Some((common, view))
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
