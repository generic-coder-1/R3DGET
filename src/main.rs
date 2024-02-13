use std::env;

use r3dget::application_state::application_state::ApplicationState;
use winit::event_loop::EventLoop;

fn main() {
    env::set_var("RUST_BACKTRACE", "0");
    let event_loop = EventLoop::new().expect("couldn't initalize a window");
    let application = pollster::block_on(ApplicationState::new(&event_loop));
    pollster::block_on(application.run(event_loop));
}
