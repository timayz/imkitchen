pub mod contact;
pub mod mealplan;
pub mod recipe;
pub mod shopping;

use evento::Executor;

#[derive(Clone)]
pub struct Core<E: Executor> {
    pub recipe: recipe::Module<E>,
    pub mealplan: mealplan::Module<E>,
    pub shopping: shopping::Module<E>,
    pub contact: contact::Module<E>,
}

impl<E: Executor> Core<E> {
    pub fn new(state: imkitchen_shared::State<E>) -> Self
    where
        imkitchen_shared::State<E>: Clone,
    {
        Self {
            recipe: recipe::Module::new(state.clone()),
            mealplan: mealplan::Module::new(state.clone()),
            shopping: shopping::Module::new(state.clone()),
            contact: contact::Module::new(state),
        }
    }
}
