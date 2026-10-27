use evento::{Executor, ProjectionAggregate};
use imkitchen_types::dish::{Changed, Created};
use imkitchen_types::recipe::RecipeType;
use validator::Validate;

use crate::dish::query::Candidate;

#[derive(Validate, Clone, Debug)]
pub struct ComposeInput {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    /// Recipes of the dish, in any order; duplicates are ignored.
    pub recipe_ids: Vec<String>,
}

/// A validated composition: the dish's recipe ids in meal order (starter,
/// main, sides, dessert, drink, sauces).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Composition {
    pub recipe_ids: Vec<String>,
}

/// Check the one-main / one-of-each rule against the user's candidate pool
/// and return the ids in meal order.
///
/// - every id must be a candidate (the user's own recipe or a saved one);
/// - exactly one main course;
/// - at most one appetizer, dessert and beverage;
/// - any number of accompaniments and condiments, kept in the given order.
pub fn compose(recipe_ids: &[String], candidates: &[Candidate]) -> crate::Result<Composition> {
    let mut seen: Vec<&str> = Vec::with_capacity(recipe_ids.len());
    let mut appetizers = vec![];
    let mut mains = vec![];
    let mut accompaniments = vec![];
    let mut desserts = vec![];
    let mut beverages = vec![];
    let mut condiments = vec![];

    for id in recipe_ids {
        if seen.contains(&id.as_str()) {
            continue;
        }
        seen.push(id);

        let Some(candidate) = candidates.iter().find(|c| &c.id == id) else {
            crate::user!("Some recipes are not in your library");
        };

        match candidate.recipe_type.0 {
            RecipeType::Appetizer => appetizers.push(id.to_owned()),
            RecipeType::MainCourse => mains.push(id.to_owned()),
            RecipeType::Accompaniment => accompaniments.push(id.to_owned()),
            RecipeType::Dessert => desserts.push(id.to_owned()),
            RecipeType::Beverage => beverages.push(id.to_owned()),
            RecipeType::Condiment => condiments.push(id.to_owned()),
        }
    }

    if mains.len() != 1 {
        crate::user!("A dish needs exactly one main course");
    }
    if appetizers.len() > 1 {
        crate::user!("A dish can have only one appetizer");
    }
    if desserts.len() > 1 {
        crate::user!("A dish can have only one dessert");
    }
    if beverages.len() > 1 {
        crate::user!("A dish can have only one beverage");
    }

    let recipe_ids = appetizers
        .into_iter()
        .chain(mains)
        .chain(accompaniments)
        .chain(desserts)
        .chain(beverages)
        .chain(condiments)
        .collect();

    Ok(Composition { recipe_ids })
}

impl<E: Executor> super::Module<E> {
    async fn composition(&self, input: &ComposeInput, user_id: &str) -> crate::Result<Composition> {
        input.validate()?;
        let candidates = self
            .candidates_by_ids(user_id, input.recipe_ids.clone())
            .await?;
        compose(&input.recipe_ids, &candidates)
    }

    /// Compose a new dish from the user's candidate pool. Returns its id.
    pub async fn create(
        &self,
        input: ComposeInput,
        request_by: impl Into<String>,
    ) -> crate::Result<String> {
        let request_by = request_by.into();
        let composition = self.composition(&input, &request_by).await?;

        Ok(evento::create()
            .event(&Created {
                name: input.name.trim().to_owned(),
                recipe_ids: composition.recipe_ids,
            })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?)
    }

    /// Rename and/or recompose an owned dish. Nothing is written when the
    /// composition is unchanged.
    pub async fn update(
        &self,
        id: impl Into<String>,
        input: ComposeInput,
        request_by: impl Into<String>,
    ) -> crate::Result<()> {
        let request_by = request_by.into();
        let Some(dish) = self.load(id).await? else {
            crate::not_found!("dish");
        };
        if dish.owner_id != request_by {
            crate::forbidden!("not owner of dish");
        }

        let composition = self.composition(&input, &request_by).await?;
        let name = input.name.trim().to_owned();

        if dish.name == name && dish.recipe_ids == composition.recipe_ids {
            return Ok(());
        }

        dish.write()?
            .event(&Changed {
                name,
                recipe_ids: composition.recipe_ids,
            })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::compose;
    use crate::dish::query::Candidate;
    use imkitchen_types::recipe::RecipeType;

    fn candidate(id: &str, recipe_type: RecipeType) -> Candidate {
        Candidate {
            id: id.to_owned(),
            name: id.to_owned(),
            recipe_type: sqlx::types::Text(recipe_type),
            accepts_accompaniment: false,
        }
    }

    fn pool() -> Vec<Candidate> {
        vec![
            candidate("starter", RecipeType::Appetizer),
            candidate("starter-2", RecipeType::Appetizer),
            candidate("main", RecipeType::MainCourse),
            candidate("main-2", RecipeType::MainCourse),
            candidate("side-1", RecipeType::Accompaniment),
            candidate("side-2", RecipeType::Accompaniment),
            candidate("sweet", RecipeType::Dessert),
            candidate("drink", RecipeType::Beverage),
            candidate("sauce-1", RecipeType::Condiment),
            candidate("sauce-2", RecipeType::Condiment),
        ]
    }

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn orders_recipes_by_course_and_keeps_every_side_and_sauce() {
        let composition = compose(
            &ids(&[
                "sauce-2", "side-2", "drink", "main", "sauce-1", "sweet", "side-1", "starter",
            ]),
            &pool(),
        )
        .unwrap();
        assert_eq!(
            composition.recipe_ids,
            ids(&[
                "starter", "main", "side-2", "side-1", "sweet", "drink", "sauce-2", "sauce-1"
            ])
        );
    }

    #[test]
    fn a_main_course_alone_is_a_dish() {
        let composition = compose(&ids(&["main"]), &pool()).unwrap();
        assert_eq!(composition.recipe_ids, ids(&["main"]));
    }

    #[test]
    fn duplicates_are_ignored() {
        let composition = compose(&ids(&["main", "side-1", "main", "side-1"]), &pool()).unwrap();
        assert_eq!(composition.recipe_ids, ids(&["main", "side-1"]));
    }

    #[test]
    fn needs_exactly_one_main_course() {
        assert!(matches!(
            compose(&ids(&["side-1"]), &pool()),
            Err(crate::Error::User(m)) if m.contains("main course")
        ));
        assert!(matches!(
            compose(&ids(&["main", "main-2"]), &pool()),
            Err(crate::Error::User(m)) if m.contains("main course")
        ));
        assert!(matches!(
            compose(&[], &pool()),
            Err(crate::Error::User(m)) if m.contains("main course")
        ));
    }

    #[test]
    fn single_courses_cannot_repeat() {
        assert!(matches!(
            compose(&ids(&["main", "starter", "starter-2"]), &pool()),
            Err(crate::Error::User(m)) if m.contains("appetizer")
        ));
    }

    #[test]
    fn rejects_recipes_outside_the_pool() {
        assert!(matches!(
            compose(&ids(&["main", "someone-elses"]), &pool()),
            Err(crate::Error::User(m)) if m.contains("library")
        ));
    }
}
