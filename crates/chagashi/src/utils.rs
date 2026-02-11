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
    if at >= value.graphemes(true).count() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn grapheme_len(value: &str) -> usize {
        value.graphemes(true).count()
    }

    proptest! {
        #[test]
        fn insert_then_remove_restores_original(
            value in proptest::string::string_regex("[ -~]*").expect("valid regex"),
            at in any::<usize>(),
            c in proptest::char::range(' ', '~'),
        ) {
            let len = grapheme_len(&value);
            let index = if len == 0 { 0 } else { at % (len + 1) };

            let inserted = insert_char(value.clone(), index, c);
            let removed = remove_char(inserted, index);

            prop_assert_eq!(removed, value);
        }

        #[test]
        fn split_and_concat_is_identity(
            value in proptest::string::string_regex("[ -~]*").expect("valid regex"),
            at in any::<usize>(),
        ) {
            let len = grapheme_len(&value);
            let index = if len == 0 { 0 } else { at % (len + 1) };

            let (head, tail) = split_at(value.clone(), index);
            prop_assert_eq!(format!("{head}{tail}"), value);
        }

        #[test]
        fn remove_out_of_bounds_keeps_string(
            value in proptest::string::string_regex("[ -~]*").expect("valid regex"),
            at in any::<usize>(),
        ) {
            let len = grapheme_len(&value);
            let index = len.saturating_add(at);

            let removed = remove_char(value.clone(), index);
            prop_assert_eq!(removed, value);
        }
    }
}
