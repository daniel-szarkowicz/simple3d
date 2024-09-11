use std::{rc::Rc, thread, time::Duration};

use winit::{event_loop::EventLoop, platform::x11::EventLoopBuilderExtX11};

use crate::canvas::Canvas;

type Update<State> = fn(&mut State) -> ();
type Draw<State> = fn(&mut Canvas, &State) -> ();

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

    pub fn run(self) -> ! {
        let event_loop = EventLoop::new().unwrap();
        let mut app = crate::backend::winit::WinitApp::new(
            self.user_state,
            self.update,
            self.draw,
        );
        event_loop.run_app(&mut app).unwrap();
        std::process::exit(0)
        // let context = Rc::new(Context::new());
        // let mut meshes = MeshManager::new(context.clone());
        // let mut shaders = ShaderManager::new(context.clone());
        // loop {
        //     (self.update)(&mut self.user_state);
        //     let mut canvas = Canvas::new(&mut meshes, &mut shaders);
        //     (self.draw)(&mut canvas, &self.user_state);
        //     let commands = canvas.commands;
        //     context.render(commands);
        //     thread::sleep(Duration::from_millis(100));
        // }
    }
}

// struct Renderer {}

// impl Renderer {
//     fn render(&mut self, commands: Vec<DrawCommand>) {
//         let _ = &commands;
//         // todo
//     }
// }
