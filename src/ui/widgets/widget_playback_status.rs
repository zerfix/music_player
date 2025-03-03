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
    let width = common.term_size.width;
    output.format(Format{fg: Color::Black, bg: Color::Blue, bold: false});

    if width < 20 {
        output.frame.extend(repeat(' ').take(width));
        return;
    }

    match common.playlist.playback_progress() {
        None => {
            output.frame.push_str("⏹  --:--/--:-- [");
            output.frame.extend(repeat('/').take(width-18));
            output.frame.push_str("] "); 
        },
        Some((playing, elapsed, progress, total)) => {
            // playback status
            match playing {
                false => output.frame.push_str("⏸  "),
                true  => output.frame.push_str("⏵  "),
            }

            // track duration total
            output.text_buf.clear();
            render_duration(&mut output.text_buf, elapsed);
            output.text_buf.push('/');
            render_duration(&mut output.text_buf, total);

            output.frame.push_str(&output.text_buf);

            // track progress
            let status_width     = 2;
            let whitespace_width = 3;
            let progress_width   = 3;
            let duration_width   = output.text_buf.len();
            let remaining_width  = width.saturating_sub(status_width + whitespace_width + progress_width + duration_width);
        
            let pre_progress  = ((remaining_width as f64 * progress) as usize).min(remaining_width);
            let post_progress = remaining_width - pre_progress;

            output.frame.push(' ');
            output.frame.push('[');
            output.frame.extend(repeat('⎯').take(pre_progress));
            output.frame.push('●');
            output.frame.extend(repeat('-').take(post_progress));
            output.frame.push(']');
            output.frame.push(' ');
        },
    }
}
//-////////////////////////////////////////////////////////////////////////////
//
//-////////////////////////////////////////////////////////////////////////////
