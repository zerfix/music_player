use anyhow::Error;
use crossterm::event;
use crossterm::event::KeyModifiers;
use crossterm::event::MouseEventKind;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use std::time::Duration;
use crate::types::types_msg_channels::MsgChannels;
use crate::enums::enum_navigate::Navigate;
use crate::tasks::listener_playback::PlaybackActions;
use crate::tasks::listener_state::StateActions;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub fn start_input_listener(tx: MsgChannels) -> ! {
    loop {
        if let Err(err) = input_loop(&tx) {
            error!("{}", err);
        }
    }
}

fn input_loop(tx: &MsgChannels) -> Result<(), Error> {
    let tx_state    = &tx.tx_state;
    let tx_playback = &tx.tx_playback;
    loop {
        if event::poll(Duration::from_secs(60*60)).is_ok() {
            if let Ok(event) = event::read() {
                debug!("{:?}", &event);
                match event {
                    Event::FocusGained => (),
                    Event::FocusLost => (),
                    Event::Mouse(event) => match event.kind {
                        MouseEventKind::ScrollUp   => tx_state.send(StateActions::Navigate(Navigate::Up))?,
                        MouseEventKind::ScrollDown => tx_state.send(StateActions::Navigate(Navigate::Down))?,
                        _ => (),
                    },
                    Event::Key(key) => match key.code {
                        // wasd
                        KeyCode::Char('w') => tx_state.send(StateActions::Navigate(Navigate::Up))?,
                        KeyCode::Char('a') => tx_state.send(StateActions::Navigate(Navigate::Down))?,
                        KeyCode::Char('s') => tx_state.send(StateActions::Navigate(Navigate::Left))?,
                        KeyCode::Char('d') => tx_state.send(StateActions::Navigate(Navigate::Right))?,
                        // vim
                        KeyCode::Char('k') => tx_state.send(StateActions::Navigate(Navigate::Up))?,
                        KeyCode::Char('j') => tx_state.send(StateActions::Navigate(Navigate::Down))?,
                        KeyCode::Char('h') => tx_state.send(StateActions::Navigate(Navigate::Left))?,
                        KeyCode::Char('l') => tx_state.send(StateActions::Navigate(Navigate::Right))?,
                        // arrows
                        KeyCode::Up        => tx_state.send(StateActions::Navigate(Navigate::Up))?,
                        KeyCode::Down      => tx_state.send(StateActions::Navigate(Navigate::Down))?,
                        KeyCode::Left      => tx_state.send(StateActions::Navigate(Navigate::Left))?,
                        KeyCode::Right     => tx_state.send(StateActions::Navigate(Navigate::Right))?,
                        // extra nav
                        KeyCode::PageUp    => tx_state.send(StateActions::Navigate(Navigate::PgUp))?,
                        KeyCode::PageDown  => tx_state.send(StateActions::Navigate(Navigate::PgDown))?,
                        KeyCode::Home      => tx_state.send(StateActions::Navigate(Navigate::Home))?,
                        KeyCode::End       => tx_state.send(StateActions::Navigate(Navigate::End))?,
                        KeyCode::Tab       => match key.modifiers {
                            KeyModifiers::SHIFT => tx_state.send(StateActions::Navigate(Navigate::RevTab))?,
                            _                   => tx_state.send(StateActions::Navigate(Navigate::Tab))?,
                        },
                        // actions
                        KeyCode::Enter     => tx_state.send(StateActions::PlayFrom)?, // enter/play
                        KeyCode::Char(' ') => (), // add track to playlist, +shift add album, +alt add artist
                        KeyCode::Char('q') => tx_state.send(StateActions::Exit)?,
                        KeyCode::Char('c') => (), // play/pause
                        KeyCode::Char('v') => tx_playback.send(PlaybackActions::StopAndClear).unwrap(), // stop
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
