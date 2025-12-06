use std::collections::{HashMap, HashSet};

use bincode::{Decode, Encode};
use imkitchen_shared::Event;

use crate::{Checked, Generated, Resetted, Unchecked};

#[derive(Default, Encode, Decode, Clone, Debug)]
pub struct Shopping {
    pub user_id: String,
    pub checked: HashMap<u64, HashSet<String>>,
    pub ingredients: HashMap<u64, HashSet<String>>,
}

#[evento::aggregator]
impl Shopping {
    async fn handle_generated(&mut self, event: Event<Generated>) -> anyhow::Result<()> {
        self.user_id = event.metadata.trigger_by()?;

        let ingredients = event
            .data
            .ingredients
            .iter()
            .map(|i| i.name.to_owned())
            .collect();

        self.ingredients.insert(event.data.week, ingredients);
        self.checked.remove(&event.data.week);

        if self.ingredients.len() <= 5 {
            return Ok(());
        }

        let mut keys = self.ingredients.keys().cloned().collect::<Vec<_>>();
        keys.sort();

        if let Some(key) = keys.first() {
            self.ingredients.remove(key);
            self.checked.remove(key);
        }

        Ok(())
    }

    async fn handle_checked(&mut self, event: Event<Checked>) -> anyhow::Result<()> {
        let entry = self.checked.entry(event.data.week).or_default();
        entry.insert(event.data.ingredient);

        Ok(())
    }

    async fn handle_unchecked(&mut self, event: Event<Unchecked>) -> anyhow::Result<()> {
        let entry = self.checked.entry(event.data.week).or_default();
        entry.remove(&event.data.ingredient);
        if entry.is_empty() {
            self.checked.remove(&event.data.week);
        }

        Ok(())
    }

    async fn handle_resetted(&mut self, event: Event<Resetted>) -> anyhow::Result<()> {
        self.checked.remove(&event.data.week);

        Ok(())
    }
}
