use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crate::ui::utils::ui_loading_icon_util::loading_icon;
use crate::ui::utils::ui_loading_icon_util::LOADING_ICONS_LEN;

//-////////////////////////////////////////////////////////////////////////////
static TERM_WIDTH      : AtomicU16   = AtomicU16  ::new(0);
static TERM_HEIGHT     : AtomicU16   = AtomicU16  ::new(0);
static SCANNING_LIBRARY: AtomicBool  = AtomicBool ::new(false);
static PROGRESS_WIDTH  : AtomicUsize = AtomicUsize::new(0);
static LOADING_INTERVAL: AtomicU8    = AtomicU8   ::new(0);
//-////////////////////////////////////////////////////////////////////////////
/// Contains information for UI rendering
pub struct GlobalUiState {}
#[derive(Debug)]
#[derive(Clone, Copy)]
pub struct GlobalUiStateSnapshot {
    pub width : u16,
    pub height: u16,
    pub is_scanning: bool,
    pub progress_width: usize, // The total dynamic width for the progress indicator
    pub loading_rotation: u8,
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
impl GlobalUiState {
    // -- Store -------------------------------------------
    pub fn update_term_size(width: u16, height: u16) {
        TERM_WIDTH .store(width , Ordering::Relaxed);
        TERM_HEIGHT.store(height, Ordering::Relaxed);
    }

    pub fn update_scanning_state(is_scanning: bool) {
        SCANNING_LIBRARY.store(is_scanning, Ordering::Relaxed);
    }

    pub fn update_progress_width(width: usize) {
        PROGRESS_WIDTH.store(width, Ordering::Relaxed);
    }

    pub fn increment_loading_rotation() {
        let state = LOADING_INTERVAL.load(Ordering::Relaxed) + 1;
        match state {
            LOADING_ICONS_LEN.. => LOADING_INTERVAL.store(0, Ordering::Relaxed),
            _ => LOADING_INTERVAL.store(state, Ordering::Relaxed),
        }
    }

    // -- Read --------------------------------------------
    pub fn width()            -> u16   {TERM_WIDTH .load(Ordering::Relaxed)}
    pub fn height()           -> u16   {TERM_HEIGHT.load(Ordering::Relaxed)}
    pub fn is_scanning()      -> bool  {SCANNING_LIBRARY.load(Ordering::Relaxed)}
    pub fn progress_width()   -> usize {PROGRESS_WIDTH.load(Ordering::Relaxed)}
    pub fn loading_rotation() -> u8    {LOADING_INTERVAL.load(Ordering::Relaxed)}
    pub fn loading_icon()     -> char  {loading_icon(GlobalUiState::loading_rotation())}

    pub fn snapshot() -> GlobalUiStateSnapshot {
        GlobalUiStateSnapshot {
            width           : TERM_WIDTH .load(Ordering::Relaxed),
            height          : TERM_HEIGHT.load(Ordering::Relaxed),
            is_scanning     : SCANNING_LIBRARY.load(Ordering::Relaxed),
            progress_width  : PROGRESS_WIDTH.load(Ordering::Relaxed),
            loading_rotation: LOADING_INTERVAL.load(Ordering::Relaxed),
        }
    }
}

impl GlobalUiStateSnapshot {
    pub fn loading_icon(&self) -> char {
        loading_icon(self.loading_rotation)
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
