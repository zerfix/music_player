use crate::globals::playback_state::PlaybackState;
use crate::globals::terminal_state::GlobalUiState;
use crate::tasks::listener_tui::RenderDataCommon;
use crate::types::types_tui::Format;
use crate::types::types_tui::TermState;
use crate::types::types_tui::Color;
use crate::ui::utils::ui_time_util::render_duration;
use std::iter::repeat;

//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
pub fn render_playback_status_widget(
    output: &mut TermState,
    common: &RenderDataCommon
) {
    let width = common.term.width as usize;
    output.format(Format{fg: Color::Black, bg: Color::Blue, bold: true});

    if width < 20 {
        output.frame.extend(repeat(' ').take(width));
        return;
    }

    let playback = &common.playback;

    output.frame.extend(repeat(' ').take(2));
    output.frame.push(playback.state.icon());
    output.frame.extend(repeat(' ').take(1));

    match playback.state {
        PlaybackState::Stopped |
        PlaybackState::Loading => {
            let progress_width = width-20;
            GlobalUiState::update_progress_width(progress_width);
            output.frame.push_str("--:--/--:-- [");
            output.frame.extend(repeat('/').take(progress_width));
            output.frame.push_str("]  ");
            //  --:--/--:-- [///////////]
        },
        PlaybackState::Paused |
        PlaybackState::Playing => {
            let elapsed  = playback.elapsed();
            let progress = playback.progress();
            let duration = playback.duration;

            // track duration total
            output.text_buf.clear();
            render_duration(&mut output.text_buf, elapsed);
            output.text_buf.push('/');
            render_duration(&mut output.text_buf, duration);

            output.frame.push_str(&output.text_buf);

            // track progress
            let status_width     = 2;
            let whitespace_width = 5;
            let progress_width   = 3;
            let duration_width   = output.text_buf.len();
            let remaining_width  = width.saturating_sub(status_width + whitespace_width + progress_width + duration_width);
            GlobalUiState::update_progress_width(remaining_width);

            let pre_progress  = ((remaining_width as f64 * progress) as usize).min(remaining_width);
            let post_progress = remaining_width - pre_progress;
            output.frame.push(' ');
            output.frame.push('[');
            output.frame.extend(repeat('━').take(pre_progress));
            output.frame.push('➤');
            output.frame.extend(repeat('⋅').take(post_progress));
            output.frame.push(']');
            output.frame.extend(repeat(' ').take(2));
            //  00:30/01:00 [━━━━━➤⋅⋅⋅⋅⋅]
        },
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
