use unicode_width::UnicodeWidthStr;


//-//////////////////////////////////////////////////////////////////
//
//-//////////////////////////////////////////////////////////////////
pub fn term_text(mut buf: String, length: usize) -> String {
    let offset = length as isize - buf.width() as isize;

    if offset < 0 {
        buf.push_str("...");
        loop {
            if buf.width() <= length {
                break;
            }
            buf.pop();
            buf.pop();
            buf.pop();
            buf.pop();
            buf.push_str("...");
        }
    } else
    if offset > 0 {
        while buf.width() < length {
            buf.push(' ');
        }
    }

    buf
}

pub fn term_text_line(pre: &str, dynamic: String, post: &str, length: usize) -> String {
    let fixed_len = pre.chars().count() + post.chars().count();
    let extra_len = length - fixed_len;

    let dynamic = term_text(dynamic, extra_len);

    let mut buf = String::with_capacity(pre.len() + dynamic.len() + post.len());
    buf.push_str(pre);
    buf.push_str(&dynamic);
    buf.push_str(post);
    buf
}

//-//////////////////////////////////////////////////////////////////
//
//-//////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_ui_text_len() {
        let text_en = "xXx yYy zZz".to_string();
        let text_jp = "平仮名, ひらがな".to_string();

        assert_eq!([
            term_text(text_en.clone(), 10),
            term_text(text_en.clone(), 11),
            term_text(text_en.clone(), 12),
            term_text(text_jp.clone(), 15),
            term_text(text_jp.clone(), 16),
            term_text(text_jp.clone(), 17),
        ],[
            "xXx yYy...".to_string(),
            "xXx yYy zZz".to_string(),
            "xXx yYy zZz ".to_string(),
            "平仮名, ひら...".to_string(),
            "平仮名, ひらがな".to_string(),
            "平仮名, ひらがな ".to_string(),
        ]);

        assert_eq!([
            term_text(text_en.clone(), 10).width(),
            term_text(text_en.clone(), 11).width(),
            term_text(text_en.clone(), 12).width(),
            term_text(text_jp.clone(), 15).width(),
            term_text(text_jp.clone(), 16).width(),
            term_text(text_jp.clone(), 17).width(),
        ],[
            10,
            11,
            12,
            15,
            16,
            17,
        ]);
    }
}

//-//////////////////////////////////////////////////////////////////
//
//-//////////////////////////////////////////////////////////////////
