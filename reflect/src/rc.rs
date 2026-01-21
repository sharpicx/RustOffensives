pub struct State {
    state: [u8; 256],
    i: u8,
    j: u8,
}

impl State {
    pub fn create(key: &[u8]) -> Self {
        let mut rc4 = Self {
            state: [0; 256],
            i: 0,
            j: 0,
        };
        for i in 0..256 {
            rc4.state[i] = i as u8;
        }
        let mut j: u8 = 0;
        for i in 0..256 {
            j = j
                .wrapping_add(rc4.state[i])
                .wrapping_add(key[i % key.len()]);
            rc4.state.swap(i, j as usize);
        }
        rc4
    }

    pub fn next(&mut self) -> u8 {
        self.i = self.i.wrapping_add(1);
        self.j = self.j.wrapping_add(self.state[self.i as usize]);
        self.state.swap(self.i as usize, self.j as usize);
        let index = self.state[self.i as usize].wrapping_add(self.state[self.j as usize]);
        self.state[index as usize]
    }

    pub fn apply(&mut self, data: &mut [u8]) {
        for byte in data.iter_mut() {
            *byte ^= self.next();
        }
    }
}
