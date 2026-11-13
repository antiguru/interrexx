//! The messages `rxqueue` and `rxsubcom` print: catalogue entries of major 999, by subcode.

use interrexx_inventory::errors::{lookup, substitute};

/// The catalogue major of the utilities' messages.
const MAJOR: u16 = 999;

/// `rxsubcom`: the parameters are incorrect, no action named.
pub const SUBCOM_GENERAL: u16 = 0;
/// `rxsubcom REGISTER`: the parameters are incorrect.
pub const SUBCOM_REGISTER: u16 = 1;
/// `rxsubcom DROP`: the parameters are incorrect.
pub const SUBCOM_DROP: u16 = 2;
/// `rxsubcom LOAD`: the parameters are incorrect.
pub const SUBCOM_LOAD: u16 = 3;
/// `rxsubcom QUERY`: the parameters are incorrect.
pub const SUBCOM_QUERY: u16 = 11;
/// `rxqueue`: the queuing system is not available.
pub const QUEUE_NOT_INITIALISED: u16 = 4;
/// `rxqueue`: the size of the data is incorrect.
pub const QUEUE_SIZE: u16 = 5;
/// `rxqueue`: storage for data queues is exhausted.
pub const QUEUE_NO_MEMORY: u16 = 6;
/// `rxqueue`: a queue name is not valid. Insert 1 is the name.
pub const QUEUE_NAME: u16 = 7;
/// `rxqueue`: the queue access mode is not correct.
pub const QUEUE_ACCESS: u16 = 8;
/// `rxqueue`: a queue does not exist. Insert 1 is the name.
pub const QUEUE_EXIST: u16 = 10;
/// `rxqueue`: the command's syntax is incorrect.
pub const QUEUE_SYNTAX: u16 = 16;

/// The text of message `999.sub` with `inserts` in place of `&1`, `&2`, ... Bytes that are not
/// UTF-8 are replaced, as the terminal shows them.
///
/// Panics for a subcode the catalogue lacks: the constants above are all catalogued.
pub fn message(sub: u16, inserts: &[&[u8]]) -> String {
    let entry = lookup(MAJOR, sub).expect("a catalogued utility message");
    String::from_utf8_lossy(&substitute(entry.text, inserts)).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [u16; 12] = [
        SUBCOM_GENERAL,
        SUBCOM_REGISTER,
        SUBCOM_DROP,
        SUBCOM_LOAD,
        SUBCOM_QUERY,
        QUEUE_NOT_INITIALISED,
        QUEUE_SIZE,
        QUEUE_NO_MEMORY,
        QUEUE_NAME,
        QUEUE_ACCESS,
        QUEUE_EXIST,
        QUEUE_SYNTAX,
    ];

    #[test]
    fn every_message_is_catalogued_and_distinct() {
        let mut seen = Vec::new();
        for sub in ALL {
            assert!(lookup(MAJOR, sub).is_some(), "999.{sub}");
            assert!(!seen.contains(&sub), "999.{sub} listed twice");
            seen.push(sub);
        }
    }

    #[test]
    fn a_message_is_the_catalogue_text_with_its_inserts() {
        for sub in ALL {
            let entry = lookup(MAJOR, sub).expect("catalogued");
            let with_none = message(sub, &[]);
            assert_eq!(
                with_none.as_bytes(),
                substitute::<&[u8]>(entry.text, &[]).as_slice(),
                "999.{sub}"
            );
        }
        // A named queue reaches the text at insert 1, and nothing is left of the marker.
        for sub in [QUEUE_NAME, QUEUE_EXIST] {
            let text = message(sub, &[b"qq-probe"]);
            assert!(text.contains("qq-probe"), "999.{sub}");
            assert!(!text.contains("&1"), "999.{sub}");
        }
        // A message that takes no insert ignores the one handed to it.
        assert_eq!(message(QUEUE_SIZE, &[b"x"]), message(QUEUE_SIZE, &[]));
    }
}
