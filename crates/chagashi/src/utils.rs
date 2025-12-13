use unicode_segmentation::UnicodeSegmentation;

#[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
pub(crate) fn insert_char(value: String, at: usize, c: char) -> String {
    let len = value.graphemes(true).count();
    if at >= len {
        let mut v = value;
        v.push(c);
        return v;
    }

    let mut result: String = String::new();
    for (index, char) in value.graphemes(true).enumerate() {
        if index == at {
            result.push(c);
        }
        result += char;
    }
    result
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
pub(crate) fn remove_char(value: String, at: usize) -> String {
    let mut result: String = String::new();
    for (index, char) in value.graphemes(true).enumerate() {
        if index == at {
            continue;
        }
        result += char;
    }
    result
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
pub(crate) fn split_at(value: String, at: usize) -> (String, String) {
    if at >= value.len() {
        return (value, String::default());
    }

    let mut head: String = String::new();
    let mut tail: String = String::new();

    for (index, char) in value.graphemes(true).enumerate() {
        if index >= at {
            tail += char;
        } else {
            head += char;
        }
    }
    (head, tail)
}
