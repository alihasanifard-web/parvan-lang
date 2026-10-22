/// Bidi & Arabic/Persian Reshaper for ParvanLang
/// Converts Persian/Arabic strings into properly shaped, visual RTL order for terminal output.

pub fn reshape_persian(text: &str) -> String {
    // If text has no Persian/Arabic characters, return as is
    if !text.chars().any(is_pure_persian_arabic) {
        return text.to_string();
    }

    let words: Vec<&str> = text.split(' ').collect();
    let mut shaped_words = Vec::new();

    for word in words {
        if word.chars().any(is_pure_persian_arabic) {
            let shaped = shape_arabic_words(word);
            let reversed: String = shaped.chars().rev().collect();
            shaped_words.push(reversed);
        } else {
            shaped_words.push(word.to_string());
        }
    }

    // Reverse word order for full sentence RTL in LTR terminal
    shaped_words.reverse();
    shaped_words.join(" ")
}

fn is_pure_persian_arabic(ch: char) -> bool {
    matches!(ch as u32,
        0x0600..=0x06FF | // Arabic
        0x0750..=0x077F | // Arabic Supplement
        0x08A0..=0x08FF | // Arabic Extended-A
        0xFB50..=0xFDFF | // Arabic Presentation Forms-A
        0xFE70..=0xFEFF   // Arabic Presentation Forms-B
    ) && ch != ' ' && ch != '\u{200C}'
}

#[allow(dead_code)]
fn is_persian_arabic_char(ch: char) -> bool {
    matches!(ch as u32,
        0x0600..=0x06FF | // Arabic
        0x0750..=0x077F | // Arabic Supplement
        0x08A0..=0x08FF | // Arabic Extended-A
        0xFB50..=0xFDFF | // Arabic Presentation Forms-A
        0xFE70..=0xFEFF   // Arabic Presentation Forms-B
    )
}

struct CharForms {
    isolated: char,
    initial: char,
    medial: char,
    final_: char,
}

fn get_char_forms(ch: char) -> Option<CharForms> {
    match ch {
        'آ' => Some(CharForms { isolated: '\u{FE81}', initial: '\u{FE81}', medial: '\u{FE82}', final_: '\u{FE82}' }),
        'ا' => Some(CharForms { isolated: '\u{FE8D}', initial: '\u{FE8D}', medial: '\u{FE8E}', final_: '\u{FE8E}' }),
        'ب' => Some(CharForms { isolated: '\u{FE8F}', initial: '\u{FE91}', medial: '\u{FE92}', final_: '\u{FE90}' }),
        'پ' => Some(CharForms { isolated: '\u{FB56}', initial: '\u{FB58}', medial: '\u{FB59}', final_: '\u{FB57}' }),
        'ت' => Some(CharForms { isolated: '\u{FE95}', initial: '\u{FE97}', medial: '\u{FE98}', final_: '\u{FE96}' }),
        'ث' => Some(CharForms { isolated: '\u{FE99}', initial: '\u{FE9B}', medial: '\u{FE9C}', final_: '\u{FE9A}' }),
        'ج' => Some(CharForms { isolated: '\u{FE9D}', initial: '\u{FE9F}', medial: '\u{FEA0}', final_: '\u{FE9E}' }),
        'چ' => Some(CharForms { isolated: '\u{FB7A}', initial: '\u{FB7C}', medial: '\u{FB7D}', final_: '\u{FB7B}' }),
        'ح' => Some(CharForms { isolated: '\u{FEA1}', initial: '\u{FEA3}', medial: '\u{FEA4}', final_: '\u{FEA2}' }),
        'خ' => Some(CharForms { isolated: '\u{FEA5}', initial: '\u{FEA7}', medial: '\u{FEA8}', final_: '\u{FEA6}' }),
        'د' => Some(CharForms { isolated: '\u{FEA9}', initial: '\u{FEA9}', medial: '\u{FEAA}', final_: '\u{FEAA}' }),
        'ذ' => Some(CharForms { isolated: '\u{FEAB}', initial: '\u{FEAB}', medial: '\u{FEAC}', final_: '\u{FEAC}' }),
        'ر' => Some(CharForms { isolated: '\u{FEAD}', initial: '\u{FEAD}', medial: '\u{FEAE}', final_: '\u{FEAE}' }),
        'ز' => Some(CharForms { isolated: '\u{FEAF}', initial: '\u{FEAF}', medial: '\u{FEB0}', final_: '\u{FEB0}' }),
        'ژ' => Some(CharForms { isolated: '\u{FB8A}', initial: '\u{FB8A}', medial: '\u{FB8B}', final_: '\u{FB8B}' }),
        'س' => Some(CharForms { isolated: '\u{FEB1}', initial: '\u{FEB3}', medial: '\u{FEB4}', final_: '\u{FEB2}' }),
        'ش' => Some(CharForms { isolated: '\u{FEB5}', initial: '\u{FEB7}', medial: '\u{FEB8}', final_: '\u{FEB6}' }),
        'ص' => Some(CharForms { isolated: '\u{FEB9}', initial: '\u{FEBB}', medial: '\u{FEBC}', final_: '\u{FEBA}' }),
        'ض' => Some(CharForms { isolated: '\u{FEBD}', initial: '\u{FEBF}', medial: '\u{FEC0}', final_: '\u{FEBE}' }),
        'ط' => Some(CharForms { isolated: '\u{FEC1}', initial: '\u{FEC3}', medial: '\u{FEC4}', final_: '\u{FEC2}' }),
        'ظ' => Some(CharForms { isolated: '\u{FEC5}', initial: '\u{FEC7}', medial: '\u{FEC8}', final_: '\u{FEC6}' }),
        'ع' => Some(CharForms { isolated: '\u{FEC9}', initial: '\u{FECB}', medial: '\u{FECC}', final_: '\u{FECA}' }),
        'غ' => Some(CharForms { isolated: '\u{FECD}', initial: '\u{FECF}', medial: '\u{FED0}', final_: '\u{FECE}' }),
        'ف' => Some(CharForms { isolated: '\u{FED1}', initial: '\u{FED3}', medial: '\u{FED4}', final_: '\u{FED2}' }),
        'ق' => Some(CharForms { isolated: '\u{FED5}', initial: '\u{FED7}', medial: '\u{FED8}', final_: '\u{FED6}' }),
        'ک' | 'ك' => Some(CharForms { isolated: '\u{FED9}', initial: '\u{FEDB}', medial: '\u{FEDC}', final_: '\u{FEDA}' }),
        'گ' => Some(CharForms { isolated: '\u{FB92}', initial: '\u{FB94}', medial: '\u{FB95}', final_: '\u{FB93}' }),
        'ل' => Some(CharForms { isolated: '\u{FEDD}', initial: '\u{FEDF}', medial: '\u{FEE0}', final_: '\u{FEDE}' }),
        'م' => Some(CharForms { isolated: '\u{FEE1}', initial: '\u{FEE3}', medial: '\u{FEE4}', final_: '\u{FEE2}' }),
        'ن' => Some(CharForms { isolated: '\u{FEE5}', initial: '\u{FEE7}', medial: '\u{FEE8}', final_: '\u{FEE6}' }),
        'و' => Some(CharForms { isolated: '\u{FEED}', initial: '\u{FEED}', medial: '\u{FEEE}', final_: '\u{FEEE}' }),
        'ه' => Some(CharForms { isolated: '\u{FEE9}', initial: '\u{FEEB}', medial: '\u{FEEC}', final_: '\u{FEEA}' }),
        'ی' | 'ي' => Some(CharForms { isolated: '\u{FBFC}', initial: '\u{FBFE}', medial: '\u{FBFF}', final_: '\u{FBFD}' }),
        'ئ' => Some(CharForms { isolated: '\u{FE89}', initial: '\u{FE8B}', medial: '\u{FE8C}', final_: '\u{FE8A}' }),
        _ => None,
    }
}

fn can_connect_after(ch: char) -> bool {
    !matches!(ch, 'ا' | 'آ' | 'د' | 'ذ' | 'ر' | 'ز' | 'ژ' | 'و' | ' ' | '\u{200C}')
}

fn can_connect_before(ch: char) -> bool {
    if ch == ' ' || ch == '\u{200C}' {
        return false;
    }
    // Chars that don't connect from before (none in Persian, all accept connection from prev except after non-connectors)
    get_char_forms(ch).is_some()
}

fn shape_arabic_words(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut shaped = String::new();

    for i in 0..n {
        let ch = chars[i];
        if let Some(forms) = get_char_forms(ch) {
            let prev_connects = if i > 0 {
                can_connect_after(chars[i - 1]) && can_connect_before(ch)
            } else {
                false
            };

            let next_connects = if i + 1 < n {
                can_connect_after(ch) && can_connect_before(chars[i + 1])
            } else {
                false
            };

            let chosen = match (prev_connects, next_connects) {
                (true, true) => forms.medial,
                (true, false) => forms.final_,
                (false, true) => forms.initial,
                (false, false) => forms.isolated,
            };
            shaped.push(chosen);
        } else {
            shaped.push(ch);
        }
    }

    shaped
}
