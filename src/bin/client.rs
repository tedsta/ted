extern crate ted;

use ted::editor::Editor;

fn main() {
    // Run our client editor
    Editor::from_file("src/ted.rs".to_string()).unwrap().run();
}
