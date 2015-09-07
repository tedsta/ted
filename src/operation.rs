use cursor::Cursor;
use self::Operation::*;

#[derive(Clone, PartialEq, Eq, RustcEncodable, RustcDecodable)]
pub enum Operation {
    InsertChar(u64, char),
    Insert(u64, String),
    RemoveChar(u64, char),
    Remove(u64, u64, String),
}

impl Operation {
    pub fn inverse(self) -> Operation {
        match self {
            InsertChar(index, c) => RemoveChar(index, c),
            Insert(index, text) => Remove(index, index+((text.len()-1) as u64), text),
            RemoveChar(index, c) => InsertChar(index, c),
            Remove(start, _, text) => Insert(start, text),
        }
    }

    pub fn get_coords(&self) -> OpCoords {
        match *self {
            InsertChar(index, _) => OpCoords::InsertChar(index),
            Insert(index, _) => OpCoords::Insert(index),
            RemoveChar(index, _) => OpCoords::RemoveChar(index),
            Remove(start, end, _) => OpCoords::Remove(start, end),
        }
    }

    pub fn set_coords(&mut self, coords: &OpCoords) {
        match *self {
            InsertChar(ref mut index, _) => {
                match *coords {
                    OpCoords::InsertChar(new_index) => {
                    },
                    _ => { panic!("Tried to set coords with non-matching coords"); },
                }
            },
            Insert(ref mut index, _) => {
                match *coords {
                    OpCoords::Insert(new_index) => {
                    },
                    _ => { panic!("Tried to set coords with non-matching coords"); },
                }
            },
            RemoveChar(ref mut index, _) => {
                match *coords {
                    OpCoords::RemoveChar(new_index) => {
                    },
                    _ => { panic!("Tried to set coords with non-matching coords"); },
                }
            },
            Remove(ref mut start, ref mut end, _) => {
                match *coords {
                    OpCoords::Remove(new_start, new_end) => {
                    },
                    _ => { panic!("Tried to set coords with non-matching coords"); },
                }
            },
        }
    }

    pub fn do_before(&self, mut op: &mut Operation) -> bool {
        let (op_start, op_end, bias): (u64, u64, i64) =
            match *self {
                InsertChar(index, _) => {
                    (index, index, 1)
                },
                Insert(index, ref text) => {
                    (index, index + (text.len() as u64), text.len() as i64)
                },
                RemoveChar(index, _) => {
                    (index, index, -1)
                },
                Remove(start, end, ref text) => {
                    (start, end, -(text.len() as i64))
                },
            };

        match *op {
            InsertChar(ref mut index, _) => {
                if *index >= op_start && *index <= op_end && bias < 0 {
                    *index = op_start;
                } else if *index > op_start {
                    *index = ((*index as i64) + bias) as u64;
                }
            },
            Insert(ref mut index, _) => {
                if *index >= op_start && *index <= op_end && bias < 0 {
                    *index = op_start;
                } else if *index > op_start {
                    *index = ((*index as i64) + bias) as u64;
                }
            },
            RemoveChar(ref mut index, _) => {
                if *index >= op_start && *index <= op_end && bias < 0 {
                    return false;
                } else if *index > op_start {
                    *index = ((*index as i64) + bias) as u64;
                }
            },
            Remove(ref mut start, ref mut end, _) => {
                if *end >= op_start && *start <= op_end && bias < 0 {
                    return false;
                } else if *start > op_start {
                    *start = ((*start as i64) + bias) as u64;
                    *end = ((*end as i64) + bias) as u64;
                }
            },
        }

        true
    }
}

#[derive(Copy, Clone, PartialEq, Eq, RustcEncodable, RustcDecodable)]
pub enum OpCoords {
    InsertChar(u64),
    Insert(u64),
    RemoveChar(u64),
    Remove(u64, u64),
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Tests

#[test]
fn do_before_removed_same() {
    let before = Remove(0, 10, "asdfghjklz".to_string());
    let mut after = Remove(0, 10, "asdfghjklz".to_string());

    assert!(before.do_before(&mut after) == false);
}

#[test]
fn do_before_removed_touch_left() {
    let before = Remove(5, 10, "asdfghjklz".to_string());
    let mut after = Remove(1, 7, "hellos".to_string());

    assert!(before.do_before(&mut after) == false);
}

#[test]
fn do_before_removed_touch_right() {
    let before = Remove(0, 10, "asdfghjklz".to_string());
    let mut after = Remove(5, 15, "hellosderphello".to_string());

    assert!(before.do_before(&mut after) == false);
}

#[test]
fn do_before_removed_contained() {
    let before = Remove(0, 10, "asdfghjklz".to_string());
    let mut after = Remove(1, 9, "hellosir".to_string());

    assert!(before.do_before(&mut after) == false);
}

#[test]
fn insert_char_do_before_insert_char() {
    let before = InsertChar(0, 'a');
    let mut after = InsertChar(1, 'a');

    assert!(before.do_before(&mut after) == true);
    assert!(after == InsertChar(2, 'a'));
}

#[test]
fn insert_char_do_before_insert_char_no_effect() {
    let before = InsertChar(4, 'a');
    let mut after = InsertChar(2, 'a');

    assert!(before.do_before(&mut after) == true);
    assert!(after == InsertChar(2, 'a'));
}
