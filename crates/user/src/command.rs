use evento::Executor;

pub struct Command<E: Executor + Clone>(E);

impl<E: Executor + Clone> Command<E> {}
