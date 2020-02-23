use gpio_cdev::{
    Chip,
    LineHandle,
    LineRequestFlags,
};

use gpio_cdev::errors::Error;

// 4 Segment display
pub struct SegDisplay {
    pub chip: Chip,

    pub register: ShiftRegister,

    pub lines: [LineHandle; 4],
}

impl SegDisplay {
    const COMMON_ANODES: [u8; 10] = [
        0xc0,
        0xf9,
        0xa4,
        0xb0,
        0x99,
        0x92,
        0x82,
        0xf8,
        0x80,
        0x90,
    ];

    pub fn new() -> Result<Self, Error> {
        let mut chip = Chip::new("/dev/gpiochip0")?;

        let register = ShiftRegister::new(&mut chip)?;

        let l1 = chip.get_line(10)?.request(LineRequestFlags::OUTPUT, 1, "segdisplay")?;
        let l2 = chip.get_line(22)?.request(LineRequestFlags::OUTPUT, 1, "segdisplay")?;
        let l3 = chip.get_line(27)?.request(LineRequestFlags::OUTPUT, 1, "segdisplay")?;
        let l4 = chip.get_line(17)?.request(LineRequestFlags::OUTPUT, 1, "segdisplay")?;

        Ok(Self {
            chip,
            register,
            lines: [l1, l2, l3, l4],
        })
    }

    pub fn select_position(&mut self, value: u8) -> Result<(), Error> {
        for i in 0..4 {
            self.lines[i as usize].set_value(1)?;
        }

        self.lines[value as usize].set_value(0)?;

        Ok(())
    }

    pub fn write_int(&mut self, value: u32) -> Result<(), Error> {
        self.register.purge()?;
        self.select_position(0)?;
        self.register.write_u8(SegDisplay::COMMON_ANODES[(value % 10) as usize])?;

        self.register.purge()?;
        self.select_position(1)?;
        self.register.write_u8(SegDisplay::COMMON_ANODES[(value % 100 / 10) as usize])?;

        self.register.purge()?;
        self.select_position(2)?;
        self.register.write_u8(SegDisplay::COMMON_ANODES[(value % 1000 / 100) as usize])?;

        self.register.purge()?;
        self.select_position(3)?;
        self.register.write_u8(SegDisplay::COMMON_ANODES[(value % 10000 / 1000) as usize])?;

        Ok(())
    }
}

pub struct ShiftRegister {
    // Shift register clock
    pub srclk: LineHandle,

    // Register clock / latch
    pub rclk: LineHandle,

    // Serial input
    pub ser: LineHandle,
}

impl ShiftRegister {
    pub fn new(chip: &mut Chip) -> Result<Self, Error> {
        Ok(Self {
            srclk: chip.get_line(18)?.request(LineRequestFlags::OUTPUT, 0, "segdisplay")?,
            rclk: chip.get_line(23)?.request(LineRequestFlags::OUTPUT, 0, "segdisplay")?,
            ser: chip.get_line(24)?.request(LineRequestFlags::OUTPUT, 0, "segdisplay")?,
        })
    }

    pub fn purge(&mut self) -> Result<(), Error> {
        for _i in 0..8 {
            self.ser.set_value(1)?;
            self.srclk.set_value(1)?;
            self.srclk.set_value(0)?;
        }

        self.rclk.set_value(1)?;
        self.rclk.set_value(0)?;

        Ok(())
    }

    pub fn write_u8(&mut self, value: u8) -> Result<(), Error> {
        for i in 0..8 {
            // Write value to current cell
            self.ser.set_value(0x80 & (value << i))?;

            print!("{:X} = {} ", value, 0x80 & (value << i));

            if i == 8 {
                println!();
            }

            // Shift cell over
            self.srclk.set_value(1)?;
            self.srclk.set_value(0)?;
        }

        // Pulse latch once
        self.rclk.set_value(1)?;
        self.rclk.set_value(0)?;

        Ok(())
    }
}
