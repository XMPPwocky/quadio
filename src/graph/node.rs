use core::any::Any;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub enum ElementType {
    Float,   // f32
    Complex, // Complex32
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SocketDescriptor {
    pub label: String,

    pub element_type: ElementType,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NodeDescriptor {
    pub input_sockets: Vec<SocketDescriptor>,
    pub output_sockets: Vec<SocketDescriptor>,
}

pub trait NodeIO<'a> {
    fn from_parts(inputs: &'a [&dyn Any], outputs: &'a mut [&mut dyn Any]) -> Self;

    fn get_descriptor() -> NodeDescriptor;
}

pub trait Node<Ctx>: Send + Sync + 'static {
    type Io<'a>: NodeIO<'a>;

    fn run(&mut self, ctx: &Ctx, io: Self::Io<'_>);
}

pub trait DynamicNode<Ctx>: Send + Sync + 'static {
    fn get_descriptor(&self) -> NodeDescriptor;
    fn run<'io>(&mut self, ctx: &Ctx, inputs: &'io [&dyn Any], outputs: &'io mut [&mut dyn Any]);
}

impl<Ctx, T> DynamicNode<Ctx> for T
where
    T: Node<Ctx>,
{
    fn get_descriptor(&self) -> NodeDescriptor {
        T::Io::get_descriptor()
    }
    fn run<'io>(&mut self, ctx: &Ctx, inputs: &'io [&dyn Any], outputs: &'io mut [&mut dyn Any]) {
        let io = T::Io::from_parts(inputs, outputs);

        <T as Node<_>>::run(self, ctx, io);
    }
}
