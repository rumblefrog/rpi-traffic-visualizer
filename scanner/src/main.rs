use segdisplay::{
    SegDisplay,
    ShiftRegister,
};

use sysfs_gpio::Error;

const TEST_VAL: u32 = 4158u32;

fn main() -> Result<(), Error> {
    let mut shift_register = ShiftRegister::new(1, 4, 5)?;
    let mut seg_display = SegDisplay::new(&[0, 2, 3, 12])?;

    shift_register.write_u8(0xff)?;
    seg_display.select_position(0x01)?;
    shift_register.write_u8((TEST_VAL % 10) as u8)?;

    shift_register.write_u8(0xff)?;
    seg_display.select_position(0x02)?;
    shift_register.write_u8((TEST_VAL % 100 / 10) as u8)?;

    shift_register.write_u8(0xff)?;
    seg_display.select_position(0x04)?;
    shift_register.write_u8((TEST_VAL % 1000 / 100) as u8)?;

    shift_register.write_u8(0xff)?;
    seg_display.select_position(0x08)?;
    shift_register.write_u8((TEST_VAL % 10000 / 1000) as u8)?;

    Ok(())
}