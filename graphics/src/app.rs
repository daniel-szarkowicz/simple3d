use winit::event_loop::EventLoop;

use crate::canvas::Canvas;

type Update<State> = fn(&mut State) -> ();
type Draw<State> = fn(&State, &mut Canvas) -> ();

pub struct App<State> {
    user_state: State,
    update: Update<State>,
    draw: Draw<State>,
}

impl<State> App<State> {
    pub fn new(
        user_state: State,
        update: Update<State>,
        draw: Draw<State>,
    ) -> Self {
        Self {
            user_state,
            update,
            draw,
        }
    }

    pub fn run(self) {
        let event_loop = EventLoop::new().unwrap();
        let mut app = crate::backend::winit::WinitApp::new(
            self.user_state,
            self.update,
            self.draw,
        );
        event_loop.run_app(&mut app).unwrap();
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
