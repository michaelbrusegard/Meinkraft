use glutin::config::ConfigTemplateBuilder;
use glutin_winit::DisplayBuilder;
use meinkraft::app;
use std::error::Error;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::Window;

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let template = ConfigTemplateBuilder::new().with_alpha_size(8);

    let display_builder = DisplayBuilder::new()
        .with_window_attributes(Some(Window::default_attributes().with_title("Meinkraft")));

    let mut app = app::App::new(template, display_builder);
    event_loop.run_app(&mut app)?;

    app.exit_state
}
