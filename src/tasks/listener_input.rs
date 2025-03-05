use crate::enums::enum_input::InputGlobal;
use crate::enums::enum_input::InputLocal;
use crate::tasks::listener_state::StateActions;
use crate::types::types_msg_channels::MsgChannels;
use color_eyre::Result;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::MouseEventKind;
use std::time::Instant;

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
    let send_l = |input: InputLocal | tx.state.send((Instant::now(), StateActions::InputLocal(input)));
    let send_g = |input: InputGlobal| tx.state.send((Instant::now(), StateActions::InputGlobal(input)));

    loop {
        if let Ok(event) = event::read() {
            trace!("{:?}", &event);
            match event {
                Event::Mouse(event) => match event.kind {
                    MouseEventKind::ScrollUp   => send_l(InputLocal::Up)?,
                    MouseEventKind::ScrollDown => send_l(InputLocal::Down)?,
                    _ => {},
                },
                Event::Key(key) => match key.code {
                    // wasd
                    KeyCode::Char('w') => send_l(InputLocal::Up)?,
                    KeyCode::Char('a') => send_l(InputLocal::Down)?,
                    KeyCode::Char('s') => send_l(InputLocal::Left)?,
                    KeyCode::Char('d') => send_l(InputLocal::Right)?,
                    
                    KeyCode::Char('x') => send_g(InputGlobal::Previous)?,
                    KeyCode::Char('c') => send_g(InputGlobal::PlayPause)?,
                    KeyCode::Char('v') => send_g(InputGlobal::Stop)?,
                    KeyCode::Char('b') => send_g(InputGlobal::Next)?,
                    
                    KeyCode::Char('f') => send_g(InputGlobal::SkipBackward{sec: 10})?,
                    KeyCode::Char('F') => send_g(InputGlobal::SkipBackward{sec: 60})?,
                    KeyCode::Char('g') => send_g(InputGlobal::SkipForward{sec: 10})?,
                    KeyCode::Char('G') => send_g(InputGlobal::SkipForward{sec: 60})?,
                    
                    KeyCode::Char('e') => send_l(InputLocal::Select)?,
                    // vim
                    KeyCode::Char('k') => send_l(InputLocal::Up)?,
                    KeyCode::Char('j') => send_l(InputLocal::Down)?,
                    KeyCode::Char('h') => send_l(InputLocal::Left)?,
                    KeyCode::Char('l') => send_l(InputLocal::Right)?,
                    
                    KeyCode::Char('y') => send_g(InputGlobal::Previous)?,
                    KeyCode::Char('u') => send_g(InputGlobal::PlayPause)?,
                    KeyCode::Char('i') => send_g(InputGlobal::Stop)?,
                    KeyCode::Char('o') => send_g(InputGlobal::Next)?,
                    
                    KeyCode::Char('n') => send_g(InputGlobal::SkipBackward{sec: 10})?,
                    KeyCode::Char('N') => send_g(InputGlobal::SkipBackward{sec: 60})?,
                    KeyCode::Char('m') => send_g(InputGlobal::SkipForward{sec: 10})?,
                    KeyCode::Char('M') => send_g(InputGlobal::SkipForward{sec: 60})?,
                    
                    KeyCode::Enter     => send_l(InputLocal::Select)?,
                    // extra
                    KeyCode::Up        => send_l(InputLocal::Up)?,
                    KeyCode::Down      => send_l(InputLocal::Down)?,
                    KeyCode::Left      => send_l(InputLocal::Left)?,
                    KeyCode::Right     => send_l(InputLocal::Right)?,
                    
                    KeyCode::Char(' ') => send_l(InputLocal::SelectAlt)?,

                    KeyCode::PageUp    => send_l(InputLocal::PgUp)?,
                    KeyCode::PageDown  => send_l(InputLocal::PgDown)?,
                    KeyCode::Home      => send_l(InputLocal::Home)?,
                    KeyCode::End       => send_l(InputLocal::End)?,

                    KeyCode::Tab       => send_l(InputLocal::Tab)?,
                    KeyCode::BackTab   => send_l(InputLocal::RevTab)?,
                    // Exit
                    KeyCode::Esc |
                    KeyCode::Char('q') => tx.exit.send(Ok("".to_string()))?,
                    _ => (),
                },
                Event::Resize(columns, rows) => tx.state.send((
                    Instant::now(),
                    StateActions::Resize{height: rows, width: columns}
                ))?,
                _ => {},
            }
        }
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
