use anyhow::Error;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use crossterm::event::MouseEventKind;
use crossterm::event;
use std::time::Duration;
use crate::enums::enum_navigate::Navigate;
use crate::tasks::listener_state::StateActions;
use crate::tasks::listener_tui::RenderActions;
use crate::types::types_msg_channels::MsgChannels;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub fn start_input_listener(tx: MsgChannels) {
    if let Err(err) = input_loop(&tx) {
        error!("{}", err);
    }
    tx.tx_tui.send(RenderActions::Exit).unwrap();
}

fn input_loop(tx: &MsgChannels) -> Result<(), Error> {
    let tx_state = &tx.tx_state;
    loop {
        if event::poll(Duration::from_secs(60*60)).is_ok() {
            if let Ok(event) = event::read() {
                debug!("{:?}", &event);
                match event {
                    Event::FocusGained => (),
                    Event::FocusLost => (),
                    Event::Mouse(event) => match event.kind {
                        MouseEventKind::ScrollUp   => tx_state.send(StateActions::InputNavigate(Navigate::Up))?,
                        MouseEventKind::ScrollDown => tx_state.send(StateActions::InputNavigate(Navigate::Down))?,
                        _ => (),
                    },
                    Event::Key(key) => match key.code {
                        // wasd
                        KeyCode::Char('w') => tx_state.send(StateActions::InputNavigate(Navigate::Up))?,
                        KeyCode::Char('a') => tx_state.send(StateActions::InputNavigate(Navigate::Down))?,
                        KeyCode::Char('s') => tx_state.send(StateActions::InputNavigate(Navigate::Left))?,
                        KeyCode::Char('d') => tx_state.send(StateActions::InputNavigate(Navigate::Right))?,
                        // vim
                        KeyCode::Char('k') => tx_state.send(StateActions::InputNavigate(Navigate::Up))?,
                        KeyCode::Char('j') => tx_state.send(StateActions::InputNavigate(Navigate::Down))?,
                        KeyCode::Char('h') => tx_state.send(StateActions::InputNavigate(Navigate::Left))?,
                        KeyCode::Char('l') => tx_state.send(StateActions::InputNavigate(Navigate::Right))?,
                        // arrows
                        KeyCode::Up        => tx_state.send(StateActions::InputNavigate(Navigate::Up))?,
                        KeyCode::Down      => tx_state.send(StateActions::InputNavigate(Navigate::Down))?,
                        KeyCode::Left      => tx_state.send(StateActions::InputNavigate(Navigate::Left))?,
                        KeyCode::Right     => tx_state.send(StateActions::InputNavigate(Navigate::Right))?,
                        // extra nav
                        KeyCode::PageUp    => tx_state.send(StateActions::InputNavigate(Navigate::PgUp))?,
                        KeyCode::PageDown  => tx_state.send(StateActions::InputNavigate(Navigate::PgDown))?,
                        KeyCode::Home      => tx_state.send(StateActions::InputNavigate(Navigate::Home))?,
                        KeyCode::End       => tx_state.send(StateActions::InputNavigate(Navigate::End))?,
                        KeyCode::Tab       => match key.modifiers {
                            KeyModifiers::SHIFT => tx_state.send(StateActions::InputNavigate(Navigate::RevTab))?,
                            _                   => tx_state.send(StateActions::InputNavigate(Navigate::Tab))?,
                        },
                        // actions
                        KeyCode::Enter     => tx_state.send(StateActions::PlaybackPlayFrom)?, // enter/play
                        KeyCode::Char(' ') => (), // add track to playlist, +shift add album, +alt add artist
                        KeyCode::Char('q') => tx_state.send(StateActions::Exit)?,
                        KeyCode::Char('c') => (), // play/pause
                        KeyCode::Char('v') => tx_state.send(StateActions::PlaybackStopAndClear)?, // stop
                        _ => (),
                    },
                    _ => (),
                }
            }
        }
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
