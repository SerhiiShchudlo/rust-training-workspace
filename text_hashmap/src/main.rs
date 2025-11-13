use std::collections::HashMap;
use std::collections::hash_map::Entry;

fn main() {
    let input = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
    let map = index_str(input);
    println!("{map:?}");
}

fn index_str(text: &str) -> HashMap<&str, Vec<usize>> {
    let mut map = HashMap::new();

    let mut in_word = false;
    let mut word_start_index = 0;

    for (index, ch) in text.char_indices() {
        if ch.is_alphabetic() {
            if !in_word {
                word_start_index = index;
                in_word = true;
            }
        } else {
            if in_word { //end of the word

                match map.entry(&text[word_start_index..index]) {
                    Entry::Vacant(e) => {
                        e.insert(vec![word_start_index]);
                    }
                    Entry::Occupied(mut e) => {
                        e.get_mut().push(word_start_index);
                    }
                };

                /* Alternative version of filling HashMap

                map.entry(&text[word_start_index..index])
                    .or_insert(Vec::new())
                    .push(word_start_index)
                */

            }
            in_word = false;
        }
    }

    map
}
