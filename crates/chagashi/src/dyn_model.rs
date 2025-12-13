use matcha::{Cmd, InitInput, Model, Msg};

/// `matcha::Model` is not object-safe because it is `Sized` and returns `impl Display`.
/// This provides an object-safe wrapper so container components (e.g. `Flex`) can hold
/// heterogeneous `Model` implementations.
pub trait DynModel {
    fn init_box(self: Box<Self>, input: &InitInput) -> (Box<dyn DynModel>, Option<Cmd>);
    fn update_box(self: Box<Self>, msg: &Msg) -> (Box<dyn DynModel>, Option<Cmd>);
    fn view_string(&self) -> String;
}

struct DynModelAdapter<M: Model + 'static>(M);

impl<M: Model + 'static> DynModel for DynModelAdapter<M> {
    fn init_box(self: Box<Self>, input: &InitInput) -> (Box<dyn DynModel>, Option<Cmd>) {
        let (m, cmd) = self.0.init(input);
        (Box::new(DynModelAdapter(m)) as Box<dyn DynModel>, cmd)
    }

    fn update_box(self: Box<Self>, msg: &Msg) -> (Box<dyn DynModel>, Option<Cmd>) {
        let (m, cmd) = self.0.update(msg);
        (Box::new(DynModelAdapter(m)) as Box<dyn DynModel>, cmd)
    }

    fn view_string(&self) -> String {
        self.0.view().to_string()
    }
}

/// Convert a `matcha::Model` into `Box<dyn DynModel>` (used by containers like `Flex`).
pub fn boxed<M: Model + 'static>(model: M) -> Box<dyn DynModel> {
    Box::new(DynModelAdapter(model)) as Box<dyn DynModel>
}
