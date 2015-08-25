#![feature(convert)]

extern crate rustbox;

use editor::Editor;

mod buffer;
mod cursor;
mod editor;
mod operation;
mod ted;

fn main() {
    Editor::from_file("src/ted.rs").unwrap().run();
}
