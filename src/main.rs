use glutin::config::ConfigTemplateBuilder;
use meinkraft::app::App;
use std::error::Error;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let template = ConfigTemplateBuilder::new().with_alpha_size(0);
    let mut app = App::new(template);
    event_loop.run_app(&mut app)?;

    app.exit_state
}
