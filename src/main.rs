#![feature(convert)]

extern crate rustbox;

use editor::Editor;

mod buffer;
mod editor;
mod operation;
mod ted;

fn main() {
    Editor::new().run();
}
