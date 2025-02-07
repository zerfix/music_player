use color_eyre::Result;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use crossterm::event::MouseEventKind;
use crossterm::event;
use std::time::Duration;
use crate::enums::enum_input::InputGlobal;
use crate::tasks::listener_state::StateActions;
use crate::types::types_msg_channels::MsgChannels;
use crate::enums::enum_input::InputLocal;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub fn start_input_listener(tx: MsgChannels) {
    if let Err(err) = input_loop(&tx) {
        error!("{}", err);
        tx.exit.send(Err(err)).unwrap();
    }
}

fn input_loop(tx: &MsgChannels) -> Result<()> {
    let send_l = |input: InputLocal | tx.state.send(StateActions::InputLocal(input));
    let send_g = |input: InputGlobal| tx.state.send(StateActions::InputGlobal(input));

    loop {
        if event::poll(Duration::from_secs(60*60)).is_ok() {
            if let Ok(event) = event::read() {
                debug!("{:?}", &event);
                match event {
                    Event::FocusGained => (),
                    Event::FocusLost => (),
                    Event::Mouse(event) => match event.kind {
                        MouseEventKind::ScrollUp   => send_l(InputLocal::Up)?,
                        MouseEventKind::ScrollDown => send_l(InputLocal::Down)?,
                        _ => (),
                    },
                    Event::Key(key) => match key.code {
                        // wasd
                        KeyCode::Char('w') => send_l(InputLocal::Up)?,
                        KeyCode::Char('a') => send_l(InputLocal::Down)?,
                        KeyCode::Char('s') => send_l(InputLocal::Left)?,
                        KeyCode::Char('d') => send_l(InputLocal::Right)?,
                        // vim
                        KeyCode::Char('k') => send_l(InputLocal::Up)?,
                        KeyCode::Char('j') => send_l(InputLocal::Down)?,
                        KeyCode::Char('h') => send_l(InputLocal::Left)?,
                        KeyCode::Char('l') => send_l(InputLocal::Right)?,
                        // arrows
                        KeyCode::Up        => send_l(InputLocal::Up)?,
                        KeyCode::Down      => send_l(InputLocal::Down)?,
                        KeyCode::Left      => send_l(InputLocal::Left)?,
                        KeyCode::Right     => send_l(InputLocal::Right)?,
                        // extra nav
                        KeyCode::PageUp    => send_l(InputLocal::PgUp)?,
                        KeyCode::PageDown  => send_l(InputLocal::PgDown)?,
                        KeyCode::Home      => send_l(InputLocal::Home)?,
                        KeyCode::End       => send_l(InputLocal::End)?,
                        KeyCode::Tab       => match key.modifiers {
                            KeyModifiers::SHIFT => send_l(InputLocal::RevTab)?,
                            _                   => send_l(InputLocal::Tab)?,
                        },
                        // actions
                        KeyCode::Enter     => send_l(InputLocal::Select)?,
                        KeyCode::Char(' ') => send_l(InputLocal::Select)?,
                        // global shortcuts
                        KeyCode::Char('x') => send_g(InputGlobal::Previous)?,
                        KeyCode::Char('c') => send_g(InputGlobal::PlayPause)?,
                        KeyCode::Char('v') => send_g(InputGlobal::Stop)?,
                        KeyCode::Char('b') => send_g(InputGlobal::Next)?,
                        KeyCode::Char('q') => tx.exit.send(Ok("".to_string()))?,
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
