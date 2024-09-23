use crossterm::execute;
use crossterm::style::Print;
use crossterm::terminal;
use crossterm::event;
use crossterm::cursor;
use std::io::stdout;
use std::sync::mpsc::Receiver;
use std::time::SystemTime;
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
        render_start: SystemTime,
    },
    RenderFrame{
        render_start: SystemTime,
        common: RenderDataCommon,
        view: RenderDataView,
    },
    Exit,
}

#[derive(Debug)]
pub struct RenderDataCommon {
    pub is_scanning: bool,
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
            RenderActions::RenderRequest{render_start} => {
                let term_size = TermSize::new().unwrap();
                tx_state.send(StateActions::Render{render_start, term_size}).unwrap()
            },
            RenderActions::RenderFrame{render_start, common, view} => {
                let term_size = TermSize::new().unwrap();

                match view {
                    RenderDataView::Library(view) => draw_library_view(&mut term_state, term_size, view),
                }

                execute!(
                    stdout,
                    terminal::BeginSynchronizedUpdate,
                    terminal::Clear(terminal::ClearType::All),
                    cursor::MoveTo(0,0),
                    Print(term_state.output()),
                    terminal::EndSynchronizedUpdate,
                ).unwrap();

                term_state.clear();

                info!("Render_time: {:?}", render_start.elapsed().unwrap());
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
