use anyhow::anyhow;
use anyhow::Error;
use crate::MsgChannels;
use crate::state::state_app::AppState;
use crate::state::state_interface::CurrentView;
use crate::tasks::listener_state::StateActions;
use crate::ui::views::view_library::draw_library_view;
use crossterm::execute;
use crossterm::terminal;
use std::io::stdout;
use std::sync::mpsc::Receiver;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use tui::backend::Backend;
use tui::backend::CrosstermBackend;
use tui::Frame;
use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;
use tui::Terminal;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub fn start_tui_listener(rx: Receiver<Option<AppState>>, tx: MsgChannels) {
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

    let interval_ms = 10;
    let mut last_iteration_instant = Instant::now();
    let mut last_iteration_id = usize::MAX;

    let tx_state = tx.tx_state;
    let tx_main  = tx.tx_exit;

    // -- Render Loop -----------------------------------------------
    let mut core_loop = || -> Result<(), Error> {
        let target_ms = last_iteration_instant + Duration::from_millis(interval_ms);
        sleep(target_ms - Instant::now());
        last_iteration_instant = Instant::now();

        if let Err(err) = tx_state.send(StateActions::GetState{
            iteration: last_iteration_id,
            body_height: terminal.size().unwrap().height.saturating_sub(2), // height minus top and bottom bar
        }) {
            return Err(anyhow!(err.to_string()));
        };
        match rx.recv()? {
            None => (),
            Some(state) => {
                last_iteration_id = state.iteration;
                terminal.draw(|frame| render_tui(state, frame))?;
            }
        }
        Ok(())
    };

    info!("Running tui render loop");
    loop {
        if let Err(err) = core_loop() {
            error!("{:?}", err);
            break;
        }
    }

    // -- Cleanup Terminal ---------------------------------------------
    info!("Render loop broken, cleaning up terminal");
    terminal::disable_raw_mode().unwrap();
    execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture,
    ).unwrap();
    terminal.show_cursor().unwrap();

    tx_main.send(()).unwrap();
}
//-////////////////////////////////////////////////////////////////////////////
pub fn render_tui<B: Backend>(mut state: AppState, frame: &mut Frame<B>) {
    let chuncs = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(100),
        ])
        .split(frame.size());

    match &state.interface.current_view {
        CurrentView::Library => draw_library_view(frame, chuncs[0], &mut state.library),
    };

}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
