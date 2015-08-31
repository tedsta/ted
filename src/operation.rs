use cursor::Cursor;
use self::Operation::*;

#[derive(Clone, PartialEq, Eq, RustcEncodable, RustcDecodable)]
pub enum Operation {
    InsertChar(usize, char),
    Insert(usize, String),
    RemoveChar(usize, char),
    Remove(usize, usize, String),
}

impl Operation {
    pub fn inverse(self) -> Operation {
        match self {
            InsertChar(index, c) => RemoveChar(index, c),
            Insert(index, text) => Remove(index, index+text.len()-1, text),
            RemoveChar(index, c) => InsertChar(index, c),
            Remove(start, _, text) => Insert(start, text),
        }
    }

    pub fn do_before(&self, mut op: Operation) -> Option<Operation> {
        let (op_start, op_end, bias): (usize, usize, isize) =
            match *self {
                InsertChar(index, _) => {
                    (index, index, 1)
                },
                Insert(index, ref text) => {
                    (index, index + text.len(), text.len() as isize)
                },
                RemoveChar(index, _) => {
                    (index, index, -1)
                },
                Remove(start, end, ref text) => {
                    (start, end, -(text.len() as isize))
                },
            };

        match op {
            InsertChar(ref mut index, _) => {
                if *index >= op_start && *index <= op_end && bias < 0 {
                    *index = op_start;
                } else if *index > op_start {
                    *index = ((*index as isize) + bias) as usize;
                }
            },
            Insert(ref mut index, _) => {
                if *index >= op_start && *index <= op_end && bias < 0 {
                    *index = op_start;
                } else if *index > op_start {
                    *index = ((*index as isize) + bias) as usize;
                }
            },
            RemoveChar(ref mut index, _) => {
                if *index >= op_start && *index <= op_end && bias < 0 {
                    return None;
                } else if *index > op_start {
                    *index = ((*index as isize) + bias) as usize;
                }
            },
            Remove(ref mut start, ref mut end, _) => {
                if *end >= op_start && *start <= op_end && bias < 0 {
                    return None;
                } else if *start > op_start {
                    *start = ((*start as isize) + bias) as usize;
                    *end = ((*end as isize) + bias) as usize;
                }
            },
        }

        Some(op)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Dear got the unit tests :( don't wanna write

#[test]
fn do_before_removed_same() {
    let before = Remove(0, 10, "asdfghjklz".to_string());
    let after = Remove(0, 10, "asdfghjklz".to_string());

    assert!(before.do_before(after) == None);
}
