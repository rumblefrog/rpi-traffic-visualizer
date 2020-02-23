use segdisplay::SegDisplay;

use gpio_cdev::errors::Error;

fn main() -> Result<(), Error> {
    let mut seg_display = SegDisplay::new()?;

    let mut i = 0;

    loop {
        seg_display.write_int(i)?;

        i += 1;
    }

    // seg_display.write_int(TEST_VAL)?;

    // Ok(())
}