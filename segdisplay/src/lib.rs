use sysfs_gpio::{
    Pin,
    Direction,
    Error,
};

// 4 Segment display
pub struct SegDisplay {
    pub pins: [Pin; 4],
}

impl SegDisplay {
    pub fn new(pins: &[u64; 4]) -> Result<Self, Error> {
        let sd = Self {
            pins: [
                Pin::new(pins[0]),
                Pin::new(pins[1]),
                Pin::new(pins[2]),
                Pin::new(pins[3]),
            ],
        };

        for pin in &sd.pins {
            pin.export()?;
            pin.set_direction(Direction::Out)?;
            pin.set_value(1)?;
        }

        Ok(sd)
    }

    pub fn select_position(&mut self, value: u8) -> Result<(), Error> {
        self.pins[0].set_value( !(value & 0x08) )?;
        self.pins[1].set_value( !(value & 0x04) )?;
        self.pins[2].set_value( !(value & 0x02) )?;
        self.pins[3].set_value( !(value & 0x01) )?;

        Ok(())
    }
}

pub struct ShiftRegister {
    // Shift register clock
    pub srclk: Pin,

    // Register clock / latch
    pub rclk: Pin,

    // Serial input
    pub ser: Pin,
}

impl ShiftRegister {
    pub fn new(srclk: u64, rclk: u64, ser: u64) -> Result<Self, Error> {
        let sr = Self {
            srclk: Pin::new(srclk),
            rclk: Pin::new(rclk),
            ser: Pin::new(ser),
        };

        sr.srclk.export()?;
        sr.srclk.set_direction(Direction::Out)?;

        sr.rclk.export()?;
        sr.rclk.set_direction(Direction::Out)?;

        sr.ser.export()?;
        sr.ser.set_direction(Direction::Out)?;

        Ok(sr)
    }

    pub fn write_u8(&mut self, value: u8) -> Result<(), Error> {
        for i in 0..8 {
            // Write value to current cell
            self.ser.set_value(value & (1 << i))?;

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
