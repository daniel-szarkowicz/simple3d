use std::{any::TypeId, collections::HashMap, rc::Rc};

use crate::context::Context;

// pub struct ShaderManager {
//     shaders: HashMap<TypeId, ShaderId>,
//     context: Rc<Context>,
// }

// impl ShaderManager {
//     pub fn new(context: Rc<Context>) -> Self {
//         Self {
//             shaders: HashMap::new(),
//             context,
//         }
//     }

//     // pub fn get_or_insert<T: ShaderProvider>(&mut self, _: T) -> ShaderId {
//     //     *self
//     //         .shaders
//     //         .entry(TypeId::of::<T>())
//     //         .or_insert_with(|| self.context.load_shader(T::create_shader()))
//     // }
// }

// pub struct Shader {
//     pub name: String,
// }

// pub trait ShaderProvider: 'static + Copy {
//     fn create_shader() -> Shader;
// }

// #[derive(Clone, Copy)]
// pub struct DefaultShader;

// impl ShaderProvider for DefaultShader {
//     fn create_shader() -> Shader {
//         Shader {
//             name: "Default".to_owned(),
//         }
//     }
// }
