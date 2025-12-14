<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./logo_white.svg">
    <img alt="matcha-rs logo" src="./logo_black.svg" width="240">
  </picture>
</div>

<div align="center">
  A TEA framework for building TUI apps in Rust<br>
  inspired by <a href="https://github.com/charmbracelet/bubbletea">charmbracelet/bubbletea</a>.
</div>

---

<a href="https://github.com/bokuweb/matcha-rs/actions/workflows/ci.yml">
  <img alt="CI" src="https://github.com/bokuweb/matcha-rs/actions/workflows/ci.yml/badge.svg">
</a>
   
This repository is a Cargo workspace composed of two crates:

- **`matcha-rs`**: the core runtime (event loop, input handling, command execution, rendering)
- **`chagashi`**: UI components built on top of `matcha`

Name origin:

- **matcha**: Japanese tea
- **chagashi**: sweets served alongside matcha

---

## Concepts (TEA)

`matcha` implements the core TEA building blocks in a Rust-friendly way:

- **Model**: state + behavior (`Model` trait)
- **Msg**: input / IO results (`Msg = Box<dyn Any + Send>`, handled via downcasting)
- **Cmd**: IO and async work (`Cmd::sync` / `Cmd::async`)
- **View**: returns a `Display` (rendered after every update)

`Program` drives the event loop and forwards crossterm events (key/mouse/resize) as `Msg` into `Model::update`.

---

## crates

### `matcha-rs` (core package, `matcha` crate)

Key features:

- **Event loop / rendering**: `Program`
- **Commands**: `Cmd` (sync/async), `batch` (run multiple commands together)
- **Utilities**: `tick`, `quit`
- **Input**: re-exports from crossterm (e.g. `KeyEvent`)
- **Key bindings**: re-exports `crokey`; use `KeyBindings` / `key!()` declaratively
- **Extensions**: a type-safe container keyed by `TypeId` (useful for DI / shared context)

### `chagashi` (components)

Reusable UI components that can be composed as `matcha::Model`s:

- **`textarea`**: multi-line editor
- **`textinput`**: single-line input
- **`viewport`**: scrolling / selection view
- **`list`**: list UI (customizable rendering via a delegate)
- **`spinner`**: animated spinner (via ticking)
- **`border` / `borderize`**: borders / framing
- **`flex`**: a flexbox-inspired layout container (row/column, wrapping)

---

## Quick start

Run an example from this repository:

```bash
cargo run -p matcha --example textinput
```

## Examples

```bash
cargo run -p matcha --example async
cargo run -p matcha --example flex
cargo run -p matcha --example hello
cargo run -p matcha --example simple
cargo run -p matcha --example textarea
cargo run -p matcha --example textinput
cargo run -p matcha --example viewport
```

### Example: `async`

Async command example. The program starts an async task on init, waits 3 seconds, then updates the view. Press any key to quit.

```rust
use std::fmt::Display;

use matcha::{
    quit, style, AsyncCmd, Cmd, Extensions, InitInput, KeyEvent, Model, Msg, Program, Stylize,
};

pub fn init() -> Msg {
    Box::new(AsyncMsg) as Msg
}

pub fn done() -> Msg {
    Box::new(DoneMsg) as Msg
}

pub struct AsyncMsg;
pub struct DoneMsg;

struct App {
    done: bool,
}

#[async_trait::async_trait]
impl Model for App {
    fn init(self, _input: &InitInput) -> (Self, Option<Cmd>) {
        (self, Some(matcha::r#async!(init())))
    }

    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if msg.downcast_ref::<KeyEvent>().is_some() {
            return (self, Some(matcha::sync!(quit())));
        }
        if msg.downcast_ref::<DoneMsg>().is_some() {
            return (Self { done: true }, None);
        }
        (self, None)
    }

    fn view(&self) -> impl Display {
        if self.done {
            style("Completed.").negative().to_string()
        } else {
            style("Waiting for the completion of an async task.")
                .negative()
                .to_string()
        }
    }

    async fn execute(_ext: Extensions, AsyncCmd(cmd): AsyncCmd) -> Option<Cmd> {
        let msg = cmd();
        if msg.downcast_ref::<AsyncMsg>().is_some() {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            return Some(matcha::sync!(done()));
        }
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let p = Program::new(App { done: false }, Extensions::default());
    p.start().await.unwrap();
    Ok(())
}
```

### Example: `flex`

Flex layout example (from `chagashi::Flex`). This example runs in alt-screen mode for stable redraw during resize.

```bash
cargo run -p matcha --example flex
```

### Example: `textinput`

Single-line input powered by `chagashi::textinput`. Press `Esc` to quit.

```rust
use std::fmt::Display;

use chagashi::textinput::TextInput;
use matcha::{quit, Cmd, Extensions, InitInput, KeyCode, KeyEvent, Model, Msg, Program};

struct App {
    input: TextInput,
}

impl Model for App {
    fn init(self, _input: &InitInput) -> (Self, Option<Cmd>) {
        let (input, cmd) = self.input.focus();
        (Self { input }, cmd)
    }

    fn update(self, msg: &Msg) -> (Self, Option<Cmd>) {
        if let Some(msg) = msg.downcast_ref::<KeyEvent>() {
            if msg.code == KeyCode::Esc {
                return (self, Some(matcha::sync!(quit())));
            };
        };

        let (input, cmd) = self.input.update(msg);
        (Self { input }, cmd)
    }

    fn view(&self) -> impl Display {
        "What’s your favorite language\n".to_string()
            + "\n"
            + &format!("{}", self.input.view())
            + "\n"
            + "\n"
            + "(esc to quit)"
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let input = TextInput::new().set_placeholder("Rust");
    let p = Program::new(App { input }, Extensions::default());
    p.start().await.unwrap();
    Ok(())
}
```

The Rust toolchain is pinned via `rust-toolchain.toml` (currently `1.83.0`).

---

## Use from another project (git dependency)

Since this is a Cargo workspace, you can depend on `matcha` / `chagashi` individually via git:

```toml
[dependencies]
matcha = { git = "https://github.com/bokuweb/matcha-rs", package = "matcha-rs" }
chagashi = { git = "https://github.com/bokuweb/matcha-rs", package = "chagashi" }
```

---

## Development

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

GitHub Actions also runs `fmt / clippy / build / test` on an OS matrix (Ubuntu/macOS/Windows).

---

## License

See `LICENSE`.
