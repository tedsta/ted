use buffer::Buffer;
use operation::Operation;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Cursor {
    pub line: u64,
    pub column: u64,
    pub buf_index: u64,
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
        if self.line < (buffer.line_count()-1) as u64 {
            self.line += 1;
        }

        self.calculate_index(buffer);
    }

    /// Moves the cursor left and returns the new index within the buffer
    pub fn move_left(&mut self, buffer: &Buffer) {
        if self.column > 0 {
            // Cursor can move to the left, need to determine if it's past the current line, though.
            if self.column < buffer.line_info()[self.line as usize].length as u64 {
                self.column -= 1;
            } else if buffer.line_info()[self.line as usize].length >= 2 {
                self.column = (buffer.line_info()[self.line as usize].length - 2) as u64;
            } else if self.line > 0 {
                // column = 0 or 1, so cursor is at the beginning of the line, move to previous line
                self.line -= 1;
                if buffer.line_info()[self.line as usize].length > 0 {
                    self.column = (buffer.line_info()[self.line as usize].length - 1) as u64;
                } else {
                    self.column = 0;
                }
            }
        } else if self.line > 0 {
            // Cursor is at the beginning of the line, move to previous line
            self.line -= 1;
            if buffer.line_info()[self.line as usize].length > 0 {
                self.column = (buffer.line_info()[self.line as usize].length - 1) as u64;
            } else {
                self.column = 0;
            }
        }

        self.calculate_index(buffer);
    }

    /// Moves the cursor right and returns the new index within the buffer
    pub fn move_right(&mut self, buffer: &Buffer) {
        if buffer.line_info()[self.line as usize].length > 0 &&
           self.column < (buffer.line_info()[self.line as usize].length - 1) as u64 {
            // Cursor can move to the right
            self.column += 1;
        } else if self.line < (buffer.line_count() - 1) as u64 {
            // Cursor can't move right, move to next line
            self.line += 1;
            self.column = 0;
        }

        self.calculate_index(buffer);
    }

    /// Calculates the position to display the cursor at
    pub fn get_display_xy(&self, buffer: &Buffer) -> (u64, u64) {
        let line_info = buffer.line_info()[self.line as usize];

        (self.buf_index - (line_info.buf_index as u64), self.line)
    }

    /// Moves the cursor right and returns the new index within the buffer
    pub fn op_adjust_cursor(&mut self, buffer: &Buffer, op: &Operation) {
        match *op {
            Operation::InsertChar(index, c) => {
                // Insertion only pushes cursor forward if it happened behind or on the cursor
                if index <= self.buf_index {
                    if c == '\n' {
                        self.line += 1;
                    }
                    self.buf_index += 1;
                    self.calculate_column(buffer);
                }
            },
            Operation::Insert(index, ref text) => {
                // Insertion only pushes cursor forward if it happened behind or on the cursor
                if index <= self.buf_index {
                    let newline_count = text.chars().filter(|c| *c == '\n').count();
                    self.line += newline_count as u64;
                    self.buf_index += text.len() as u64;
                    self.calculate_column(buffer);
                }
            },
            Operation::RemoveChar(index, c) => {
                // Remove only pulls cursor backward if it happened behind or on the cursor
                if index <= self.buf_index && self.buf_index > 0 {
                    if c == '\n' {
                        // Handle special newline case
                        self.line -= 1;
                    }
                    self.buf_index -= 1;
                    self.calculate_column(buffer);
                }
            },
            Operation::Remove(start, end, ref text) => {
                // Remove only pulls cursor backward if it happened behind or on the cursor
                if start <= self.buf_index && self.buf_index > 0 {
                    if end < self.buf_index {
                        let newline_count = text.chars().filter(|c| *c == '\n').count();
                        self.line -= newline_count as u64;
                        self.buf_index -= text.len() as u64;
                        self.calculate_column(buffer);
                    } else {
                        let newline_count =
                            text[0..(self.buf_index-start) as usize].chars()
                                                                    .filter(|c| *c == '\n')
                                                                    .count();
                        self.line -= newline_count as u64;
                        self.buf_index = start;
                        self.calculate_column(buffer);
                    }
                }
            },
        }
    }

    /// Calculates the cursor's index within the specified buffer based on line and column
    fn calculate_index(&mut self, buffer: &Buffer) {
        use std::cmp;

        let line_info = buffer.line_info()[self.line as usize];
        self.buf_index =
            if line_info.length > 0 {
                line_info.buf_index + cmp::min(line_info.length-1, self.column as usize)
            } else {
                line_info.buf_index
            } as u64;
    }

    /// Calculates the cursor's column within the specified buffer based on line and buf_index
    fn calculate_column(&mut self, buffer: &Buffer) {
        let line_info = buffer.line_info()[self.line as usize];
        self.column = self.buf_index - line_info.buf_index as u64;
    }
}
