use crossterm::execute;
use crossterm::terminal;
use std::io::stdout;
use std::sync::mpsc::Receiver;
use std::time::SystemTime;
use tui::Frame;
use tui::Terminal;
use tui::backend::Backend;
use tui::backend::CrosstermBackend;
use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;
use crate::MsgChannels;
use crate::tasks::listener_state::StateActions;
use crate::ui::views::view_library::RenderDataViewLibrary;
use crate::ui::views::view_library::draw_library_view;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub enum RenderActions {
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
    pub term_height: usize,
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
        crossterm::event::EnableMouseCapture,
    ).unwrap();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // -- Render Loop -----------------------------------------------
    info!("Running tui render loop");
    while let Ok(msg) = rx.recv() {
        match msg {
            RenderActions::RenderFrame{render_start, common, view} => {
                let term_height = (terminal.size().unwrap().height) as usize;
                if term_height != common.term_height {
                    tx_state.send(StateActions::TuiUpdateTermHeight{render_start, term_height}).unwrap()
                } else {
                    if let Err(err) = terminal.draw(|frame| render_tui(common, view, frame)) {
                        error!("Render error: {:?}", err)
                    }
                    info!("Render_time: {:?}", render_start.elapsed().unwrap());
                }
            },
            RenderActions::Exit => {
                info!("resetting terminal");
                if let Err(err) = terminal::disable_raw_mode() {
                    error!("Disable raw terminal mode error: {:?}", err);
                };
                if let Err(err) = execute!(
                    terminal.backend_mut(),
                    terminal::LeaveAlternateScreen,
                    crossterm::event::DisableMouseCapture,
                ) {
                    error!("Reset terminal error: {:?}", err);
                };
                if let Err(err) = terminal.show_cursor() {
                    error!("Reset terminal cursor error: {:?}", err);
                };
                break;
            }
        }
    }

    tx_exit.send(()).unwrap();
}
//-////////////////////////////////////////////////////////////////////////////
pub fn render_tui<B: Backend>(common: RenderDataCommon, view: RenderDataView, frame: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Percentage(100),
        ])
        .split(frame.size());

    match view {
        RenderDataView::Library(view) => draw_library_view(frame, chunks[0], common, view),
    };

}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
