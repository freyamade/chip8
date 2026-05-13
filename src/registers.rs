pub struct Registers {
    pub i: usize,
    pub v0: u8,
    pub v1: u8,
    pub v2: u8,
    pub v3: u8,
    pub v4: u8,
    pub v5: u8,
    pub v6: u8,
    pub v7: u8,
    pub v8: u8,
    pub v9: u8,
    pub va: u8,
    pub vb: u8,
    pub vc: u8,
    pub vd: u8,
    pub ve: u8,
    pub vf: u8,
}

impl Registers {
    pub fn new() -> Registers {
        return Registers { i: 0, v0: 0, v1: 0, v2: 0, v3: 0, v4: 0, v5: 0, v6: 0, v7: 0, v8: 0, v9: 0, va: 0, vb: 0, vc: 0, vd: 0, ve: 0, vf: 0 };
    }

    pub fn get(&self, register: u8) -> u8 {
        match register {
            0x0 => {return self.v0}
            0x1 => {return self.v1}
            0x2 => {return self.v2}
            0x3 => {return self.v3}
            0x4 => {return self.v4}
            0x5 => {return self.v5}
            0x6 => {return self.v6}
            0x7 => {return self.v7}
            0x8 => {return self.v8}
            0x9 => {return self.v9}
            0xa => {return self.va}
            0xb => {return self.vb}
            0xc => {return self.vc}
            0xd => {return self.vd}
            0xe => {return self.ve}
            0xf => {return self.vf}
            _ => {return 0}
        }
    }

    pub fn set(&mut self, register: u8, value: u8) {
        match register {
            0x0 => {self.v0 = value}
            0x1 => {self.v1 = value}
            0x2 => {self.v2 = value}
            0x3 => {self.v3 = value}
            0x4 => {self.v4 = value}
            0x5 => {self.v5 = value}
            0x6 => {self.v6 = value}
            0x7 => {self.v7 = value}
            0x8 => {self.v8 = value}
            0x9 => {self.v9 = value}
            0xa => {self.va = value}
            0xb => {self.vb = value}
            0xc => {self.vc = value}
            0xd => {self.vd = value}
            0xe => {self.ve = value}
            0xf => {self.vf = value}
            _ => {}
        }
    }

    pub fn add(&mut self, register: u8, value: u8) {
        match register {
            0x0 => {self.v0 = self.v0.wrapping_add(value)}
            0x1 => {self.v1 = self.v1.wrapping_add(value)}
            0x2 => {self.v2 = self.v2.wrapping_add(value)}
            0x3 => {self.v3 = self.v3.wrapping_add(value)}
            0x4 => {self.v4 = self.v4.wrapping_add(value)}
            0x5 => {self.v5 = self.v5.wrapping_add(value)}
            0x6 => {self.v6 = self.v6.wrapping_add(value)}
            0x7 => {self.v7 = self.v7.wrapping_add(value)}
            0x8 => {self.v8 = self.v8.wrapping_add(value)}
            0x9 => {self.v9 = self.v9.wrapping_add(value)}
            0xa => {self.va = self.va.wrapping_add(value)}
            0xb => {self.vb = self.vb.wrapping_add(value)}
            0xc => {self.vc = self.vc.wrapping_add(value)}
            0xd => {self.vd = self.vd.wrapping_add(value)}
            0xe => {self.ve = self.ve.wrapping_add(value)}
            0xf => {self.vf = self.vf.wrapping_add(value)}
            _ => {}
       }
    }
}