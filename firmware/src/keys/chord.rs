use embassy_time::{Duration, Instant};

pub type Key = (u8, u8);
pub const CHORD_TIMEOUT: Duration = Duration::from_millis(40);

pub struct Chord {
    pub key_map: &'static phf::Map<[u8; 2], usize>,
    pub key_states: &'static mut [bool],
    pub is_active: bool,
    pub action: &'static [Key],
}

impl Chord {
    fn process(&mut self, event: keyberon::layout::Event) -> Option<bool> {
        let coord = event.coord();
        let coord = [coord.0, coord.1];
        let Some(&state_idx) = self.key_map.get(&coord) else { return None; };

        self.key_states[state_idx] = event.is_press();

        let was_active = self.is_active;
        self.is_active = self.key_states.iter().all(|s| *s);

        if was_active != self.is_active {
            Some(self.is_active)
        } else {
            None
        }
    }

    fn clear_if_inactive(&mut self) {
        if !self.is_active {
            self.key_states.fill(false);
        }
    }
}

pub struct Chorder {
    pub key_chord_map: &'static phf::Map<[u8; 2], &'static [usize]>,
    pub chords: &'static mut [Chord],
}

pub struct ChordingEngine {
    chorder: Chorder,
    held_keys: heapless::Vec<Key, 16>,

    // after firing a release of a chord, ignore the following key releases
    ignored_releases: heapless::Vec<Key, 16>,
    last_press: Instant,
}

impl ChordingEngine {
    pub fn new(chorder: Chorder) -> Self {
        Self {
            chorder,
            held_keys: heapless::Vec::new(),
            ignored_releases: heapless::Vec::new(),
            last_press: Instant::now(),
        }
    }

    pub fn purge(&mut self) -> heapless::Vec<Key, 16> {
        for &(x, y) in &self.held_keys {
            if let Some(appropriate_chords) = self.chorder.key_chord_map.get(&[x, y]) {
                for &chord_idx in appropriate_chords.iter() {
                    let chord = &mut self.chorder.chords[chord_idx];

                    chord.process(keyberon::layout::Event::Release(x, y));
                }
            }
        }

        for chord in &mut *self.chorder.chords {
            chord.clear_if_inactive();
        }

        core::mem::replace(&mut self.held_keys, heapless::Vec::new())
    }

    pub fn tick(&mut self) -> heapless::Vec<Key, 16> {
        let now = Instant::now();

        if now.duration_since(self.last_press) > CHORD_TIMEOUT {
            // ran out of time, release all the currently pressed keys

            return self.purge();
        }

        heapless::Vec::new()
    }

    /// called on every event
    pub fn process(
        &mut self,
        event: keyberon::layout::Event,
    ) -> heapless::Vec<keyberon::layout::Event, 16> {
        let coord = event.coord();
        let coord = [coord.0, coord.1];

        if event.is_press() {
            // while we shouldn't see any presses until a release, remove a key
            // from ignored_releases if we see a press of it, just in case
            self.ignored_releases.retain(|&e| e != event.coord());
        }

        if let Some(appropriate_chords) = self.chorder.key_chord_map.get(&coord) {
            for &chord_idx in appropriate_chords.iter() {
                let chord = &mut self.chorder.chords[chord_idx];

                if let Some(active) = chord.process(event) {
                    match (active, event.is_press()) {
                        (true, true) => {
                            // chord became active with the key, clear out
                            // held_keys and emit the chord
                            self.held_keys.clear();

                            return heapless::Vec::from_iter(
                                chord
                                    .action
                                    .iter()
                                    .map(|(x, y)| keyberon::layout::Event::Press(*x, *y)),
                            );
                        }
                        (false, false) => {
                            // chord became inactive with this depress, emit the
                            // release of its action and add the unpressed keys
                            // to ignored_releases
                            for (&[x, y], &idx) in chord.key_map {
                                if chord.key_states[idx] {
                                    let _ = self.ignored_releases.push((x, y));
                                }
                            }

                            return heapless::Vec::from_iter(
                                chord
                                    .action
                                    .iter()
                                    .map(|(x, y)| keyberon::layout::Event::Release(*x, *y)),
                            );
                        }
                        _ => {
                            // shouldn't be possible
                        }
                    }
                }
            }

            // event was noted but didn't result in an activation (we didn't return)
            //
            // if it's a press we want to add it to held_keys so that they can
            // be emitted after the timeout and if it's a release we just emit
            // it straight away

            if let keyberon::layout::Event::Press(x, y) = event {
                let _ = self.held_keys.push((x, y));
            } else {
                if let Some(idx) = self.held_keys.iter().position(|&k| k == event.coord()) {
                    // if the key was in held_keys, remove it so we don't get double releases, and emit the press-release immediately
                    self.held_keys.remove(idx);
                    let (x, y) = event.coord();
                    return heapless::Vec::from_iter([keyberon::layout::Event::Press(x, y), event]);
                }

                if let Some(idx) = self
                    .ignored_releases
                    .iter()
                    .position(|&e| e == event.coord())
                {
                    // if the released key exists in ignored_releases then ignore it once
                    self.ignored_releases.remove(idx);
                } else {
                    // otherwise emit the release
                    return heapless::Vec::from_iter(core::iter::once(event));
                }
            }

            self.last_press = Instant::now();

            heapless::Vec::new()
        } else {
            // event applies to no chords, just emit it as is

            self.purge()
                .into_iter()
                .map(|(x, y)| keyberon::layout::Event::Release(x, y))
                .chain(core::iter::once(event))
                .collect()
        }
    }
}
