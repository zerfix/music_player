use crossbeam_channel::Receiver;
use crossterm::cursor;
use crossterm::event;
use crossterm::execute;
use crossterm::style::Print;
use crossterm::terminal;
use std::io::stdout;
use std::time::Duration;
use std::time::Instant;
use crate::state::state_playlist::StatePlaylist;
use crate::types::types_tui::TermSize;
use crate::types::types_tui::TermState;
use crate::MsgChannels;
use crate::tasks::listener_state::StateActions;
use crate::ui::views::view_library::RenderDataViewLibrary;
use crate::ui::views::view_library::draw_library_view;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub enum RenderActions {
    RenderRequest{
        render_start: Instant,
        interval: u8,
    },
    RenderFrame{
        render_start  : Instant,
        render_request: Duration,
        render_state  : Duration,
        common: RenderDataCommon,
        view: RenderDataView,
    },
    Exit,
}

#[derive(Debug)]
pub struct RenderDataCommon {
    pub interval: u8,
    pub is_scanning: bool,
    pub playlist: StatePlaylist,
}

#[derive(Debug)]
pub enum RenderDataView {
    Library(RenderDataViewLibrary),
}

pub fn start_tui_listener(rx: Receiver<RenderActions>, tx: MsgChannels) {
    let tx_state = tx.tx_state;
    let tx_exit  = tx.tx_exit;

    // -- Init Tui --------------------------------------------------
    info!("Setting up terminal...");
    let mut stdout = stdout();
    terminal::enable_raw_mode().unwrap();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture,
        cursor::Hide,
    ).unwrap();

    let mut term_state = TermState::new();

    // -- Render Loop -----------------------------------------------
    info!("Running tui render loop");
    while let Ok(msg) = rx.recv() {
        match msg {
            RenderActions::RenderRequest{render_start, interval} => {
                let term_size = TermSize::new().unwrap();
                tx_state.send(StateActions::Render{
                    interval,
                    render_start,
                    render_request: render_start.elapsed(),
                    term_size,
                }).unwrap()
            },
            RenderActions::RenderFrame{
                render_start,
                render_request,
                render_state,
                common,
                view
            } => {
                let term_size = TermSize::new().unwrap();

                match view {
                    RenderDataView::Library(view) => draw_library_view(&mut term_state, term_size, &common, view),
                }

                let render_layout = render_start.elapsed();

                execute!(
                    stdout,
                    terminal::BeginSynchronizedUpdate,
                    cursor::MoveTo(0,0),
                    Print(term_state.output()),
                    terminal::EndSynchronizedUpdate,
                ).unwrap();

                let render_output = render_start.elapsed();

                info!(
                    "Render {:?}: start > request {:?} > copy state {:?} > render {:?} > output {:?}",
                    render_output,
                    render_request,
                    render_state  - render_request,
                    render_layout - render_state,
                    render_output - render_layout,
                );

                term_state.clear();
            },
            RenderActions::Exit => {
                info!("resetting terminal");
                if let Err(err) = terminal::disable_raw_mode() {
                    error!("Disable raw terminal mode error: {:?}", err);
                };
                if let Err(err) = execute!(
                    stdout,
                    terminal::LeaveAlternateScreen,
                    event::DisableMouseCapture,
                    cursor::Show,
                ) {
                    error!("Reset terminal error: {:?}", err);
                };
                break;
            }
        }
    }

    tx_exit.send(()).unwrap();
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
