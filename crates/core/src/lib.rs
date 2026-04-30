mod command;
pub mod contact;
mod date;
pub mod mealplan;
pub mod recipe;
pub mod shopping;

pub use command::*;
pub use date::*;

use evento::Executor;

#[derive(Clone)]
pub struct State<E: Executor> {
    pub executor: E,
    pub read_db: sqlx::SqlitePool,
    pub write_db: sqlx::SqlitePool,
}

#[derive(Clone)]
pub struct Core<E: Executor> {
    pub recipe: recipe::Module<E>,
    pub mealplan: mealplan::Module<E>,
    pub shopping: shopping::Module<E>,
    pub contact: contact::Module<E>,
}

impl<E: Executor> Core<E> {
    pub fn new(state: State<E>) -> Self
    where
        State<E>: Clone,
    {
        Self {
            recipe: recipe::Module::new(state.clone()),
            mealplan: mealplan::Module::new(state.clone()),
            shopping: shopping::Module::new(state.clone()),
            contact: contact::Module::new(state),
        }
    }
}
