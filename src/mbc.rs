
pub trait MBC {
	fn readrom(&self, a: u16) -> u8;
	fn readram(&self, a: u16) -> u8;
	fn writerom(&mut self, a: u16, v: u8);
	fn writeram(&mut self, a: u16, v: u8);
}

struct MBC0 {
	priv rom: ~[u8],
}

struct MBC1 {
	priv rom: ~[u8],
	priv ram: ~[u8],
	priv ram_on: bool,
	priv ram_mode: bool,
	priv rombank: u32,
	priv rambank: u32,
}

pub fn get_mbc(data: ~[u8]) -> ~MBC {
	if data.len() < 0x149 { fail!("Rom size to small"); }
	let ramsize = match data[0x149] {
		1 => 0x800,
		2 => 0x2000,
		3 => 0x8000,
		_ => 0,
	};
	match data[0x147] {
		0x00 => (~MBC0 { rom: data }) as ~MBC,
		0x01 .. 0x03 => (~MBC1 {
			rom: data,
			ram: ::std::vec::from_elem(ramsize, 0u8),
			ram_on: false,
			ram_mode: false,
			rombank: 1,
			rambank: 0,
		}) as ~MBC,
		m => fail!("Unsupported MBC type: {:02X}", m),
	}
}

impl MBC for MBC0 {
	fn readrom(&self, a: u16) -> u8 { self.rom[a] }
	fn readram(&self, _a: u16) -> u8 { 0 }
	fn writerom(&mut self, _a: u16, _v: u8) { () }
	fn writeram(&mut self, _a: u16, _v: u8) { () }
}

impl MBC for MBC1 {
	fn readrom(&self, a: u16) -> u8 {
		if a < 0x4000 { self.rom[a] }
		else { self.rom[self.rombank * 0x4000 | ((a as u32) & 0x3FFF) ] }
	}
	fn readram(&self, a: u16) -> u8 {
		if !self.ram_on || !self.ram_mode { 0 }
		else { self.ram[self.rambank * 0x2000 | a as u32] }
	}

	fn writerom(&mut self, a: u16, v: u8) {
		match a {
			0x0000 .. 0x1FFF => { self.ram_on = (v == 0x0A); },
			0x2000 .. 0x3FFF => {
				self.rombank = (self.rombank & 0x60) | match (v as u32) & 0x1F { 0 => 1, n => n }
			},
			0x4000 .. 0x5FFF => {
				if !self.ram_mode {
					self.rombank = self.rombank & 0x1F | (((v as u32) & 0x03) << 5)
				} else {
					self.rambank = (v as u32) & 0x03;
				}
			},
			0x6000 .. 0x7FFF => { self.ram_mode = (v & 0x01) == 0x01; },
			_ => fail!("Could not write to {:04X} (MBC1)", a),
		}
	}

	fn writeram(&mut self, a: u16, v: u8) {
		if !self.ram_on || !self.ram_mode { return }
		self.ram[self.rambank * 0x2000 | a as u32] = v;
	}
}
