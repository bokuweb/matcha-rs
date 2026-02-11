#![no_main]
#![allow(unused_mut)]

use libfuzzer_sys::{arbitrary::Arbitrary, fuzz_target};

use matcha::*;

#[derive(Debug)]
struct Fuzzer(String);

#[derive(Debug, Clone, Copy)]
enum Op {
    MoveLeft,
    MoveRight,
    DeleteForwardChar,
    DeleteBackChar,
}

impl<'a> Arbitrary<'a> for Fuzzer {
    fn arbitrary(
        u: &mut libfuzzer_sys::arbitrary::Unstructured<'a>,
    ) -> libfuzzer_sys::arbitrary::Result<Self> {
        let s = u.arbitrary::<String>()?;
        Ok(Fuzzer(s))
    }
}

fuzz_target!(|data: Fuzzer| textinput_check(data.0));

fn key(c: char) -> Msg {
    Box::new(KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty()))
}

fn textinput_check(text: String) {
    let init = chagashi::textinput::TextInput::new()
        .set_placeholder("Rust")
        .focus();
    let mut text_input = init.0;
    let ops = [
        Op::MoveLeft,
        Op::MoveRight,
        Op::DeleteBackChar,
        Op::DeleteForwardChar,
    ];
    for c in text.chars() {
        let res = text_input.update(&key(c));
        let op = ops[c as usize % 4];
        text_input = match op {
            Op::DeleteBackChar => res.0.delete_back_char(),
            Op::DeleteForwardChar => res.0.delete_forward_char(),
            Op::MoveLeft => res.0.move_left(),
            Op::MoveRight => res.0.move_right(),
        };
    }
}
