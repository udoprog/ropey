#[macro_use]
extern crate proptest;
extern crate ropey;

use proptest::collection::vec;
use proptest::test_runner::Config;
use ropey::Rope;

// Helper function used in the tests below
fn char_to_byte_idx(idx: usize, text: &str) -> usize {
    text.char_indices().nth(idx).unwrap_or((text.len(), '\0')).0
}

fn string_insert(text: &mut String, char_idx: usize, text_ins: &str) {
    let byte_idx = char_to_byte_idx(char_idx, text);
    text.insert_str(byte_idx, text_ins);
}

fn string_remove(text: &mut String, char_start: usize, char_end: usize) {
    let byte_start = char_to_byte_idx(char_start, text);
    let byte_end = char_to_byte_idx(char_end, text);
    let text_r = text.split_off(byte_end);
    text.truncate(byte_start);
    text.push_str(&text_r);
}

fn string_slice(text: &str, char_start: usize, char_end: usize) -> &str {
    let byte_start = char_to_byte_idx(char_start, text);
    let byte_end = char_to_byte_idx(char_end, text);
    &text[byte_start..byte_end]
}

//===========================================================================

proptest! {
    #![proptest_config(Config::with_cases(512))]

    #[test]
    fn pt_from_str(ref text in "\\PC*") {
        let rope = Rope::from_str(&text);

        rope.assert_integrity();
        rope.assert_invariants();

        assert_eq!(rope, text.as_str());
    }

    #[test]
    fn pt_insert(char_idx in 0usize..(CHAR_LEN+1), ref ins_text in "\\PC*") {
        let mut rope = Rope::from_str(TEXT);
        let mut text = String::from(TEXT);

        let len = rope.len_chars();
        rope.insert(char_idx % (len + 1), &ins_text);
        string_insert(&mut text, char_idx % (len + 1), &ins_text);

        rope.assert_integrity();
        rope.assert_invariants();

        assert_eq!(rope, text);
    }

    #[test]
    fn pt_remove(range in (0usize..(CHAR_LEN+1), 0usize..(CHAR_LEN+1))) {
        let mut rope = Rope::from_str(TEXT);
        let mut text = String::from(TEXT);

        let mut idx1 = range.0 % (rope.len_chars() + 1);
        let mut idx2 = range.1 % (rope.len_chars() + 1);
        if idx1 > idx2 {
            std::mem::swap(&mut idx1, &mut idx2)
        };

        rope.remove(idx1..idx2);
        string_remove(&mut text, idx1, idx2);

        rope.assert_integrity();
        rope.assert_invariants();

        assert_eq!(rope, text);
    }

    #[test]
    fn pt_split_off_and_append(mut idx in 0usize..(CHAR_LEN+1)) {
        let mut rope = Rope::from_str(TEXT);

        idx %= rope.len_chars() + 1;

        let rope2 = rope.split_off(idx);

        rope.assert_integrity();
        rope.assert_invariants();
        rope2.assert_integrity();
        rope2.assert_invariants();

        rope.append(rope2);

        rope.assert_integrity();
        rope.assert_invariants();

        assert_eq!(rope, TEXT);
    }

    #[test]
    fn pt_shrink_to_fit_01(ref char_idxs in vec(0usize..1000000, 0..1000)) {
        let mut rope = Rope::new();

        for idx in char_idxs.iter() {
            let len = rope.len_chars();
            rope.insert(idx % (len + 1), "Hello world!")
        }

        let capacity_before = rope.capacity();
        let rope_clone = rope.clone();

        rope.shrink_to_fit();

        rope.assert_integrity();
        rope.assert_invariants();

        let max_leaf_bytes = 768 - 33;
        assert!((rope.capacity() - rope.len_bytes()) < max_leaf_bytes);
        assert!(rope.capacity() <= capacity_before);
        assert_eq!(rope, rope_clone);
    }

    #[test]
    fn pt_shrink_to_fit_02(ref char_idxs in vec(0usize..1000000, 0..1000)) {
        let mut rope = Rope::new();
        let ins_text = "AT̴̷͚͖̜͈̪͎͔̝̫̦̹͔̻̮͂ͬͬ̌ͣ̿ͤ͌ͥ͑̀̂ͬ̚͘͜͞ô̵͚̤̯̹͖̅̌̈́̑̏̕͘͝A";

        for idx in char_idxs.iter() {
            let len = rope.len_chars();
            rope.insert(idx % (len + 1), ins_text);
        }

        let rope_clone = rope.clone();

        rope.shrink_to_fit();

        rope.assert_integrity();
        rope.assert_invariants();

        let max_leaf_bytes = 768 - 33;
        let max_diff = max_leaf_bytes + ((rope.len_bytes() / max_leaf_bytes) * ins_text.len());

        assert!((rope.capacity() - rope.len_bytes()) < max_diff);
        assert_eq!(rope, rope_clone);
    }

    #[test]
    fn pt_slice(ref text in "\\PC*", range in (0usize..1000000, 0usize..1000000)) {
        let rope = Rope::from_str(&text);

        let mut idx1 = range.0 % (rope.len_chars() + 1);
        let mut idx2 = range.1 % (rope.len_chars() + 1);
        if idx1 > idx2 {
            std::mem::swap(&mut idx1, &mut idx2)
        };

        let slice = rope.slice(idx1..idx2);
        let text_slice = string_slice(&text, idx1, idx2);

        assert_eq!(slice, text_slice);
        assert_eq!(slice.len_bytes(), text_slice.len());
        assert_eq!(slice.len_chars(), text_slice.chars().count());
    }
}

//===========================================================================

// Char count of TEXT, below
const CHAR_LEN: usize = 18352;

// 31624 bytes, 18352 chars, 95 lines
// Contains many long graphemes.
const TEXT: &str = "
T̴̷͚͖̜͈̪͎͔̝̫̦̹͔̻̮͂ͬͬ̌ͣ̿ͤ͌ͥ͑̀̂ͬ̚͘͜͞ô̵͚̤̯̹͖͍̦̼̦̖̞̺͕̳̬͇͕̟̜̅̌̈́̑̏̕͘͝ ͍̼̗̫͈̭̦̱̬͚̱̞͓̜̭̼͇̰̞ͮ͗ͣ́ͪ̔ͪ̍̑̏́̀̽̍̔͘͜͜͝ȋ̐̽ͦ̓̔̅͏̧̢̖̭̝̳̹̯̤͈̫͔͔̠͓͉̠͖̠͜ͅn̷̯̗̗̠̱̥͕͉̥͉̳̫̙̅͗̌̒͂̏͑̎̌̌̊͌͘͘ͅͅv̧̜͕͍͙͍̬͕͍̳͉̠͍̹̮̻̜ͨ̏͒̍ͬ̈́͒̈ͥ͗ͣ̄̃ͤ͊̌͆̓o̸̧̎̓͂̊͢҉͍̼̘͇̱̪̠͎̥̹ķ̈́͗͆ͥ͐͑̆̎́͌ͩͯ̊̓͐ͬ̇̕҉̢͏͚̲̰̗̦e̿̀͐̽ͪ̈ͤͬ҉́͟͏̵̫̲̱̻̰̲̦͇̭̟̺͈̞̫̰̜͕͖ͅ ̡̰͎͓͚͓͉͈̮̻̣̮̟̩̬̮̈̋̊͆ͪ̄ͪ͒ͨͧ̇ͪ̇̑̚t̷̬̟͎̞͈̯͙̹̜ͩ̓ͪ͛͐̐ͤ̾̄̈͒̽̈́̑͒̏h̨̢̳͇͓͉̝ͫ̐̓̆̓ͮ̔̓̈́̇ͫe̟̬̣̗͚̬̾̉͋̽ͯ̌ͯͬ̂ͯͭ̓͛́̚͡ ̨̭̱͉̭͈̈̽̆̂͒͗̀ͥͩ͡h̻̼̱̹̗͖̙̗̲̤͓͇͚͚̻̞̥ͥ͛͌ͧ̚͟i̢̯̹̹̘̳̙ͩ̉ͥ͆̽̇̾̎͗̔̓͂͂́̓̌ͬv̧̡̛̟̜̠͉͖̘̲̻̯͚͍͓̯̻̲̹̥͇̻̿̓͛̊̌ͩͩ́ͩ̍͌̚e̵̾́̈́̏͌͌̊͗̏͋ͦ͘͡͏͚̜͚͎͉͍̱͙̖̹̣̘̥̤̹̟͠-̔̌͐́͒ͦͮ̇ͭ̄̏̊̇̍̕͏̩̥̰͚̟m̨̒ͫͦ̔̔͋҉̱̩̗͇̥̰̩̭͍͚͠į̵̷̻̗͉͕͚̣̼̺͉̦̮̠̆̀̐ͩ͒ͯͩͯ͞ͅn̢̫̤̝̝͚̺͍̱̦͚͂̿ͨ̇ͤ͠d̡ͯ͋̋ͧ̈́̒̈͏̛͏̵̤̬͍̗̞̠̟̞̺̠̥̹̱͉̜͍͎̤ ̷̸̢̰͓̘̯͎̤̫̘͓̙̟̳͇̹̥͈͙̮̩̅̋͌͗̓͊̓ͨͣ͗̓͐̈́ͩ̓ͣrͫ͂͌ͪ̏̐̍̾ͥ̓͗̈͆̈ͥ̀̾̚̚҉̴̶̭͇̗͙̘̯̦̭̮̪͚̥̙̯̠͙̪͡e̵̸̲͉̳̙͖͖̫̘̪͕̳͓̻̙͙ͥ̍͂̽ͨ̓̒̒̏ͬ͗ͧ̑̀͠p̵̸̛̦̣͙̳̳̩̣̼̘͈͂ͪͭͤ̎r̶̩̟̞̙͔̼ͫ̆ͦ̐̀̏̾̉̍ͬ̅ͧ͊ͪ̒̈́ͬ̃͞ẻ̴̼͙͍͎̠̀̅̔̃̒͐ͦ̏̆̅̓͋͢ͅš̆̈̆̋ͨ̅̍̇͂̒ͩͨ̂̐̓ͩ͏̸͔͔̯͇͚̤̪̬̗͈̰̦̯͚̕ę̢̱̠͙̲͉̗͚̮̪͖̙̞̦͉͕̗̳͙ͦ̆̋͌ͣ̅̊́ͅņ̴̷̫̪̦͇̺̹͉̗̬̞̲̭̜̪͒̏͂̂̎͊́̋͒̏̅̋̚͘t̷̶̨̟̦̗̦̱͌͌ͩ̀i̴̴̢̖͓͙̘͇̠̦̙̭̼͖̹̾̒̎̐ͥͭ͋ͥ̅͟ͅņ̫͙̹̦̳͈͙̬̫̮͕̰̩̣̘̘͐̀̓ͭͩͬͯ̎͛̿ͫ̊̔̅́̕͠gͥͩ̂͌̒̊̕͏̻͙͖̣͙͍̹͕̝͖̼̙̘͝ ͤ͐̓̒̓͋̐̃̇͊̓ͦ͐̚͢҉̢̨̟̠͉̳͖̲̩͙̕ć̷̡̫̩̞̯̼̝̼͖̤̳̻̘̪̤͈̦̭ͣ́͂͐̽͆̔̀̚͜h̶̢̹̹̙͔̱̓ͦ͌̋̎ͭ͒͋̒ͭ̌̃͌̿ͣ̆̅͑ą̙̳̬̞̬͚̜̤̱̙͇̠̟̈ͤ͋̃̀̓̓ͯ̍̀̽ͣ̐̈̿̌̕ǫ͋͂͐ͬ̿ͯ̂̈́͌̓̌ͧ̕͏̜͔̗͚͔̘̣͕̘̲͖̼͇͖̗̳ͅͅs̷̸̝̙̭̦̣̦̯̭̦͙̹̻͍͇̣̼͗̌͆ͨͭ̃ͮ͐̿̕.̮̝̠̱̺͖͓̼̦̱̉͂͛̓̑̔̓ͮ̈̊̔͗́͝\r
̛̣̺̻̼̙̼͓̱̬͕̩͕̲̳̭̗̍ͤ͋̒̆̄ͨ̿ͧ̓͠ͅI̷̻̤̳̲͔͈̖̬̰͔̪͇͇̟̋ͨ̋̍̉̔͝͞͝ͅn̶͕̭͖̠̣͚̹̪͆ͪ̇̂̅̾ͫ́̅̉ͭ̀͜v̖͉̩͕̣͔̭͕̩̲̖̇̀ͬ́̄͒̆͑͆ͪͤ͆̾̍ͯ̚͜ǫ̡̡̫͎̟̞̰̞̹͇̲̏ͨ̄͊̊̇͒̽͢ķ̶̪̙̰̥͙̞̹̭̺͍͕̙̲̮͊ͭ́͋͛͋̑̒͊̏̒̅͛̄̓͟i̴͎̹̞̥͖̒̄ͮ̒̾ͮͧ̀̚͡n̸̵͓̲̟̞̳͚̼̣͙͖̈ͦ͒̿̅̒̿͛͊̇ͧ̉g̡̧̪̩͚͙͓̪͓͚͉̥̪͍̙̻͖͇͗̑͊͑̾̍͊̀ͅ ̷̵̠͚̘̟͓̫̣̲͎̩̹̣̼̟͊́̏ͫ̆ͩ̓͋͆̿̽̓͘̕t̴̢̝̻̖̲̬̜̺̖̻ͩ̿ͫ͗̈́̔͑̐ͮͦ̽̉̓̚͜h̷̛̲͇̫͈̣̭͂ͭ̂͋ͭ̋̔ͮ̆ͩ͞ë̩͕͉̯͇͔͚̭̼̮̣͓̯́ͭ̀ͣ͗̋̉ͨͬ̒ͥͩ͆̓̓́̀̚͘͝ ̛̫̠̗̥̳͇͉̟̮̪̻̤̪͚̟̜̔̌͌̈͌ͪ̋̎̄ͯ͐ͦ́͞͠fͦ̂̈ͬ̇̅̓̓ͫͣ̉̂̉̚͘͡͡͏̼̖̟͚̙̳͔͎̲̫̦̯͔̣̼̹ě̷̶̫͎̞̺̪̪͇͈̞̳̏̋̋͋̾̓̽̓̑ͮ͊ͣ̋̃̅̀͡e͇̗͎̱͔̦̠̰̩̩͖͙̠̻̝ͯ̿̔̀͋͑ͧ͊̆̇̿ͤ̄ͯ̇̀͢͠ͅl̂̿ͯ͛̊̒̓̈́͏̵̪̦̞̤̫̤͇̙̗͕͎̪͕̙̻̳̗̕͟͢i̞̣̙͎͈̗̮͉̱̜̱̝̞̤͋ͯ͋͐̈́ͫ̉̊̏̀ͯͨ͢͟͝n̳̻̼̥̖͍̭̅͂̓̔̔ͦ̔́ͦ͊̀͛̈́ͬͦ͢͡͡ģ̶̡̳̰̻̙̞̱̳̣̤̫̫͕̤̮̰̬̪̜͋͒̎̈́̉̏̀ͬͯ͌̇͊̚ ́̽ͤͦ̾̔͢҉̛̤͍͉̺̙̮̗̜̟̀͝ơ̢̱͓͓̙͉̖̠̯̦̗͍̓̐̃̉̅̃ͨ͆́ͪ̂̒̀̊̃͆̔͡͡ͅf́ͬ̊ͯͫ̉̈́̽̉̚͢͏̡̺̬̖͇̫͉̱ ̴͇̦̗̙̼̬͓̯͖̮͓͎̗͈̻̈́͆ͭ̐ͦ́͛̀͋̐̌ͬ͑̒̊̿̃͞c̶̸̣͔̬͕̪̱̩̣̑̒̑̓̍̓͂̍̔͌̚͘͜͞h̶͈̱͇͉̳͍͍̰͈͖̬̥͚̯͓̞̹̋̔ͯ̑̃́̒̎̎͊̈́̍̚̕ạ̴̞̱̥͍͙̺͉͚͎̫̦͎̥ͩ̀̀̊ͥ͢o̵̧͕̜͓͈̬̰̫̮͙̹͉̩̝̩͎̓̆͗̿̊̀ͯ̃ͪ̊ͫ̽̉̓ͧ͗́̚͢ͅͅs̡ͫ͋̑ͮ̍̃͊̄ͬ̅̈́ͬ̍̇̔̈̅̍̀҉̜͓̝̘̘̮̼͖͎̻͓͖̖͙̞ͅ.͗ͬͭͩ̌̅͗͏̷̮̗͇͔͇͈̮͢\r
̨͚̲̫̠̼͖̝̻̉ͤ̅̂ͩ̀̇ͬͭ̀͜Ẅ̢́̉͌ͮͬͨ͊̏͌̇̐͊͟͠҉̼̰̦̩͇͕̟̭̪̲͕̥͖̰̪͈̀ͅͅį̷ͣͦ̉̍ͨ͂͂͑̃͂ͪ̊̈̋̄͜҉̨͚̟̲̯̹̺̝̭̺̙͖͍t̼͓̰̩͙̦͓̟͚͖̀ͯ͛̍̈́͑͂̍̋́h̛̼̺̘̥̠̼̼̭͙̮͚̱̍ͯ̓̃̐̂̇͟ ̴̛͖͔̰̠̺̥̲ͮ̍ͫ̽͜õ̒ͯ̒̓ͦ̈́͑̔̒̓̎ͤ͑҉̸̭̱̤̭̬͈ų̙̫̤͖̺̫̱͓͓̗̪͇̩̙̔̉̊͂ͪ̇͢͟͞ͅt̸̬̣̫̞̫̅͐ͮ̌͌̈́̀̀͘ ̷̴̨̖̙̹͚ͬ̈́̈ͯͨͮ̇̈́̋̈́ͭ͛̑̉͊̕ö̡̍ͥ̂ͬͪͧ͒ͧ̏̓̇̂̄͆̌ͫͤ͢͠͝͏̖̱̯̘͙̰̖͎̰͓̟̤ṙ̡̬̟̬̜̪̮̺͖̗̘͈̟ͨ͐͗̑͒̐d̢ͭͫ̊̏ͬͥ͋́̌̈́ͮ̆ͬ̐̌̎͏̵̷̡̞̲̹̙͕̮̮͚ḙ̴̸̠͔͎̥͇͖͕̘̍̓̏̐ͩͩ̈́ͦ̐̋ͤ̎̾̌̏͊̊́̚͞ͅr̸͈̗̣̲̗̣̬̤ͦ̎ͫ̏̀ͥͪ̋ͧ̄͑̋͒͌͋ͦ̉͟͞.ͨͣ̽̈́͒̄ͮ̀͋͋͏̴̧̯̺̙̱̻͙̜\r
̡̣̞̠͓̰͍̠͕̭̺̼͊̽̿͊ͮ̐̓̒̊͒̔̓͐ͨ̈̌́T̸̸̓́̋ͬ́͆ͨͫ͌͂ͣ̋͒҉̺̝͎̟͖͚̠h̸̡̰̜̦͇͕̪̝̳͕͉̲̝̑ͥ͋ͧ̎̆͌͟e̛̹͍͍̫̙̞̪̭̙̟͙̱̺̮̳͕̜ͫ̓ͭ͊ͫ͆̀̚͟͡ ̿͂̄ͧ̔̎ͧ͑̾̀̓͏̦͍̳͈̳͔̘̖̲̯̰̟̝̳̖̦N̶̡̧̦̮̟̦̩̰̣̝̆̀͊̔͢e͛̄ͮͦͨ͂̔̓̍̄̉͆͊̑̑̆̚͏̜̗͎̝̼̯̥̜͖͍̪̝͞ͅͅz̨̛̀̾ͪ͗̉́͠͏͚̫̼̫̜̣pͪͦ͌̄ͥ̆ͣͩ͋̉́̏͞͏̥̜̝̳̱̞̙̳̤͙̟̟̮̦ȅ̷̩̟͉̯͕͔̘̺̥̻̻ͧ̊̅̽ͣ͑̓̑̽ͦ̾͌͜r̴̭̥̲̲̤͚͈̰͇̰͈̰̹ͫ̒ͯ̿͒ͧ̊͆͒ͣ́ḍ̭̟̤̈́̌̓̈́ͫ͐̍͂͞į̛̞̝̮̣͙͙̤̇̂̓̎͋̿̓̎̄̈́ͧ̓ͩ̐̓̄̋ͭ͞͠a͋̔̋ͫ̂͐͂҉̸̛̥̩̯̯̤̝͔̠̝̯̪̥̩̻̼̮n͌ͣ̂͋̿̚҉̛̙̲̺̯͇͓̝̯̪̟͔̩͟ͅ ̢̨͚̻̗̘͖̯̐ͥ͋̽ͯ̎̈́͋̏̄͋̆̑̊̆̚̕͟ͅh̢̛̗̱̭͇͖̰̮̮͈̲͍̯̟ͭ͊̎̽̓ͦͤ͠ï̛̘̝̦͎̦̭̠͖̳͎̮̼̏͐ͧ̒̒͐͑ͪͫ̋̽̚̚͜v̴̮͕̝̮̞͐̄͗̋͒ͤ̎̈̑ͬͮ̄̾ͤ̓̾͊͗͟é̶̷̡̩͖̰̫͓̟ͮͬͣ͊-ͦ͛ͩͤͨͨ̆̄͏̼̜̭͔̳͈͖̳̩͢ͅͅm̷̴̓́̓͛͒̾̍̉҉̛̗̹̠̣̪̺͎̖̝͚̖͙i̛̥͓̬̫͉͕͉͆͒ͧ̂̿̔̔͆̆̓̍͊̀͜n͌ͧͣ̅̌̎ͦͦ͑̑ͭ̆ͬ̀ͤ̀ͣ̚҉͎̰̱͚͈͈̬̹͕̺̙͙̼͘͘͞d̶͖̫̟̲͕̺̠͎̘͕̱̼͙̪̪̩͙̅̅̑̓̇͑̊̉͜͞ ̶̵̷̴̡̠͚̪͕̣̱̖̱̗̤̭̭͔͖͚ͧͤͥ͒̌ͪ͊͂͒̓͂ͧͧ̇̇͐̑̔ͅͅơ̵̲̲͇̯̰͇̜̣͕͕͓̲̤̲͔͚̞͑͗ͤ̓́̚͠ͅf̢̧̛̩̯̼̫͖̾ͣ͌̾̉́̈́̑̈́̚͞͞ͅ ͤͩ́͋͒ͫͬͣ̋̅̆҉̧̱̻͓͕͉̹̫̫̞̯̪̙̩͍̦͔̖̮̀͟ͅc͉̠̜̩̟͕͎̙̣̮̘̼͋ͯ̍ͨ̅̄ͫ̈̋ͫ̊͡͝ȟ̸̨ͯͦ̂̉̇̾̆ͭ̋̐̈̆̀̚͜҉͚͕̻̖a̶̴̛͚̗͙̳̬̲͚ͦ́̐ͥ́̔̅̑̎͐̑ͯ̾ͤͥͧ͡ò̶̧̞̪̦̥̪̻̦̝̳̬̔͛͛ͣ̋̌̔ͫ̂̽ͫ͘͠s̸̖̣̬̤̫͇̫̣̑͆͒̎̏́͟.̴̗̤̭͉̯̻̤͕̌ͯ̍ͤ̓͌ͤ̈̆̉ͦ̇́̚͘͟͝ͅ ̯̹̪͓̬͌̔̌ͬ̀͘͢͡͡Z̡̩̲̩̰̫̩̟͍̰͖͔̭ͣ̆̾ͭ̀́͞ͅa̡̡̙̜̭͇͎͔̙̞̫͓̜͉͔̬ͭ̈ͨ̉͆ͣͫ̃͌̓͌́ͣͥ̒̌͊͘͝l̢̨̡̯̙̫͖̫̺̘̬̟͈͌̊ͧͫͦ̉̃ͩͦ̒ͯ̇̌̓͛͟͝ͅg̵̙̼̼ͪ͂ͭ͗̈̕ȯ̅ͧ̓ͪ́̂͑̐ͩͥͬ̊̑͆̇͒ͫͣ͝҉͎̟̜̥͎̮̣͉̖̟̯̦̖͙͙͞ͅ.̈̑ͩ̇̂ͬ̓ͬ͊͂ͨ̽͠͏̺͎̞̦̜͍͚̯̯͔̝̞̻̩̖\r
̷̰̪͍͎͔͒ͯͥ̾̉͆ͤ̊̓̂͋̀͆H̸̸̹̞̙̺͎̠̯̤ͨ̉̍ͬͤ̓̐͌ͥͮ͞eͣ̈̾͛́͏͕̗͍̜̼͎͚̟̬̣̝̕ͅͅ ̴̛̩̗̼̝̣̩͚͇̯́̉͋̂̍͂̌ͮ͋̾͜͠wͮ̽̓ͭ̿͐̽̐̽͆̓͝҉̡̼̲͖̪̥h̢̢̛͍̰̰̻̱̼̰̹̖̖̪̝̥̘̎̀ͪ͒̾ͫͬ̆̑o̡̗̠̞̱̥͎̰͎͍̫̻͓͇͓͐ͥͯ͂̅͠ͅ ̡̛̏͑ͦ̓͊ͮͫͯͭ̌͒̆̍̈͠҉͖͚̪̫̗̮W̴̐̊͋̾ͥͫ҉͎̞͔̯̫̹͖̰͉̹̼͎̰̱͓̻̀a̶̩̤̙̣͎̳̭̲̗̠͉̳̭̭̦̞͎̮̅͌̾͗̾͛̇̀́͟͞ͅi̷̡ͣ̆̌͋͒͒́͘͏̮̺̩͎͇̜͍̫ṯ̴̢͖̥̖͇͎̦̦̹̖͇̪ͭ̅̍͐̇͒͋̽̏̿̒͆ͧ̄͋ͧͩ͒͜s̙̥̖̘̖͚̭̤̮̖̘̰̫̟̈́ͣ̍ͧ͐ͥ̏͆̃̿͒̔͐́̚͟ͅ ̨ͭ̌ͬͯ͆̒͋ͭ̔̿ͧ̅̓ͣ͡͏͇̟͉̥͔̬̼͚͙͚B̛̜̮̤͓̝̪̪͈͕̘̜͙̰̮̫̘̣͓͔̅ͩ͊̔ͦͯ́̌́͆ͭ̓́e̶̢̡̦͇͕̙͈͖͕̦̬̫͕̣̺̒̿͂͐͒͋͂ͦ́͋ͤ̿ͬ̊ͣ͗̑̽͜ͅͅh̸͑ͫͧ̑ͬͧ̈́̎̃ͣ̊̾͂ͨͤ̓͐̐̑͏̸̭͓̘͉̩i̧̧̭̣͈̝̺̼̺̠͉̞̜̲̳͙̦͐̔ͯ͛̅̾n̸͓̝̤̙͙͔ͪ̋̈́͒̒ͭ̈́̓ͮ̋̀̋̀̈ͩ́͌̄͘d̷̫̳̩̼̥̗̲̰͇͉̼̬̤͇̖ͮ̿ͬ͂ͦ̏̓ͮ̽͂̾̾ͯ͆͜͠ ̨̈́͒̇̏̄̑̓ͮͥ̒ͤͨ̋҉̴̴̟̱͙̟̫̩̗͔̝͔̀Ţ̵̝̟̖̭͇̻̳͖͉̺̖̖͙͙̺̐̈́̓ͯ̆̇̋ͩ͊̄̾̾ͬ̌̚͟ͅh̡͈̗̜͙̬̗̲̦̲̟̗̦̬͓̳ͧ̋̌͂͂ͨͬͦ̿̏̈́̋ͣ̒̕͡ͅͅe̗͇̰̰̥̪̗͑̔̓́̈́ͨ̊́̿̅ͯͥ̈́͐͗͘͢͝ ̡̢̛̯͎͓̰̘͎̦̪̯̪̥̰̲͇̠̲͔ͤͤ̇̅̆̋̂̆̈́ͤ̿͑ͅW̡͓͈̲̲͉̜͔̖͈̻̱͚̿̌͗̉ͤ͢͡ͅͅa̔̾͛̅͊͋͐҉̱̹̬͍͙̻̱l̢͎̟̬̙̼̱̫̮̘̼͔̭̅ͬ͑ͣ̏̾̅̓ͣ̿ͣ̈́̕͢͡ͅͅl̡̥̣͔̭̇̒͛͒͐̄̽͛̋ͥ̌͢͟͡.̷̰̝̮͔̟̦͈̥̬̻̥̬͎͓̻̲̇ͮ̿ͨͦ̽ͫ͟͢͝͠\r
̗̱͖͈͌̈ͦ͛ͮ̌͋̽̃͆̀͂ͨͧ̄̔̔ͭ̏͢Z̃̉̿ͮ̃̀͘͏͕̬̯̖͚̗͔Aͣ̑̈̓̈̑̈̀̿̚҉͙͍̦̗̦͙̠̝̩̯ͅͅL̴͖̞̞͙̱̻̥̬̜̦̐̇̉̈̽ͪ̅ͪ̂̔͌͑ͭ͐ͤ̈́̿̉͞ͅG̴̵̲̰̹̖͎͕ͯ̆̓̽͢͠Ŏ̶̡̺̼͙̣̞̩͕̥̟̝͕͔̯̞ͨ͒͊̂̊͂͗̒͆̾͆̌͆̃̎ͣͫ͜͡ͅ!̓̽̎̑̏́̓̓ͣ̀͏̱̩̭̣̹̺̗͜͞͞\r

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Maecenas sit\r
amet tellus  nec turpis feugiat semper. Nam at nulla laoreet, finibus\r
eros sit amet, fringilla  mauris. Fusce vestibulum nec ligula efficitur\r
laoreet. Nunc orci leo, varius eget  ligula vulputate, consequat\r
eleifend nisi. Cras justo purus, imperdiet a augue  malesuada, convallis\r
cursus libero. Fusce pretium arcu in elementum laoreet. Duis  mauris\r
nulla, suscipit at est nec, malesuada pellentesque eros. Quisque semper\r
porta  malesuada. Nunc hendrerit est ac faucibus mollis. Nam fermentum\r
id libero sed  egestas. Duis a accumsan sapien. Nam neque diam, congue\r
non erat et, porta sagittis  turpis. Vivamus vitae mauris sit amet massa\r
mollis molestie. Morbi scelerisque,  augue id congue imperdiet, felis\r
lacus euismod dui, vitae facilisis massa dui quis  sapien. Vivamus\r
hendrerit a urna a lobortis.\r

T̴̷͚͖̜͈̪͎͔̝̫̦̹͔̻̮͂ͬͬ̌ͣ̿ͤ͌ͥ͑̀̂ͬ̚͘͜͞ô̵͚̤̯̹͖͍̦̼̦̖̞̺͕̳̬͇͕̟̜̅̌̈́̑̏̕͘͝ ͍̼̗̫͈̭̦̱̬͚̱̞͓̜̭̼͇̰̞ͮ͗ͣ́ͪ̔ͪ̍̑̏́̀̽̍̔͘͜͜͝ȋ̐̽ͦ̓̔̅͏̧̢̖̭̝̳̹̯̤͈̫͔͔̠͓͉̠͖̠͜ͅn̷̯̗̗̠̱̥͕͉̥͉̳̫̙̅͗̌̒͂̏͑̎̌̌̊͌͘͘ͅͅv̧̜͕͍͙͍̬͕͍̳͉̠͍̹̮̻̜ͨ̏͒̍ͬ̈́͒̈ͥ͗ͣ̄̃ͤ͊̌͆̓o̸̧̎̓͂̊͢҉͍̼̘͇̱̪̠͎̥̹ķ̈́͗͆ͥ͐͑̆̎́͌ͩͯ̊̓͐ͬ̇̕҉̢͏͚̲̰̗̦e̿̀͐̽ͪ̈ͤͬ҉́͟͏̵̫̲̱̻̰̲̦͇̭̟̺͈̞̫̰̜͕͖ͅ ̡̰͎͓͚͓͉͈̮̻̣̮̟̩̬̮̈̋̊͆ͪ̄ͪ͒ͨͧ̇ͪ̇̑̚t̷̬̟͎̞͈̯͙̹̜ͩ̓ͪ͛͐̐ͤ̾̄̈͒̽̈́̑͒̏h̨̢̳͇͓͉̝ͫ̐̓̆̓ͮ̔̓̈́̇ͫe̟̬̣̗͚̬̾̉͋̽ͯ̌ͯͬ̂ͯͭ̓͛́̚͡ ̨̭̱͉̭͈̈̽̆̂͒͗̀ͥͩ͡h̻̼̱̹̗͖̙̗̲̤͓͇͚͚̻̞̥ͥ͛͌ͧ̚͟i̢̯̹̹̘̳̙ͩ̉ͥ͆̽̇̾̎͗̔̓͂͂́̓̌ͬv̧̡̛̟̜̠͉͖̘̲̻̯͚͍͓̯̻̲̹̥͇̻̿̓͛̊̌ͩͩ́ͩ̍͌̚e̵̾́̈́̏͌͌̊͗̏͋ͦ͘͡͏͚̜͚͎͉͍̱͙̖̹̣̘̥̤̹̟͠-̔̌͐́͒ͦͮ̇ͭ̄̏̊̇̍̕͏̩̥̰͚̟m̨̒ͫͦ̔̔͋҉̱̩̗͇̥̰̩̭͍͚͠į̵̷̻̗͉͕͚̣̼̺͉̦̮̠̆̀̐ͩ͒ͯͩͯ͞ͅn̢̫̤̝̝͚̺͍̱̦͚͂̿ͨ̇ͤ͠d̡ͯ͋̋ͧ̈́̒̈͏̛͏̵̤̬͍̗̞̠̟̞̺̠̥̹̱͉̜͍͎̤ ̷̸̢̰͓̘̯͎̤̫̘͓̙̟̳͇̹̥͈͙̮̩̅̋͌͗̓͊̓ͨͣ͗̓͐̈́ͩ̓ͣrͫ͂͌ͪ̏̐̍̾ͥ̓͗̈͆̈ͥ̀̾̚̚҉̴̶̭͇̗͙̘̯̦̭̮̪͚̥̙̯̠͙̪͡e̵̸̲͉̳̙͖͖̫̘̪͕̳͓̻̙͙ͥ̍͂̽ͨ̓̒̒̏ͬ͗ͧ̑̀͠p̵̸̛̦̣͙̳̳̩̣̼̘͈͂ͪͭͤ̎r̶̩̟̞̙͔̼ͫ̆ͦ̐̀̏̾̉̍ͬ̅ͧ͊ͪ̒̈́ͬ̃͞ẻ̴̼͙͍͎̠̀̅̔̃̒͐ͦ̏̆̅̓͋͢ͅš̆̈̆̋ͨ̅̍̇͂̒ͩͨ̂̐̓ͩ͏̸͔͔̯͇͚̤̪̬̗͈̰̦̯͚̕ę̢̱̠͙̲͉̗͚̮̪͖̙̞̦͉͕̗̳͙ͦ̆̋͌ͣ̅̊́ͅņ̴̷̫̪̦͇̺̹͉̗̬̞̲̭̜̪͒̏͂̂̎͊́̋͒̏̅̋̚͘t̷̶̨̟̦̗̦̱͌͌ͩ̀i̴̴̢̖͓͙̘͇̠̦̙̭̼͖̹̾̒̎̐ͥͭ͋ͥ̅͟ͅņ̫͙̹̦̳͈͙̬̫̮͕̰̩̣̘̘͐̀̓ͭͩͬͯ̎͛̿ͫ̊̔̅́̕͠gͥͩ̂͌̒̊̕͏̻͙͖̣͙͍̹͕̝͖̼̙̘͝ ͤ͐̓̒̓͋̐̃̇͊̓ͦ͐̚͢҉̢̨̟̠͉̳͖̲̩͙̕ć̷̡̫̩̞̯̼̝̼͖̤̳̻̘̪̤͈̦̭ͣ́͂͐̽͆̔̀̚͜h̶̢̹̹̙͔̱̓ͦ͌̋̎ͭ͒͋̒ͭ̌̃͌̿ͣ̆̅͑ą̙̳̬̞̬͚̜̤̱̙͇̠̟̈ͤ͋̃̀̓̓ͯ̍̀̽ͣ̐̈̿̌̕ǫ͋͂͐ͬ̿ͯ̂̈́͌̓̌ͧ̕͏̜͔̗͚͔̘̣͕̘̲͖̼͇͖̗̳ͅͅs̷̸̝̙̭̦̣̦̯̭̦͙̹̻͍͇̣̼͗̌͆ͨͭ̃ͮ͐̿̕.̮̝̠̱̺͖͓̼̦̱̉͂͛̓̑̔̓ͮ̈̊̔͗́͝\r
̛̣̺̻̼̙̼͓̱̬͕̩͕̲̳̭̗̍ͤ͋̒̆̄ͨ̿ͧ̓͠ͅI̷̻̤̳̲͔͈̖̬̰͔̪͇͇̟̋ͨ̋̍̉̔͝͞͝ͅn̶͕̭͖̠̣͚̹̪͆ͪ̇̂̅̾ͫ́̅̉ͭ̀͜v̖͉̩͕̣͔̭͕̩̲̖̇̀ͬ́̄͒̆͑͆ͪͤ͆̾̍ͯ̚͜ǫ̡̡̫͎̟̞̰̞̹͇̲̏ͨ̄͊̊̇͒̽͢ķ̶̪̙̰̥͙̞̹̭̺͍͕̙̲̮͊ͭ́͋͛͋̑̒͊̏̒̅͛̄̓͟i̴͎̹̞̥͖̒̄ͮ̒̾ͮͧ̀̚͡n̸̵͓̲̟̞̳͚̼̣͙͖̈ͦ͒̿̅̒̿͛͊̇ͧ̉g̡̧̪̩͚͙͓̪͓͚͉̥̪͍̙̻͖͇͗̑͊͑̾̍͊̀ͅ ̷̵̠͚̘̟͓̫̣̲͎̩̹̣̼̟͊́̏ͫ̆ͩ̓͋͆̿̽̓͘̕t̴̢̝̻̖̲̬̜̺̖̻ͩ̿ͫ͗̈́̔͑̐ͮͦ̽̉̓̚͜h̷̛̲͇̫͈̣̭͂ͭ̂͋ͭ̋̔ͮ̆ͩ͞ë̩͕͉̯͇͔͚̭̼̮̣͓̯́ͭ̀ͣ͗̋̉ͨͬ̒ͥͩ͆̓̓́̀̚͘͝ ̛̫̠̗̥̳͇͉̟̮̪̻̤̪͚̟̜̔̌͌̈͌ͪ̋̎̄ͯ͐ͦ́͞͠fͦ̂̈ͬ̇̅̓̓ͫͣ̉̂̉̚͘͡͡͏̼̖̟͚̙̳͔͎̲̫̦̯͔̣̼̹ě̷̶̫͎̞̺̪̪͇͈̞̳̏̋̋͋̾̓̽̓̑ͮ͊ͣ̋̃̅̀͡e͇̗͎̱͔̦̠̰̩̩͖͙̠̻̝ͯ̿̔̀͋͑ͧ͊̆̇̿ͤ̄ͯ̇̀͢͠ͅl̂̿ͯ͛̊̒̓̈́͏̵̪̦̞̤̫̤͇̙̗͕͎̪͕̙̻̳̗̕͟͢i̞̣̙͎͈̗̮͉̱̜̱̝̞̤͋ͯ͋͐̈́ͫ̉̊̏̀ͯͨ͢͟͝n̳̻̼̥̖͍̭̅͂̓̔̔ͦ̔́ͦ͊̀͛̈́ͬͦ͢͡͡ģ̶̡̳̰̻̙̞̱̳̣̤̫̫͕̤̮̰̬̪̜͋͒̎̈́̉̏̀ͬͯ͌̇͊̚ ́̽ͤͦ̾̔͢҉̛̤͍͉̺̙̮̗̜̟̀͝ơ̢̱͓͓̙͉̖̠̯̦̗͍̓̐̃̉̅̃ͨ͆́ͪ̂̒̀̊̃͆̔͡͡ͅf́ͬ̊ͯͫ̉̈́̽̉̚͢͏̡̺̬̖͇̫͉̱ ̴͇̦̗̙̼̬͓̯͖̮͓͎̗͈̻̈́͆ͭ̐ͦ́͛̀͋̐̌ͬ͑̒̊̿̃͞c̶̸̣͔̬͕̪̱̩̣̑̒̑̓̍̓͂̍̔͌̚͘͜͞h̶͈̱͇͉̳͍͍̰͈͖̬̥͚̯͓̞̹̋̔ͯ̑̃́̒̎̎͊̈́̍̚̕ạ̴̞̱̥͍͙̺͉͚͎̫̦͎̥ͩ̀̀̊ͥ͢o̵̧͕̜͓͈̬̰̫̮͙̹͉̩̝̩͎̓̆͗̿̊̀ͯ̃ͪ̊ͫ̽̉̓ͧ͗́̚͢ͅͅs̡ͫ͋̑ͮ̍̃͊̄ͬ̅̈́ͬ̍̇̔̈̅̍̀҉̜͓̝̘̘̮̼͖͎̻͓͖̖͙̞ͅ.͗ͬͭͩ̌̅͗͏̷̮̗͇͔͇͈̮͢\r
̨͚̲̫̠̼͖̝̻̉ͤ̅̂ͩ̀̇ͬͭ̀͜Ẅ̢́̉͌ͮͬͨ͊̏͌̇̐͊͟͠҉̼̰̦̩͇͕̟̭̪̲͕̥͖̰̪͈̀ͅͅį̷ͣͦ̉̍ͨ͂͂͑̃͂ͪ̊̈̋̄͜҉̨͚̟̲̯̹̺̝̭̺̙͖͍t̼͓̰̩͙̦͓̟͚͖̀ͯ͛̍̈́͑͂̍̋́h̛̼̺̘̥̠̼̼̭͙̮͚̱̍ͯ̓̃̐̂̇͟ ̴̛͖͔̰̠̺̥̲ͮ̍ͫ̽͜õ̒ͯ̒̓ͦ̈́͑̔̒̓̎ͤ͑҉̸̭̱̤̭̬͈ų̙̫̤͖̺̫̱͓͓̗̪͇̩̙̔̉̊͂ͪ̇͢͟͞ͅt̸̬̣̫̞̫̅͐ͮ̌͌̈́̀̀͘ ̷̴̨̖̙̹͚ͬ̈́̈ͯͨͮ̇̈́̋̈́ͭ͛̑̉͊̕ö̡̍ͥ̂ͬͪͧ͒ͧ̏̓̇̂̄͆̌ͫͤ͢͠͝͏̖̱̯̘͙̰̖͎̰͓̟̤ṙ̡̬̟̬̜̪̮̺͖̗̘͈̟ͨ͐͗̑͒̐d̢ͭͫ̊̏ͬͥ͋́̌̈́ͮ̆ͬ̐̌̎͏̵̷̡̞̲̹̙͕̮̮͚ḙ̴̸̠͔͎̥͇͖͕̘̍̓̏̐ͩͩ̈́ͦ̐̋ͤ̎̾̌̏͊̊́̚͞ͅr̸͈̗̣̲̗̣̬̤ͦ̎ͫ̏̀ͥͪ̋ͧ̄͑̋͒͌͋ͦ̉͟͞.ͨͣ̽̈́͒̄ͮ̀͋͋͏̴̧̯̺̙̱̻͙̜\r
̡̣̞̠͓̰͍̠͕̭̺̼͊̽̿͊ͮ̐̓̒̊͒̔̓͐ͨ̈̌́T̸̸̓́̋ͬ́͆ͨͫ͌͂ͣ̋͒҉̺̝͎̟͖͚̠h̸̡̰̜̦͇͕̪̝̳͕͉̲̝̑ͥ͋ͧ̎̆͌͟e̛̹͍͍̫̙̞̪̭̙̟͙̱̺̮̳͕̜ͫ̓ͭ͊ͫ͆̀̚͟͡ ̿͂̄ͧ̔̎ͧ͑̾̀̓͏̦͍̳͈̳͔̘̖̲̯̰̟̝̳̖̦N̶̡̧̦̮̟̦̩̰̣̝̆̀͊̔͢e͛̄ͮͦͨ͂̔̓̍̄̉͆͊̑̑̆̚͏̜̗͎̝̼̯̥̜͖͍̪̝͞ͅͅz̨̛̀̾ͪ͗̉́͠͏͚̫̼̫̜̣pͪͦ͌̄ͥ̆ͣͩ͋̉́̏͞͏̥̜̝̳̱̞̙̳̤͙̟̟̮̦ȅ̷̩̟͉̯͕͔̘̺̥̻̻ͧ̊̅̽ͣ͑̓̑̽ͦ̾͌͜r̴̭̥̲̲̤͚͈̰͇̰͈̰̹ͫ̒ͯ̿͒ͧ̊͆͒ͣ́ḍ̭̟̤̈́̌̓̈́ͫ͐̍͂͞į̛̞̝̮̣͙͙̤̇̂̓̎͋̿̓̎̄̈́ͧ̓ͩ̐̓̄̋ͭ͞͠a͋̔̋ͫ̂͐͂҉̸̛̥̩̯̯̤̝͔̠̝̯̪̥̩̻̼̮n͌ͣ̂͋̿̚҉̛̙̲̺̯͇͓̝̯̪̟͔̩͟ͅ ̢̨͚̻̗̘͖̯̐ͥ͋̽ͯ̎̈́͋̏̄͋̆̑̊̆̚̕͟ͅh̢̛̗̱̭͇͖̰̮̮͈̲͍̯̟ͭ͊̎̽̓ͦͤ͠ï̛̘̝̦͎̦̭̠͖̳͎̮̼̏͐ͧ̒̒͐͑ͪͫ̋̽̚̚͜v̴̮͕̝̮̞͐̄͗̋͒ͤ̎̈̑ͬͮ̄̾ͤ̓̾͊͗͟é̶̷̡̩͖̰̫͓̟ͮͬͣ͊-ͦ͛ͩͤͨͨ̆̄͏̼̜̭͔̳͈͖̳̩͢ͅͅm̷̴̓́̓͛͒̾̍̉҉̛̗̹̠̣̪̺͎̖̝͚̖͙i̛̥͓̬̫͉͕͉͆͒ͧ̂̿̔̔͆̆̓̍͊̀͜n͌ͧͣ̅̌̎ͦͦ͑̑ͭ̆ͬ̀ͤ̀ͣ̚҉͎̰̱͚͈͈̬̹͕̺̙͙̼͘͘͞d̶͖̫̟̲͕̺̠͎̘͕̱̼͙̪̪̩͙̅̅̑̓̇͑̊̉͜͞ ̶̵̷̴̡̠͚̪͕̣̱̖̱̗̤̭̭͔͖͚ͧͤͥ͒̌ͪ͊͂͒̓͂ͧͧ̇̇͐̑̔ͅͅơ̵̲̲͇̯̰͇̜̣͕͕͓̲̤̲͔͚̞͑͗ͤ̓́̚͠ͅf̢̧̛̩̯̼̫͖̾ͣ͌̾̉́̈́̑̈́̚͞͞ͅ ͤͩ́͋͒ͫͬͣ̋̅̆҉̧̱̻͓͕͉̹̫̫̞̯̪̙̩͍̦͔̖̮̀͟ͅc͉̠̜̩̟͕͎̙̣̮̘̼͋ͯ̍ͨ̅̄ͫ̈̋ͫ̊͡͝ȟ̸̨ͯͦ̂̉̇̾̆ͭ̋̐̈̆̀̚͜҉͚͕̻̖a̶̴̛͚̗͙̳̬̲͚ͦ́̐ͥ́̔̅̑̎͐̑ͯ̾ͤͥͧ͡ò̶̧̞̪̦̥̪̻̦̝̳̬̔͛͛ͣ̋̌̔ͫ̂̽ͫ͘͠s̸̖̣̬̤̫͇̫̣̑͆͒̎̏́͟.̴̗̤̭͉̯̻̤͕̌ͯ̍ͤ̓͌ͤ̈̆̉ͦ̇́̚͘͟͝ͅ ̯̹̪͓̬͌̔̌ͬ̀͘͢͡͡Z̡̩̲̩̰̫̩̟͍̰͖͔̭ͣ̆̾ͭ̀́͞ͅa̡̡̙̜̭͇͎͔̙̞̫͓̜͉͔̬ͭ̈ͨ̉͆ͣͫ̃͌̓͌́ͣͥ̒̌͊͘͝l̢̨̡̯̙̫͖̫̺̘̬̟͈͌̊ͧͫͦ̉̃ͩͦ̒ͯ̇̌̓͛͟͝ͅg̵̙̼̼ͪ͂ͭ͗̈̕ȯ̅ͧ̓ͪ́̂͑̐ͩͥͬ̊̑͆̇͒ͫͣ͝҉͎̟̜̥͎̮̣͉̖̟̯̦̖͙͙͞ͅ.̈̑ͩ̇̂ͬ̓ͬ͊͂ͨ̽͠͏̺͎̞̦̜͍͚̯̯͔̝̞̻̩̖\r
̷̰̪͍͎͔͒ͯͥ̾̉͆ͤ̊̓̂͋̀͆H̸̸̹̞̙̺͎̠̯̤ͨ̉̍ͬͤ̓̐͌ͥͮ͞eͣ̈̾͛́͏͕̗͍̜̼͎͚̟̬̣̝̕ͅͅ ̴̛̩̗̼̝̣̩͚͇̯́̉͋̂̍͂̌ͮ͋̾͜͠wͮ̽̓ͭ̿͐̽̐̽͆̓͝҉̡̼̲͖̪̥h̢̢̛͍̰̰̻̱̼̰̹̖̖̪̝̥̘̎̀ͪ͒̾ͫͬ̆̑o̡̗̠̞̱̥͎̰͎͍̫̻͓͇͓͐ͥͯ͂̅͠ͅ ̡̛̏͑ͦ̓͊ͮͫͯͭ̌͒̆̍̈͠҉͖͚̪̫̗̮W̴̐̊͋̾ͥͫ҉͎̞͔̯̫̹͖̰͉̹̼͎̰̱͓̻̀a̶̩̤̙̣͎̳̭̲̗̠͉̳̭̭̦̞͎̮̅͌̾͗̾͛̇̀́͟͞ͅi̷̡ͣ̆̌͋͒͒́͘͏̮̺̩͎͇̜͍̫ṯ̴̢͖̥̖͇͎̦̦̹̖͇̪ͭ̅̍͐̇͒͋̽̏̿̒͆ͧ̄͋ͧͩ͒͜s̙̥̖̘̖͚̭̤̮̖̘̰̫̟̈́ͣ̍ͧ͐ͥ̏͆̃̿͒̔͐́̚͟ͅ ̨ͭ̌ͬͯ͆̒͋ͭ̔̿ͧ̅̓ͣ͡͏͇̟͉̥͔̬̼͚͙͚B̛̜̮̤͓̝̪̪͈͕̘̜͙̰̮̫̘̣͓͔̅ͩ͊̔ͦͯ́̌́͆ͭ̓́e̶̢̡̦͇͕̙͈͖͕̦̬̫͕̣̺̒̿͂͐͒͋͂ͦ́͋ͤ̿ͬ̊ͣ͗̑̽͜ͅͅh̸͑ͫͧ̑ͬͧ̈́̎̃ͣ̊̾͂ͨͤ̓͐̐̑͏̸̭͓̘͉̩i̧̧̭̣͈̝̺̼̺̠͉̞̜̲̳͙̦͐̔ͯ͛̅̾n̸͓̝̤̙͙͔ͪ̋̈́͒̒ͭ̈́̓ͮ̋̀̋̀̈ͩ́͌̄͘d̷̫̳̩̼̥̗̲̰͇͉̼̬̤͇̖ͮ̿ͬ͂ͦ̏̓ͮ̽͂̾̾ͯ͆͜͠ ̨̈́͒̇̏̄̑̓ͮͥ̒ͤͨ̋҉̴̴̟̱͙̟̫̩̗͔̝͔̀Ţ̵̝̟̖̭͇̻̳͖͉̺̖̖͙͙̺̐̈́̓ͯ̆̇̋ͩ͊̄̾̾ͬ̌̚͟ͅh̡͈̗̜͙̬̗̲̦̲̟̗̦̬͓̳ͧ̋̌͂͂ͨͬͦ̿̏̈́̋ͣ̒̕͡ͅͅe̗͇̰̰̥̪̗͑̔̓́̈́ͨ̊́̿̅ͯͥ̈́͐͗͘͢͝ ̡̢̛̯͎͓̰̘͎̦̪̯̪̥̰̲͇̠̲͔ͤͤ̇̅̆̋̂̆̈́ͤ̿͑ͅW̡͓͈̲̲͉̜͔̖͈̻̱͚̿̌͗̉ͤ͢͡ͅͅa̔̾͛̅͊͋͐҉̱̹̬͍͙̻̱l̢͎̟̬̙̼̱̫̮̘̼͔̭̅ͬ͑ͣ̏̾̅̓ͣ̿ͣ̈́̕͢͡ͅͅl̡̥̣͔̭̇̒͛͒͐̄̽͛̋ͥ̌͢͟͡.̷̰̝̮͔̟̦͈̥̬̻̥̬͎͓̻̲̇ͮ̿ͨͦ̽ͫ͟͢͝͠\r
̗̱͖͈͌̈ͦ͛ͮ̌͋̽̃͆̀͂ͨͧ̄̔̔ͭ̏͢Z̃̉̿ͮ̃̀͘͏͕̬̯̖͚̗͔Aͣ̑̈̓̈̑̈̀̿̚҉͙͍̦̗̦͙̠̝̩̯ͅͅL̴͖̞̞͙̱̻̥̬̜̦̐̇̉̈̽ͪ̅ͪ̂̔͌͑ͭ͐ͤ̈́̿̉͞ͅG̴̵̲̰̹̖͎͕ͯ̆̓̽͢͠Ŏ̶̡̺̼͙̣̞̩͕̥̟̝͕͔̯̞ͨ͒͊̂̊͂͗̒͆̾͆̌͆̃̎ͣͫ͜͡ͅ!̓̽̎̑̏́̓̓ͣ̀͏̱̩̭̣̹̺̗͜͞͞\r

Pellentesque nec viverra metus. Sed aliquet pellentesque scelerisque.\r
Duis efficitur  erat sit amet dui maximus egestas. Nullam blandit ante\r
tortor. Suspendisse vitae  consectetur sem, at sollicitudin neque.\r
Suspendisse sodales faucibus eros vitae  pellentesque. Cras non quam\r
dictum, pellentesque urna in, ornare erat. Praesent leo  est, aliquet et\r
euismod non, hendrerit sed urna. Sed convallis porttitor est, vel\r
aliquet felis cursus ac. Vivamus feugiat eget nisi eu molestie.\r
Phasellus tincidunt  nisl eget molestie consectetur. Phasellus vitae ex\r
ut odio sollicitudin vulputate.  Sed et nulla accumsan, eleifend arcu\r
eget, gravida neque. Donec sit amet tincidunt  eros. Ut in volutpat\r
ante.\r

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Maecenas sit\r
amet tellus  nec turpis feugiat semper. Nam at nulla laoreet, finibus\r
eros sit amet, fringilla  mauris. Fusce vestibulum nec ligula efficitur\r
laoreet. Nunc orci leo, varius eget  ligula vulputate, consequat\r
eleifend nisi. Cras justo purus, imperdiet a augue  malesuada, convallis\r
cursus libero. Fusce pretium arcu in elementum laoreet. Duis  mauris\r
nulla, suscipit at est nec, malesuada pellentesque eros. Quisque semper\r
porta  malesuada. Nunc hendrerit est ac faucibus mollis. Nam fermentum\r
id libero sed  egestas. Duis a accumsan sapien. Nam neque diam, congue\r
non erat et, porta sagittis  turpis. Vivamus vitae mauris sit amet massa\r
mollis molestie. Morbi scelerisque,  augue id congue imperdiet, felis\r
lacus euismod dui, vitae facilisis massa dui quis  sapien. Vivamus\r
hendrerit a urna a lobortis.\r

T̴̷͚͖̜͈̪͎͔̝̫̦̹͔̻̮͂ͬͬ̌ͣ̿ͤ͌ͥ͑̀̂ͬ̚͘͜͞ô̵͚̤̯̹͖͍̦̼̦̖̞̺͕̳̬͇͕̟̜̅̌̈́̑̏̕͘͝ ͍̼̗̫͈̭̦̱̬͚̱̞͓̜̭̼͇̰̞ͮ͗ͣ́ͪ̔ͪ̍̑̏́̀̽̍̔͘͜͜͝ȋ̐̽ͦ̓̔̅͏̧̢̖̭̝̳̹̯̤͈̫͔͔̠͓͉̠͖̠͜ͅn̷̯̗̗̠̱̥͕͉̥͉̳̫̙̅͗̌̒͂̏͑̎̌̌̊͌͘͘ͅͅv̧̜͕͍͙͍̬͕͍̳͉̠͍̹̮̻̜ͨ̏͒̍ͬ̈́͒̈ͥ͗ͣ̄̃ͤ͊̌͆̓o̸̧̎̓͂̊͢҉͍̼̘͇̱̪̠͎̥̹ķ̈́͗͆ͥ͐͑̆̎́͌ͩͯ̊̓͐ͬ̇̕҉̢͏͚̲̰̗̦e̿̀͐̽ͪ̈ͤͬ҉́͟͏̵̫̲̱̻̰̲̦͇̭̟̺͈̞̫̰̜͕͖ͅ ̡̰͎͓͚͓͉͈̮̻̣̮̟̩̬̮̈̋̊͆ͪ̄ͪ͒ͨͧ̇ͪ̇̑̚t̷̬̟͎̞͈̯͙̹̜ͩ̓ͪ͛͐̐ͤ̾̄̈͒̽̈́̑͒̏h̨̢̳͇͓͉̝ͫ̐̓̆̓ͮ̔̓̈́̇ͫe̟̬̣̗͚̬̾̉͋̽ͯ̌ͯͬ̂ͯͭ̓͛́̚͡ ̨̭̱͉̭͈̈̽̆̂͒͗̀ͥͩ͡h̻̼̱̹̗͖̙̗̲̤͓͇͚͚̻̞̥ͥ͛͌ͧ̚͟i̢̯̹̹̘̳̙ͩ̉ͥ͆̽̇̾̎͗̔̓͂͂́̓̌ͬv̧̡̛̟̜̠͉͖̘̲̻̯͚͍͓̯̻̲̹̥͇̻̿̓͛̊̌ͩͩ́ͩ̍͌̚e̵̾́̈́̏͌͌̊͗̏͋ͦ͘͡͏͚̜͚͎͉͍̱͙̖̹̣̘̥̤̹̟͠-̔̌͐́͒ͦͮ̇ͭ̄̏̊̇̍̕͏̩̥̰͚̟m̨̒ͫͦ̔̔͋҉̱̩̗͇̥̰̩̭͍͚͠į̵̷̻̗͉͕͚̣̼̺͉̦̮̠̆̀̐ͩ͒ͯͩͯ͞ͅn̢̫̤̝̝͚̺͍̱̦͚͂̿ͨ̇ͤ͠d̡ͯ͋̋ͧ̈́̒̈͏̛͏̵̤̬͍̗̞̠̟̞̺̠̥̹̱͉̜͍͎̤ ̷̸̢̰͓̘̯͎̤̫̘͓̙̟̳͇̹̥͈͙̮̩̅̋͌͗̓͊̓ͨͣ͗̓͐̈́ͩ̓ͣrͫ͂͌ͪ̏̐̍̾ͥ̓͗̈͆̈ͥ̀̾̚̚҉̴̶̭͇̗͙̘̯̦̭̮̪͚̥̙̯̠͙̪͡e̵̸̲͉̳̙͖͖̫̘̪͕̳͓̻̙͙ͥ̍͂̽ͨ̓̒̒̏ͬ͗ͧ̑̀͠p̵̸̛̦̣͙̳̳̩̣̼̘͈͂ͪͭͤ̎r̶̩̟̞̙͔̼ͫ̆ͦ̐̀̏̾̉̍ͬ̅ͧ͊ͪ̒̈́ͬ̃͞ẻ̴̼͙͍͎̠̀̅̔̃̒͐ͦ̏̆̅̓͋͢ͅš̆̈̆̋ͨ̅̍̇͂̒ͩͨ̂̐̓ͩ͏̸͔͔̯͇͚̤̪̬̗͈̰̦̯͚̕ę̢̱̠͙̲͉̗͚̮̪͖̙̞̦͉͕̗̳͙ͦ̆̋͌ͣ̅̊́ͅņ̴̷̫̪̦͇̺̹͉̗̬̞̲̭̜̪͒̏͂̂̎͊́̋͒̏̅̋̚͘t̷̶̨̟̦̗̦̱͌͌ͩ̀i̴̴̢̖͓͙̘͇̠̦̙̭̼͖̹̾̒̎̐ͥͭ͋ͥ̅͟ͅņ̫͙̹̦̳͈͙̬̫̮͕̰̩̣̘̘͐̀̓ͭͩͬͯ̎͛̿ͫ̊̔̅́̕͠gͥͩ̂͌̒̊̕͏̻͙͖̣͙͍̹͕̝͖̼̙̘͝ ͤ͐̓̒̓͋̐̃̇͊̓ͦ͐̚͢҉̢̨̟̠͉̳͖̲̩͙̕ć̷̡̫̩̞̯̼̝̼͖̤̳̻̘̪̤͈̦̭ͣ́͂͐̽͆̔̀̚͜h̶̢̹̹̙͔̱̓ͦ͌̋̎ͭ͒͋̒ͭ̌̃͌̿ͣ̆̅͑ą̙̳̬̞̬͚̜̤̱̙͇̠̟̈ͤ͋̃̀̓̓ͯ̍̀̽ͣ̐̈̿̌̕ǫ͋͂͐ͬ̿ͯ̂̈́͌̓̌ͧ̕͏̜͔̗͚͔̘̣͕̘̲͖̼͇͖̗̳ͅͅs̷̸̝̙̭̦̣̦̯̭̦͙̹̻͍͇̣̼͗̌͆ͨͭ̃ͮ͐̿̕.̮̝̠̱̺͖͓̼̦̱̉͂͛̓̑̔̓ͮ̈̊̔͗́͝\r
̛̣̺̻̼̙̼͓̱̬͕̩͕̲̳̭̗̍ͤ͋̒̆̄ͨ̿ͧ̓͠ͅI̷̻̤̳̲͔͈̖̬̰͔̪͇͇̟̋ͨ̋̍̉̔͝͞͝ͅn̶͕̭͖̠̣͚̹̪͆ͪ̇̂̅̾ͫ́̅̉ͭ̀͜v̖͉̩͕̣͔̭͕̩̲̖̇̀ͬ́̄͒̆͑͆ͪͤ͆̾̍ͯ̚͜ǫ̡̡̫͎̟̞̰̞̹͇̲̏ͨ̄͊̊̇͒̽͢ķ̶̪̙̰̥͙̞̹̭̺͍͕̙̲̮͊ͭ́͋͛͋̑̒͊̏̒̅͛̄̓͟i̴͎̹̞̥͖̒̄ͮ̒̾ͮͧ̀̚͡n̸̵͓̲̟̞̳͚̼̣͙͖̈ͦ͒̿̅̒̿͛͊̇ͧ̉g̡̧̪̩͚͙͓̪͓͚͉̥̪͍̙̻͖͇͗̑͊͑̾̍͊̀ͅ ̷̵̠͚̘̟͓̫̣̲͎̩̹̣̼̟͊́̏ͫ̆ͩ̓͋͆̿̽̓͘̕t̴̢̝̻̖̲̬̜̺̖̻ͩ̿ͫ͗̈́̔͑̐ͮͦ̽̉̓̚͜h̷̛̲͇̫͈̣̭͂ͭ̂͋ͭ̋̔ͮ̆ͩ͞ë̩͕͉̯͇͔͚̭̼̮̣͓̯́ͭ̀ͣ͗̋̉ͨͬ̒ͥͩ͆̓̓́̀̚͘͝ ̛̫̠̗̥̳͇͉̟̮̪̻̤̪͚̟̜̔̌͌̈͌ͪ̋̎̄ͯ͐ͦ́͞͠fͦ̂̈ͬ̇̅̓̓ͫͣ̉̂̉̚͘͡͡͏̼̖̟͚̙̳͔͎̲̫̦̯͔̣̼̹ě̷̶̫͎̞̺̪̪͇͈̞̳̏̋̋͋̾̓̽̓̑ͮ͊ͣ̋̃̅̀͡e͇̗͎̱͔̦̠̰̩̩͖͙̠̻̝ͯ̿̔̀͋͑ͧ͊̆̇̿ͤ̄ͯ̇̀͢͠ͅl̂̿ͯ͛̊̒̓̈́͏̵̪̦̞̤̫̤͇̙̗͕͎̪͕̙̻̳̗̕͟͢i̞̣̙͎͈̗̮͉̱̜̱̝̞̤͋ͯ͋͐̈́ͫ̉̊̏̀ͯͨ͢͟͝n̳̻̼̥̖͍̭̅͂̓̔̔ͦ̔́ͦ͊̀͛̈́ͬͦ͢͡͡ģ̶̡̳̰̻̙̞̱̳̣̤̫̫͕̤̮̰̬̪̜͋͒̎̈́̉̏̀ͬͯ͌̇͊̚ ́̽ͤͦ̾̔͢҉̛̤͍͉̺̙̮̗̜̟̀͝ơ̢̱͓͓̙͉̖̠̯̦̗͍̓̐̃̉̅̃ͨ͆́ͪ̂̒̀̊̃͆̔͡͡ͅf́ͬ̊ͯͫ̉̈́̽̉̚͢͏̡̺̬̖͇̫͉̱ ̴͇̦̗̙̼̬͓̯͖̮͓͎̗͈̻̈́͆ͭ̐ͦ́͛̀͋̐̌ͬ͑̒̊̿̃͞c̶̸̣͔̬͕̪̱̩̣̑̒̑̓̍̓͂̍̔͌̚͘͜͞h̶͈̱͇͉̳͍͍̰͈͖̬̥͚̯͓̞̹̋̔ͯ̑̃́̒̎̎͊̈́̍̚̕ạ̴̞̱̥͍͙̺͉͚͎̫̦͎̥ͩ̀̀̊ͥ͢o̵̧͕̜͓͈̬̰̫̮͙̹͉̩̝̩͎̓̆͗̿̊̀ͯ̃ͪ̊ͫ̽̉̓ͧ͗́̚͢ͅͅs̡ͫ͋̑ͮ̍̃͊̄ͬ̅̈́ͬ̍̇̔̈̅̍̀҉̜͓̝̘̘̮̼͖͎̻͓͖̖͙̞ͅ.͗ͬͭͩ̌̅͗͏̷̮̗͇͔͇͈̮͢\r
̨͚̲̫̠̼͖̝̻̉ͤ̅̂ͩ̀̇ͬͭ̀͜Ẅ̢́̉͌ͮͬͨ͊̏͌̇̐͊͟͠҉̼̰̦̩͇͕̟̭̪̲͕̥͖̰̪͈̀ͅͅį̷ͣͦ̉̍ͨ͂͂͑̃͂ͪ̊̈̋̄͜҉̨͚̟̲̯̹̺̝̭̺̙͖͍t̼͓̰̩͙̦͓̟͚͖̀ͯ͛̍̈́͑͂̍̋́h̛̼̺̘̥̠̼̼̭͙̮͚̱̍ͯ̓̃̐̂̇͟ ̴̛͖͔̰̠̺̥̲ͮ̍ͫ̽͜õ̒ͯ̒̓ͦ̈́͑̔̒̓̎ͤ͑҉̸̭̱̤̭̬͈ų̙̫̤͖̺̫̱͓͓̗̪͇̩̙̔̉̊͂ͪ̇͢͟͞ͅt̸̬̣̫̞̫̅͐ͮ̌͌̈́̀̀͘ ̷̴̨̖̙̹͚ͬ̈́̈ͯͨͮ̇̈́̋̈́ͭ͛̑̉͊̕ö̡̍ͥ̂ͬͪͧ͒ͧ̏̓̇̂̄͆̌ͫͤ͢͠͝͏̖̱̯̘͙̰̖͎̰͓̟̤ṙ̡̬̟̬̜̪̮̺͖̗̘͈̟ͨ͐͗̑͒̐d̢ͭͫ̊̏ͬͥ͋́̌̈́ͮ̆ͬ̐̌̎͏̵̷̡̞̲̹̙͕̮̮͚ḙ̴̸̠͔͎̥͇͖͕̘̍̓̏̐ͩͩ̈́ͦ̐̋ͤ̎̾̌̏͊̊́̚͞ͅr̸͈̗̣̲̗̣̬̤ͦ̎ͫ̏̀ͥͪ̋ͧ̄͑̋͒͌͋ͦ̉͟͞.ͨͣ̽̈́͒̄ͮ̀͋͋͏̴̧̯̺̙̱̻͙̜\r
̡̣̞̠͓̰͍̠͕̭̺̼͊̽̿͊ͮ̐̓̒̊͒̔̓͐ͨ̈̌́T̸̸̓́̋ͬ́͆ͨͫ͌͂ͣ̋͒҉̺̝͎̟͖͚̠h̸̡̰̜̦͇͕̪̝̳͕͉̲̝̑ͥ͋ͧ̎̆͌͟e̛̹͍͍̫̙̞̪̭̙̟͙̱̺̮̳͕̜ͫ̓ͭ͊ͫ͆̀̚͟͡ ̿͂̄ͧ̔̎ͧ͑̾̀̓͏̦͍̳͈̳͔̘̖̲̯̰̟̝̳̖̦N̶̡̧̦̮̟̦̩̰̣̝̆̀͊̔͢e͛̄ͮͦͨ͂̔̓̍̄̉͆͊̑̑̆̚͏̜̗͎̝̼̯̥̜͖͍̪̝͞ͅͅz̨̛̀̾ͪ͗̉́͠͏͚̫̼̫̜̣pͪͦ͌̄ͥ̆ͣͩ͋̉́̏͞͏̥̜̝̳̱̞̙̳̤͙̟̟̮̦ȅ̷̩̟͉̯͕͔̘̺̥̻̻ͧ̊̅̽ͣ͑̓̑̽ͦ̾͌͜r̴̭̥̲̲̤͚͈̰͇̰͈̰̹ͫ̒ͯ̿͒ͧ̊͆͒ͣ́ḍ̭̟̤̈́̌̓̈́ͫ͐̍͂͞į̛̞̝̮̣͙͙̤̇̂̓̎͋̿̓̎̄̈́ͧ̓ͩ̐̓̄̋ͭ͞͠a͋̔̋ͫ̂͐͂҉̸̛̥̩̯̯̤̝͔̠̝̯̪̥̩̻̼̮n͌ͣ̂͋̿̚҉̛̙̲̺̯͇͓̝̯̪̟͔̩͟ͅ ̢̨͚̻̗̘͖̯̐ͥ͋̽ͯ̎̈́͋̏̄͋̆̑̊̆̚̕͟ͅh̢̛̗̱̭͇͖̰̮̮͈̲͍̯̟ͭ͊̎̽̓ͦͤ͠ï̛̘̝̦͎̦̭̠͖̳͎̮̼̏͐ͧ̒̒͐͑ͪͫ̋̽̚̚͜v̴̮͕̝̮̞͐̄͗̋͒ͤ̎̈̑ͬͮ̄̾ͤ̓̾͊͗͟é̶̷̡̩͖̰̫͓̟ͮͬͣ͊-ͦ͛ͩͤͨͨ̆̄͏̼̜̭͔̳͈͖̳̩͢ͅͅm̷̴̓́̓͛͒̾̍̉҉̛̗̹̠̣̪̺͎̖̝͚̖͙i̛̥͓̬̫͉͕͉͆͒ͧ̂̿̔̔͆̆̓̍͊̀͜n͌ͧͣ̅̌̎ͦͦ͑̑ͭ̆ͬ̀ͤ̀ͣ̚҉͎̰̱͚͈͈̬̹͕̺̙͙̼͘͘͞d̶͖̫̟̲͕̺̠͎̘͕̱̼͙̪̪̩͙̅̅̑̓̇͑̊̉͜͞ ̶̵̷̴̡̠͚̪͕̣̱̖̱̗̤̭̭͔͖͚ͧͤͥ͒̌ͪ͊͂͒̓͂ͧͧ̇̇͐̑̔ͅͅơ̵̲̲͇̯̰͇̜̣͕͕͓̲̤̲͔͚̞͑͗ͤ̓́̚͠ͅf̢̧̛̩̯̼̫͖̾ͣ͌̾̉́̈́̑̈́̚͞͞ͅ ͤͩ́͋͒ͫͬͣ̋̅̆҉̧̱̻͓͕͉̹̫̫̞̯̪̙̩͍̦͔̖̮̀͟ͅc͉̠̜̩̟͕͎̙̣̮̘̼͋ͯ̍ͨ̅̄ͫ̈̋ͫ̊͡͝ȟ̸̨ͯͦ̂̉̇̾̆ͭ̋̐̈̆̀̚͜҉͚͕̻̖a̶̴̛͚̗͙̳̬̲͚ͦ́̐ͥ́̔̅̑̎͐̑ͯ̾ͤͥͧ͡ò̶̧̞̪̦̥̪̻̦̝̳̬̔͛͛ͣ̋̌̔ͫ̂̽ͫ͘͠s̸̖̣̬̤̫͇̫̣̑͆͒̎̏́͟.̴̗̤̭͉̯̻̤͕̌ͯ̍ͤ̓͌ͤ̈̆̉ͦ̇́̚͘͟͝ͅ ̯̹̪͓̬͌̔̌ͬ̀͘͢͡͡Z̡̩̲̩̰̫̩̟͍̰͖͔̭ͣ̆̾ͭ̀́͞ͅa̡̡̙̜̭͇͎͔̙̞̫͓̜͉͔̬ͭ̈ͨ̉͆ͣͫ̃͌̓͌́ͣͥ̒̌͊͘͝l̢̨̡̯̙̫͖̫̺̘̬̟͈͌̊ͧͫͦ̉̃ͩͦ̒ͯ̇̌̓͛͟͝ͅg̵̙̼̼ͪ͂ͭ͗̈̕ȯ̅ͧ̓ͪ́̂͑̐ͩͥͬ̊̑͆̇͒ͫͣ͝҉͎̟̜̥͎̮̣͉̖̟̯̦̖͙͙͞ͅ.̈̑ͩ̇̂ͬ̓ͬ͊͂ͨ̽͠͏̺͎̞̦̜͍͚̯̯͔̝̞̻̩̖\r
̷̰̪͍͎͔͒ͯͥ̾̉͆ͤ̊̓̂͋̀͆H̸̸̹̞̙̺͎̠̯̤ͨ̉̍ͬͤ̓̐͌ͥͮ͞eͣ̈̾͛́͏͕̗͍̜̼͎͚̟̬̣̝̕ͅͅ ̴̛̩̗̼̝̣̩͚͇̯́̉͋̂̍͂̌ͮ͋̾͜͠wͮ̽̓ͭ̿͐̽̐̽͆̓͝҉̡̼̲͖̪̥h̢̢̛͍̰̰̻̱̼̰̹̖̖̪̝̥̘̎̀ͪ͒̾ͫͬ̆̑o̡̗̠̞̱̥͎̰͎͍̫̻͓͇͓͐ͥͯ͂̅͠ͅ ̡̛̏͑ͦ̓͊ͮͫͯͭ̌͒̆̍̈͠҉͖͚̪̫̗̮W̴̐̊͋̾ͥͫ҉͎̞͔̯̫̹͖̰͉̹̼͎̰̱͓̻̀a̶̩̤̙̣͎̳̭̲̗̠͉̳̭̭̦̞͎̮̅͌̾͗̾͛̇̀́͟͞ͅi̷̡ͣ̆̌͋͒͒́͘͏̮̺̩͎͇̜͍̫ṯ̴̢͖̥̖͇͎̦̦̹̖͇̪ͭ̅̍͐̇͒͋̽̏̿̒͆ͧ̄͋ͧͩ͒͜s̙̥̖̘̖͚̭̤̮̖̘̰̫̟̈́ͣ̍ͧ͐ͥ̏͆̃̿͒̔͐́̚͟ͅ ̨ͭ̌ͬͯ͆̒͋ͭ̔̿ͧ̅̓ͣ͡͏͇̟͉̥͔̬̼͚͙͚B̛̜̮̤͓̝̪̪͈͕̘̜͙̰̮̫̘̣͓͔̅ͩ͊̔ͦͯ́̌́͆ͭ̓́e̶̢̡̦͇͕̙͈͖͕̦̬̫͕̣̺̒̿͂͐͒͋͂ͦ́͋ͤ̿ͬ̊ͣ͗̑̽͜ͅͅh̸͑ͫͧ̑ͬͧ̈́̎̃ͣ̊̾͂ͨͤ̓͐̐̑͏̸̭͓̘͉̩i̧̧̭̣͈̝̺̼̺̠͉̞̜̲̳͙̦͐̔ͯ͛̅̾n̸͓̝̤̙͙͔ͪ̋̈́͒̒ͭ̈́̓ͮ̋̀̋̀̈ͩ́͌̄͘d̷̫̳̩̼̥̗̲̰͇͉̼̬̤͇̖ͮ̿ͬ͂ͦ̏̓ͮ̽͂̾̾ͯ͆͜͠ ̨̈́͒̇̏̄̑̓ͮͥ̒ͤͨ̋҉̴̴̟̱͙̟̫̩̗͔̝͔̀Ţ̵̝̟̖̭͇̻̳͖͉̺̖̖͙͙̺̐̈́̓ͯ̆̇̋ͩ͊̄̾̾ͬ̌̚͟ͅh̡͈̗̜͙̬̗̲̦̲̟̗̦̬͓̳ͧ̋̌͂͂ͨͬͦ̿̏̈́̋ͣ̒̕͡ͅͅe̗͇̰̰̥̪̗͑̔̓́̈́ͨ̊́̿̅ͯͥ̈́͐͗͘͢͝ ̡̢̛̯͎͓̰̘͎̦̪̯̪̥̰̲͇̠̲͔ͤͤ̇̅̆̋̂̆̈́ͤ̿͑ͅW̡͓͈̲̲͉̜͔̖͈̻̱͚̿̌͗̉ͤ͢͡ͅͅa̔̾͛̅͊͋͐҉̱̹̬͍͙̻̱l̢͎̟̬̙̼̱̫̮̘̼͔̭̅ͬ͑ͣ̏̾̅̓ͣ̿ͣ̈́̕͢͡ͅͅl̡̥̣͔̭̇̒͛͒͐̄̽͛̋ͥ̌͢͟͡.̷̰̝̮͔̟̦͈̥̬̻̥̬͎͓̻̲̇ͮ̿ͨͦ̽ͫ͟͢͝͠\r
̗̱͖͈͌̈ͦ͛ͮ̌͋̽̃͆̀͂ͨͧ̄̔̔ͭ̏͢Z̃̉̿ͮ̃̀͘͏͕̬̯̖͚̗͔Aͣ̑̈̓̈̑̈̀̿̚҉͙͍̦̗̦͙̠̝̩̯ͅͅL̴͖̞̞͙̱̻̥̬̜̦̐̇̉̈̽ͪ̅ͪ̂̔͌͑ͭ͐ͤ̈́̿̉͞ͅG̴̵̲̰̹̖͎͕ͯ̆̓̽͢͠Ŏ̶̡̺̼͙̣̞̩͕̥̟̝͕͔̯̞ͨ͒͊̂̊͂͗̒͆̾͆̌͆̃̎ͣͫ͜͡ͅ!̓̽̎̑̏́̓̓ͣ̀͏̱̩̭̣̹̺̗͜͞͞\r

Aliquam finibus metus commodo sem egestas, non mollis odio pretium.\r
Aenean ex  lectus, rutrum nec laoreet at, posuere sit amet lacus. Nulla\r
eros augue, vehicula et  molestie accumsan, dictum vel odio. In quis\r
risus finibus, pellentesque ipsum  blandit, volutpat diam. Etiam\r
suscipit varius mollis. Proin vel luctus nisi, ac  ornare justo. Integer\r
porttitor quam magna. Donec vitae metus tempor, ultricies  risus in,\r
dictum erat. Integer porttitor faucibus vestibulum. Class aptent taciti\r
sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos.\r
Vestibulum  ante ipsum primis in faucibus orci luctus et ultrices\r
posuere cubilia Curae; Nam  semper congue ante, a ultricies velit\r
venenatis vitae. Proin non neque sit amet ex  commodo congue non nec\r
elit. Nullam vel dignissim ipsum. Duis sed lobortis ante.  Aenean\r
feugiat rutrum magna ac luctus.\r

Ut imperdiet non ante sit amet rutrum. Cras vel massa eget nisl gravida\r
auctor.  Nulla bibendum ut tellus ut rutrum. Quisque malesuada lacinia\r
felis, vitae semper  elit. Praesent sit amet velit imperdiet, lobortis\r
nunc at, faucibus tellus. Nullam  porttitor augue mauris, a dapibus\r
tellus ultricies et. Fusce aliquet nec velit in  mattis. Sed mi ante,\r
lacinia eget ornare vel, faucibus at metus.\r

Pellentesque nec viverra metus. Sed aliquet pellentesque scelerisque.\r
Duis efficitur  erat sit amet dui maximus egestas. Nullam blandit ante\r
tortor. Suspendisse vitae  consectetur sem, at sollicitudin neque.\r
Suspendisse sodales faucibus eros vitae  pellentesque. Cras non quam\r
dictum, pellentesque urna in, ornare erat. Praesent leo  est, aliquet et\r
euismod non, hendrerit sed urna. Sed convallis porttitor est, vel\r
aliquet felis cursus ac. Vivamus feugiat eget nisi eu molestie.\r
Phasellus tincidunt  nisl eget molestie consectetur. Phasellus vitae ex\r
ut odio sollicitudin vulputate.  Sed et nulla accumsan, eleifend arcu\r
eget, gravida neque. Donec sit amet tincidunt  eros. Ut in volutpat\r
ante.\r
";
