use winit::{
    application::ApplicationHandler,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
};

use crate::{
    canvas::Canvas,
    context::{self, Context},
};

type Update<State> = fn(&mut State) -> ();
type Draw<State> = fn(&State, &mut Canvas) -> ();

pub struct App<State> {
    state: State,
    update: Update<State>,
    draw: Draw<State>,
    context: Option<Context>,
    proxy: Option<EventLoopProxy<Context>>,
}

impl<State> App<State> {
    pub fn new(state: State, update: Update<State>, draw: Draw<State>) -> Self {
        Self {
            state,
            update,
            draw,
            context: None,
            proxy: None,
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::with_user_event().build().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        self.proxy = Some(event_loop.create_proxy());
        event_loop.run_app(&mut self).unwrap();
    }
}
impl<State: AppState> App<State> {
    pub fn run_with(state: State) {
        let app = App::new(state, State::update, State::draw);
        app.run()
    }
}

pub trait AppState {
    fn update(&mut self);
    fn draw(&self, canvas: &mut Canvas);
}

impl<State> ApplicationHandler<Context> for App<State> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // self.context = Some(Context::new(event_loop));
        let Some(proxy) = self.proxy.take() else {
            log::warn!("resumed called, but proxy was already taken!");
            return;
        };
        let create_context = async {
            proxy.send_event(Context::new(event_loop).await).unwrap();
        };
        pollster::block_on(create_context);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(context) = self.context.as_mut() else {
            log::warn!("window_event called with unitialized context!");
            return;
        };
        match &event {
            winit::event::WindowEvent::Resized(new_size) => {
                context.resize(new_size);
            }
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::RedrawRequested => {
                context.update_camera(1.0 / 60.0);
                (self.update)(&mut self.state);
                let mut canvas = context.create_canvas();
                (self.draw)(&self.state, &mut canvas);
                let commands = canvas.commands;
                context.render(commands);
            }
            _ => (),
        }
        context.event(&winit::event::Event::<()>::WindowEvent {
            window_id,
            event,
        });
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        self.context.as_mut().unwrap().event(
            &winit::event::Event::<()>::DeviceEvent { device_id, event },
        );
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.context = None;
    }

    fn user_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        context: Context,
    ) {
        context.request_redraw();
        self.context = Some(context);
    }
}
