use anyhow::Error;
use crate::MsgChannels;
use crate::types::types_library_entry::LibraryTrackEntryData;
use rodio::Decoder;
use rodio::OutputStream;
use rodio::Sink;
use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub enum PlaybackActions {
    AppendTrack{track: LibraryTrackEntryData},
    PlayPause,
    Stop,
    StopAndClear,
    GetPlaylist{callback: Sender<Vec<LibraryTrackEntryData>>},
    Exit,
}

/// Handles playback requests
pub fn start_playback_task(rx: Receiver<PlaybackActions>, tx: MsgChannels) {
    if let Err(err) = playback_loop(rx) {
        warn!("error: {:?}", err)
    }

    info!("exit");
    tx.tx_exit.send(()).unwrap();
}

fn playback_loop(rx: Receiver<PlaybackActions>) -> Result<(), Error> {
    info!("playback: init");

    let mut playlist: VecDeque<LibraryTrackEntryData> = VecDeque::new();

    // Get a output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let sink = Sink::try_new(&stream_handle).unwrap();

    info!("sink initialized");

    while let Ok(msg) = rx.recv() {
        match msg {
            PlaybackActions::AppendTrack{track} => {
                playlist.push_back(track.clone());
                // Load a sound from a file, using a path relative to Cargo.toml
                let file = BufReader::new(File::open(track.path).unwrap());
                // Decode that sound file into a source
                let source = Decoder::new(file).unwrap();
                // Play the sound directly on the device
                sink.append(source);
            },
            PlaybackActions::PlayPause => match sink.is_paused() {
                true => sink.play(),
                false => sink.pause(),
            },
            PlaybackActions::Stop => sink.stop(),
            PlaybackActions::StopAndClear => {
                playlist = VecDeque::new();
                sink.clear()
            },
            PlaybackActions::GetPlaylist{callback} => {
                let sink_len = sink.len();
                debug_assert!(sink_len <= playlist.len());
                while sink_len < playlist.len() {
                    playlist.pop_front();
                }
                callback.send(playlist.iter().cloned().collect()).unwrap();
            },
            PlaybackActions::Exit => break,
        }
    };
    Ok(())
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
