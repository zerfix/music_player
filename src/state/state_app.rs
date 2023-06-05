use crate::state::state_config::Config;
use crate::state::state_interface::StateInterface;
use crate::state::state_library::StateLibrary;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
#[derive(Debug)]
pub struct AppState {
    pub iteration: usize, // render on change
    pub config   : Config,
    pub interface: StateInterface,
    pub library  : StateLibrary,
}

impl AppState {
    pub fn init() -> AppState {
        AppState{
            iteration: 0,
            config   : Config::init(),
            interface: StateInterface::init(),
            library  : StateLibrary::init(),
        }
    }

    pub fn mut_interface<F: FnOnce(&mut StateInterface)>(&mut self, mutate: F){
        mutate(&mut self.interface);
        self.iteration = self.iteration.overflowing_add(1).0;
    }

    pub fn mut_library<F: FnOnce(&mut StateLibrary)>(&mut self, mutate: F) {
        mutate(&mut self.library);
        self.iteration  = self.iteration.overflowing_add(1).0;
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
