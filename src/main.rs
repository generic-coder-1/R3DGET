use std::env;

use r3dget::run;

fn main() {
    env::set_var("RUST_BACKTRACE", "0");
    pollster::block_on(run());
}
