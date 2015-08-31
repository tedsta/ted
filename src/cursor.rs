use buffer::Buffer;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Cursor {
    pub line: usize,
    pub column: usize,
    pub buf_index: usize,
}

impl Cursor {
    /// Moves the cursor up and returns the new index within the buffer
    pub fn move_up(&mut self, buffer: &Buffer) {
        if self.line > 0 {
            self.line -= 1;
        }

        self.calculate_index(buffer);
    }
    
    /// Moves the cursor down and returns the new index within the buffer
    pub fn move_down(&mut self, buffer: &Buffer) {
        if self.line < buffer.line_count()-1 {
            self.line += 1;
        }

        self.calculate_index(buffer);
    }

    /// Moves the cursor left and returns the new index within the buffer
    pub fn move_left(&mut self, buffer: &Buffer) {
        if self.column > 0 {
            // Cursor can move to the left, need to determine if it's past the current line, though.
            if self.column < buffer.line_info()[self.line].length {
                self.column -= 1;
            } else if buffer.line_info()[self.line].length >= 2 {
                self.column = buffer.line_info()[self.line].length - 2;
            } else if self.line > 0 {
                // column = 0 or 1, so cursor is at the beginning of the line, move to previous line
                self.line -= 1;
                if buffer.line_info()[self.line].length > 0 {
                    self.column = buffer.line_info()[self.line].length - 1;
                } else {
                    self.column = 0;
                }
            }
        } else if self.line > 0 {
            // Cursor is at the beginning of the line, move to previous line
            self.line -= 1;
            if buffer.line_info()[self.line].length > 0 {
                self.column = buffer.line_info()[self.line].length - 1;
            } else {
                self.column = 0;
            }
        }

        self.calculate_index(buffer);
    }

    /// Moves the cursor right and returns the new index within the buffer
    pub fn move_right(&mut self, buffer: &Buffer) {
        if buffer.line_info()[self.line].length > 0 &&
           self.column < buffer.line_info()[self.line].length - 1 {
            // Cursor can move to the right
            self.column += 1;
        } else if self.line < buffer.line_count() - 1 {
            // Cursor can't move right, move to next line
            self.line += 1;
            self.column = 0;
        }

        self.calculate_index(buffer);
    }

    /// Calculates the position to display the cursor at
    pub fn get_display_xy(&self, buffer: &Buffer) -> (usize, usize) {
        let line_info = buffer.line_info()[self.line];

        (self.buf_index - line_info.buf_index , self.line)
    }

    /// Calculates the cursor's index within the specified buffer
    fn calculate_index(&mut self, buffer: &Buffer) {
        use std::cmp;

        let line_info = buffer.line_info()[self.line];
        self.buf_index =
            if line_info.length > 0 {
                line_info.buf_index + cmp::min(line_info.length-1, self.column)
            } else {
                line_info.buf_index
            };
    }
}
