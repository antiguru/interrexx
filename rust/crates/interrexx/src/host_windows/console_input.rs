//! Shared state for Windows console input.

#[derive(Default)]
pub(crate) struct ConsoleInput {
    pub(crate) bytes: std::collections::VecDeque<u8>,
    pub(crate) high_surrogate: Option<u16>,
}

impl ConsoleInput {
    pub(crate) fn character(&mut self, unit: u16) -> Option<([u16; 2], usize)> {
        if (0xd800..=0xdbff).contains(&unit) {
            self.high_surrogate = Some(unit);
            return None;
        }
        let high = self.high_surrogate.take();
        if (0xdc00..=0xdfff).contains(&unit)
            && let Some(high) = high
        {
            return Some(([high, unit], 2));
        }
        Some(([unit, 0], 1))
    }
}

#[cfg(test)]
mod tests {
    use super::ConsoleInput;
    use std::sync::Mutex;

    #[test]
    fn a_second_worker_completes_the_first_workers_character() {
        let input = Mutex::new(ConsoleInput::default());
        std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    let mut input = input.lock().unwrap();
                    assert!(input.character(0xd83d).is_none());
                    assert!(input.bytes.is_empty());
                })
                .join()
                .unwrap();
            scope
                .spawn(|| {
                    let mut input = input.lock().unwrap();
                    let (units, count) = input.character(0xde00).expect("a complete character");
                    assert_eq!(String::from_utf16(&units[..count]).unwrap(), "\u{1f600}");
                    assert!(input.high_surrogate.is_none());
                })
                .join()
                .unwrap();
        });
    }
}
