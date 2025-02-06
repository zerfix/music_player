use std::iter::repeat;
use unicode_width::UnicodeWidthChar;


//-//////////////////////////////////////////////////////////////////
//
//-//////////////////////////////////////////////////////////////////
///// Formats text to fit target number of cells in terminal.
pub fn term_text(mut buf: String, target: usize) -> String {
    let width = buf.chars().map(|c| c.width().unwrap_or(0)).sum::<usize>();
    let diff  = target as isize - width as isize;

    match (target, diff) {
        (_  ,  0 ) => {},
        (_  , 1..) => buf.extend(repeat(' ').take(diff as usize)),
        (..4, ..0) => buf = ".".repeat(target),
        (4.., ..0) => {
            let iterator = buf.chars().map(|c| (c.len_utf8(), c.width().unwrap_or(0)));
            let mut current = 0;
            let mut remaining = target - 3;
            for (length, width) in iterator {
                match width <= remaining {
                    false => break,
                    true => {
                        current   += length;
                        remaining -= width;
                    },
                }
            }

            buf.truncate(current);
            buf.push_str("...");
            buf.extend(repeat(' ').take(remaining));
        }
    }

    buf
}

//-//////////////////////////////////////////////////////////////////
//
//-//////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use unicode_width::UnicodeWidthStr;

    struct TextTest {
        pub reference: &'static str,
        pub ref_len: usize,
        pub tests: Vec<TextTestGroup>,
    }

    struct TextTestGroup {
        pub target_len: usize,
        pub result: &'static str,
    }

    #[test]
    fn test_ui_text_len() {
        [
            TextTest{
                reference:"xXx yYy zZz", ref_len: 11, tests: vec![
                     TextTestGroup{target_len:10,result: "xXx yYy..."  },
                     TextTestGroup{target_len:12,result: "xXx yYy zZz "},
                ],
            },
            TextTest{
                reference:"平仮名, ひらがな", ref_len:16, tests: vec![
                    TextTestGroup{target_len:14,result: "平仮名, ひ... "   },
                    TextTestGroup{target_len:15,result: "平仮名, ひら..."  },
                    TextTestGroup{target_len:17,result: "平仮名, ひらがな "}
                ],
            },
            TextTest{
                reference: "Novak 監視™", ref_len: 11, tests: vec![
                    TextTestGroup{target_len:09,result: "Novak ..."   },
                    TextTestGroup{target_len:10,result: "Novak ... "  },
                    TextTestGroup{target_len:12,result: "Novak 監視™ "},
                ],
            },
            TextTest{
                reference: "ラン", ref_len: 4, tests: vec![
                    TextTestGroup{target_len:0,result: ""     },
                    TextTestGroup{target_len:1,result: "."    },
                    TextTestGroup{target_len:2,result: ".."   },
                    TextTestGroup{target_len:3,result: "..."  },
                    TextTestGroup{target_len:5,result: "ラン "},
                ],
            },
        ].iter()
        .for_each(|test: &TextTest| {
            println!("Testing: {}", test.reference);
            let reference = test.reference.to_string();
            {
                let text = term_text(reference.clone(), test.ref_len);
                assert_eq!(text, reference);
                assert_eq!(text.width(), test.ref_len);
            }
            test.tests.iter()
            .for_each(|sub_test| {
                let text = term_text(reference.clone(), sub_test.target_len);
                println!("{}, {}, {}", text, text.len(), text.width());
                assert_eq!(text, sub_test.result.to_string());
                assert_eq!(text.width(), sub_test.target_len);
            });
        });
    }
}

//-//////////////////////////////////////////////////////////////////
//
//-//////////////////////////////////////////////////////////////////
