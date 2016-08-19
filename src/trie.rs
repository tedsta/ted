use std;
use std::collections::HashMap;

pub type Result<T> = std::result::Result<T, String>;

pub struct Trie<T> {
    character: u8,
    data: Option<T>,
    children: HashMap<u8, Trie<T>>,
}

impl<T> Trie<T> {
    pub fn new(character: u8, data: Option<T>) -> Trie<T> {
        Trie {
            character: character,
            data: data,
            children: HashMap::new(),
        }
    }

    pub fn insert(&mut self, chars: &[u8], data: Option<T>) -> Result<()> {
        if chars.len() == 1 {
            // Made it to a leaf node
            if chars[0] == self.character && self.data.is_none() {
                self.data = data;
                return Ok(());
            } else {
                return Err("".to_string());
            }
        }

        if let Some(child) = self.children.get_mut(&chars[1]) {
            // It's on an existing branch
            return child.insert(&chars[1..], data);
        }

        // It's a new branch in the tree
        let mut trie = Trie::new(chars[1], None);
        trie.insert(&chars[1..], data);
        self.children.insert(chars[1], trie);
        Ok(())
    }

    pub fn get(&self, chars: &[u8]) -> Option<&T> {
        println!("{:?}", chars);
        if chars.len() == 1 {
            // Made it to a leaf node
            if chars[0] == self.character && self.data.is_some() {
                return self.data.as_ref();
            } else {
                return None;
            }
        }

        self.children.get(&chars[1]).and_then(|c| c.get(&chars[1..]))
    }

    pub fn get_mut(&mut self, chars: &[u8]) -> Option<&mut T> {
        if chars.len() == 1 {
            // Made it to a leaf node
            if chars[0] == self.character && self.data.is_some() {
                return self.data.as_mut();
            } else {
                return None;
            }
        }

        self.children.get_mut(&chars[1]).and_then(|c| c.get_mut(&chars[1..]))
    }
}

#[test]
pub fn trie_get() {
    let mut trie = Trie::new(b'a', None);
    trie.insert(b"a", Some(5u32)).unwrap();
    trie.insert(b"ab", None).unwrap();
    trie.insert(b"abc", Some(7u32)).unwrap();

    println!("{:?}", trie.get(b"abc"));
    assert!(trie.get(b"a") == Some(&5));
    assert!(trie.get(b"ab") == None);
    assert!(trie.get(b"abc") == Some(&7));
}

#[test]
pub fn trie_get_mut() {
    let mut trie = Trie::new(b'a', None);
    trie.insert(b"a", Some(5u32)).unwrap();
    trie.insert(b"ab", None).unwrap();
    trie.insert(b"abc", Some(7u32)).unwrap();

    *trie.get_mut(b"abc").unwrap() = 8;
    assert!(trie.get(b"a") == Some(&5));
    assert!(trie.get(b"ab") == None);
    assert!(trie.get(b"abc") == Some(&8));
}
