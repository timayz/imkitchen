//! Synthetic data for demo mode.
//!
//! Everything here builds the *real* template context structs from the
//! kitchen / menu / recipe / grocery web crates, so the demo renders the
//! exact same templates a signed-in premium user would see — just fed from
//! hand-authored placeholder data instead of the database.

use std::collections::HashSet;
use std::str::FromStr;

use evento::cursor::{Edge, PageInfo, ReadResult, Value};
use imkitchen_core::mealplan;
use imkitchen_core::mealplan::slot::SlotRow;
use imkitchen_core::recipe::favorite::Favorite;
use imkitchen_core::recipe::query::user::{SortBy, UserView, UserViewList};
use imkitchen_core::recipe::query::user_stat::UserStatView;
use imkitchen_types::mealplan::{DaySlotRecipe, DaySlotStatus};
use imkitchen_types::recipe::{
    Ingredient, IngredientCategory, IngredientUnit, Instruction, RecipeType,
};
use imkitchen_web_grocery::{AisleSection, GroceriesTemplate};
use imkitchen_web_kitchen::{CookingTemplate, KitchenTemplate, KitchenWeekDay};
use imkitchen_web_menu::{MenuBoardDay, MenuSlot, MenuTemplate};
use imkitchen_web_recipe::routes::detail::DetailTemplate;
use imkitchen_web_recipe::routes::index::{IndexTemplate as RecipesIndexTemplate, PageQuery};
use imkitchen_web_shared::auth::AuthUser;
use time::{Duration, OffsetDateTime};

const TZ: &str = "UTC";

fn ymd(date: OffsetDateTime) -> String {
    let fmt = time::macros::format_description!("[year]-[month]-[day]");
    date.format(&fmt).unwrap_or_default()
}

// ── Demo user ────────────────────────────────────────────────────────────

/// A premium, active demo account. Premium so every feature renders without
/// upsell banners; subscription is set far out so `is_premium()` holds.
pub fn demo_user() -> AuthUser {
    AuthUser::demo()
}

// ── Recipe catalog ───────────────────────────────────────────────────────

fn ing(name: &str, qty: u32, unit: Option<IngredientUnit>, cat: IngredientCategory) -> Ingredient {
    Ingredient {
        name: name.to_owned(),
        quantity: qty,
        unit,
        category: Some(cat),
    }
}

fn step(desc: &str, time_next: u16) -> Instruction {
    Instruction {
        description: desc.to_owned(),
        time_next,
    }
}

#[allow(clippy::too_many_arguments)]
fn recipe(
    id: &str,
    name: &str,
    rt: RecipeType,
    prep: u16,
    cook: u16,
    description: &str,
    advance_prep: &str,
    ingredients: Vec<Ingredient>,
    instructions: Vec<Instruction>,
) -> UserView {
    UserView {
        id: id.to_owned(),
        owner_id: "demo".to_owned(),
        owner_name: Some("imkitchen".to_owned()),
        recipe_type: sqlx::types::Text(rt),
        name: name.to_owned(),
        description: description.to_owned(),
        household_size: 4,
        prep_time: prep,
        cook_time: cook,
        ingredients: ingredients.into(),
        instructions: instructions.into(),
        accepts_accompaniment: true,
        advance_prep: advance_prep.to_owned(),
        is_shared: true,
        difficulty_score: prep + cook,
        ..Default::default()
    }
}

/// The full demo recipe library — also the source for meal-plan slots and the
/// shopping list, so every page stays internally consistent.
pub fn catalog() -> Vec<UserView> {
    use IngredientCategory::*;
    use IngredientUnit::*;
    use RecipeType::*;

    vec![
        recipe(
            "arroz-con-pollo",
            "Arroz con Pollo",
            MainCourse,
            20,
            45,
            "A one-pot Caribbean classic — saffron rice simmered with seared chicken, peppers and peas.",
            "Marinate the chicken in lime and garlic the night before.",
            vec![
                ing("Chicken thighs", 800, Some(G), Butcher),
                ing("Long-grain rice", 400, Some(G), Grocery),
                ing("Bell pepper", 2, None, FruitsAndVegetables),
                ing("Onion", 1, None, FruitsAndVegetables),
                ing("Garlic", 4, None, FruitsAndVegetables),
                ing("Chicken stock", 750, Some(ML), Grocery),
                ing("Frozen peas", 150, Some(G), Frozen),
            ],
            vec![
                step(
                    "Season the chicken thighs and sear skin-side down until golden. Set aside.",
                    0,
                ),
                step(
                    "Soften the onion, pepper and garlic in the same pan until fragrant.",
                    0,
                ),
                step(
                    "Stir in the rice and saffron, coating every grain in the oil.",
                    0,
                ),
                step(
                    "Pour in the stock, nestle the chicken back in, cover and simmer.",
                    25,
                ),
                step(
                    "Scatter the peas over the top, cover again and rest off the heat.",
                    5,
                ),
                step("Fluff with a fork, check the seasoning and serve.", 0),
            ],
        ),
        recipe(
            "coq-au-vin",
            "Coq au Vin",
            MainCourse,
            30,
            90,
            "Chicken braised low and slow in red wine with mushrooms, lardons and pearl onions.",
            "",
            vec![
                ing("Chicken legs", 1000, Some(G), Butcher),
                ing("Smoked lardons", 150, Some(G), Butcher),
                ing("Mushrooms", 250, Some(G), FruitsAndVegetables),
                ing("Pearl onions", 200, Some(G), FruitsAndVegetables),
                ing("Red wine", 500, Some(ML), Grocery),
                ing("Butter", 50, Some(G), DairyAndEggs),
            ],
            vec![
                step(
                    "Brown the lardons, then the chicken, in a heavy casserole.",
                    0,
                ),
                step("Deglaze with the red wine, scraping up every brown bit.", 0),
                step("Add the onions and mushrooms, cover and braise gently.", 60),
                step("Reduce the sauce until glossy and whisk in cold butter.", 0),
            ],
        ),
        recipe(
            "thai-green-curry",
            "Thai Green Curry",
            MainCourse,
            15,
            25,
            "Fragrant coconut curry with green paste, aubergine and Thai basil.",
            "",
            vec![
                ing("Chicken breast", 600, Some(G), Butcher),
                ing("Coconut milk", 400, Some(ML), Grocery),
                ing("Green curry paste", 60, Some(G), Grocery),
                ing("Aubergine", 1, None, FruitsAndVegetables),
                ing("Thai basil", 1, None, FruitsAndVegetables),
            ],
            vec![
                step(
                    "Fry the curry paste in a little coconut cream until it splits.",
                    0,
                ),
                step("Add the chicken and seal on all sides.", 0),
                step(
                    "Pour in the rest of the coconut milk and the aubergine, simmer.",
                    18,
                ),
                step(
                    "Finish with fish sauce, sugar and a handful of Thai basil.",
                    0,
                ),
            ],
        ),
        recipe(
            "spaghetti-bolognese",
            "Spaghetti Bolognese",
            MainCourse,
            15,
            120,
            "A deeply savoury ragù simmered for hours, tossed through al dente spaghetti.",
            "",
            vec![
                ing("Ground beef", 500, Some(G), Butcher),
                ing("Spaghetti", 400, Some(G), Grocery),
                ing("Chopped tomatoes", 800, Some(G), Grocery),
                ing("Carrot", 2, None, FruitsAndVegetables),
                ing("Parmesan", 80, Some(G), DairyAndEggs),
            ],
            vec![
                step(
                    "Soften the diced carrot, celery and onion — the soffritto base.",
                    0,
                ),
                step("Brown the beef hard until caramelised, not grey.", 0),
                step("Add the tomatoes and simmer low and slow.", 90),
                step(
                    "Cook the spaghetti, toss with the ragù and a little pasta water.",
                    0,
                ),
            ],
        ),
        recipe(
            "tomato-bruschetta",
            "Tomato Bruschetta",
            Appetizer,
            10,
            5,
            "Charred sourdough rubbed with garlic and piled with marinated tomatoes.",
            "",
            vec![
                ing("Sourdough", 4, None, Bakery),
                ing("Ripe tomatoes", 4, None, FruitsAndVegetables),
                ing("Basil", 1, None, FruitsAndVegetables),
                ing("Olive oil", 30, Some(ML), Grocery),
            ],
            vec![
                step("Dice and salt the tomatoes, then leave to drain.", 0),
                step("Grill the sourdough and rub each slice with raw garlic.", 0),
                step(
                    "Spoon the tomatoes on top and finish with basil and oil.",
                    0,
                ),
            ],
        ),
        recipe(
            "caesar-salad",
            "Caesar Salad",
            Appetizer,
            15,
            0,
            "Crisp romaine, garlicky croutons and a sharp anchovy dressing.",
            "",
            vec![
                ing("Romaine lettuce", 2, None, FruitsAndVegetables),
                ing("Parmesan", 60, Some(G), DairyAndEggs),
                ing("Bread", 100, Some(G), Bakery),
                ing("Anchovies", 30, Some(G), Grocery),
            ],
            vec![
                step("Toss the bread cubes in oil and bake until crisp.", 10),
                step("Whisk the dressing — anchovy, garlic, lemon, parmesan.", 0),
                step(
                    "Coat the romaine, add croutons and shave over more parmesan.",
                    0,
                ),
            ],
        ),
        recipe(
            "garlic-butter-rice",
            "Garlic Butter Rice",
            Accompaniment,
            5,
            20,
            "Fluffy rice enriched with toasted garlic butter.",
            "",
            vec![
                ing("Basmati rice", 300, Some(G), Grocery),
                ing("Butter", 40, Some(G), DairyAndEggs),
                ing("Garlic", 3, None, FruitsAndVegetables),
            ],
            vec![
                step("Toast the garlic in butter until just golden.", 0),
                step(
                    "Stir in the rinsed rice and the water, then simmer covered.",
                    15,
                ),
                step("Rest off the heat and fluff before serving.", 0),
            ],
        ),
        recipe(
            "roasted-vegetables",
            "Roasted Vegetables",
            Accompaniment,
            10,
            35,
            "Caramelised seasonal vegetables roasted with thyme.",
            "",
            vec![
                ing("Courgette", 2, None, FruitsAndVegetables),
                ing("Bell pepper", 2, None, FruitsAndVegetables),
                ing("Red onion", 1, None, FruitsAndVegetables),
                ing("Olive oil", 40, Some(ML), Grocery),
            ],
            vec![
                step("Chop everything into rough, even chunks.", 0),
                step("Toss with oil, thyme and seasoning on a hot tray.", 0),
                step("Roast until the edges char, tossing once halfway.", 30),
            ],
        ),
        recipe(
            "creme-brulee",
            "Crème Brûlée",
            Dessert,
            20,
            40,
            "Silky vanilla custard under a crackling caramel lid.",
            "Chill the baked custards overnight before torching.",
            vec![
                ing("Double cream", 500, Some(ML), DairyAndEggs),
                ing("Egg yolks", 6, None, DairyAndEggs),
                ing("Vanilla pod", 1, None, Grocery),
                ing("Caster sugar", 120, Some(G), Grocery),
            ],
            vec![
                step(
                    "Infuse the cream with vanilla, then whisk into the yolks and sugar.",
                    0,
                ),
                step("Bake the custards in a water bath until just set.", 35),
                step("Chill thoroughly, then torch a sugar crust on top.", 0),
            ],
        ),
        recipe(
            "chocolate-mousse",
            "Chocolate Mousse",
            Dessert,
            25,
            0,
            "Airy dark-chocolate mousse with a whisper of espresso.",
            "",
            vec![
                ing("Dark chocolate", 200, Some(G), SnacksAndConfectionery),
                ing("Eggs", 4, None, DairyAndEggs),
                ing("Caster sugar", 60, Some(G), Grocery),
            ],
            vec![
                step("Melt the chocolate gently and let it cool a little.", 0),
                step("Whip the whites to soft peaks with the sugar.", 0),
                step("Fold the whites into the chocolate and chill until set.", 0),
            ],
        ),
        recipe(
            "mint-lemonade",
            "Fresh Mint Lemonade",
            Beverage,
            10,
            0,
            "Cloudy, tart lemonade muddled with fresh mint.",
            "",
            vec![
                ing("Lemons", 6, None, FruitsAndVegetables),
                ing("Mint", 1, None, FruitsAndVegetables),
                ing("Sugar", 100, Some(G), Grocery),
            ],
            vec![
                step("Make a syrup with the sugar and a splash of water.", 0),
                step(
                    "Muddle the mint into the syrup, then add lemon juice and cold water.",
                    0,
                ),
            ],
        ),
        recipe(
            "chimichurri",
            "Chimichurri Sauce",
            Condiment,
            10,
            0,
            "Bright, herby Argentinian sauce for grilled meats.",
            "",
            vec![
                ing("Parsley", 1, None, FruitsAndVegetables),
                ing("Garlic", 3, None, FruitsAndVegetables),
                ing("Red wine vinegar", 50, Some(ML), Grocery),
                ing("Olive oil", 120, Some(ML), Grocery),
            ],
            vec![
                step("Finely chop the parsley, garlic and chilli.", 0),
                step("Stir through the vinegar and oil, then rest to meld.", 0),
            ],
        ),
    ]
}

/// Looks up a single recipe by id (used by the cooking screen).
pub fn find_recipe(id: &str) -> Option<UserView> {
    catalog().into_iter().find(|r| r.id == id)
}

fn dsr(id: &str, status: DaySlotStatus) -> DaySlotRecipe {
    let r = find_recipe(id).unwrap_or_default();
    DaySlotRecipe {
        id: r.id,
        name: r.name,
        prep_time: r.prep_time,
        cook_time: r.cook_time,
        advance_prep: r.advance_prep,
        status,
    }
}

// ── Meal-plan slots ──────────────────────────────────────────────────────

/// `(appetizer, main, accompaniment, dessert, beverage, condiment)` — recipe
/// ids for one day. Optional courses are `None`.
type DayPlan = (
    &'static str,
    &'static str,
    &'static str,
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
);

/// A repeating week of plans. Index by day-of-month so each calendar day is
/// deterministic.
const PLANS: &[DayPlan] = &[
    (
        "tomato-bruschetta",
        "arroz-con-pollo",
        "garlic-butter-rice",
        Some("creme-brulee"),
        Some("mint-lemonade"),
        None,
    ),
    (
        "caesar-salad",
        "coq-au-vin",
        "roasted-vegetables",
        Some("chocolate-mousse"),
        None,
        None,
    ),
    (
        "tomato-bruschetta",
        "thai-green-curry",
        "garlic-butter-rice",
        Some("creme-brulee"),
        None,
        Some("chimichurri"),
    ),
    (
        "caesar-salad",
        "spaghetti-bolognese",
        "roasted-vegetables",
        Some("chocolate-mousse"),
        Some("mint-lemonade"),
        None,
    ),
    (
        "tomato-bruschetta",
        "coq-au-vin",
        "garlic-butter-rice",
        Some("creme-brulee"),
        None,
        None,
    ),
    (
        "caesar-salad",
        "arroz-con-pollo",
        "roasted-vegetables",
        Some("chocolate-mousse"),
        None,
        Some("chimichurri"),
    ),
    (
        "tomato-bruschetta",
        "spaghetti-bolognese",
        "garlic-butter-rice",
        Some("creme-brulee"),
        Some("mint-lemonade"),
        None,
    ),
];

fn slot_for(day: OffsetDateTime) -> SlotRow {
    let plan = PLANS[(day.day() as usize) % PLANS.len()];
    let day_unix = day.unix_timestamp() as u64;

    SlotRow {
        day: day_unix,
        household_size: 4,
        main_course: dsr(plan.1, DaySlotStatus::Idle).into(),
        appetizer: Some(dsr(plan.0, DaySlotStatus::Idle).into()),
        accompaniment: Some(dsr(plan.2, DaySlotStatus::Idle).into()),
        dessert: plan.3.map(|id| dsr(id, DaySlotStatus::Idle).into()),
        beverage: plan.4.map(|id| dsr(id, DaySlotStatus::Idle).into()),
        condiment: plan.5.map(|id| dsr(id, DaySlotStatus::Idle).into()),
        generated_at: 0,
    }
}

fn meal_types_for(day: OffsetDateTime) -> Vec<RecipeType> {
    let plan = PLANS[(day.day() as usize) % PLANS.len()];
    let mut types = vec![
        RecipeType::Appetizer,
        RecipeType::MainCourse,
        RecipeType::Accompaniment,
    ];
    if plan.3.is_some() {
        types.push(RecipeType::Dessert);
    }
    if plan.4.is_some() {
        types.push(RecipeType::Beverage);
    }
    if plan.5.is_some() {
        types.push(RecipeType::Condiment);
    }
    types
}

// ── Kitchen page ─────────────────────────────────────────────────────────

pub fn kitchen() -> KitchenTemplate {
    let today = mealplan::now(TZ);
    let slot = slot_for(today);
    let plan = PLANS[(today.day() as usize) % PLANS.len()];
    let slot_recipe = find_recipe(plan.1);

    let slot_total_count = 3
        + slot.dessert.is_some() as u8
        + slot.beverage.is_some() as u8
        + slot.condiment.is_some() as u8;

    // Week strip — Mon..Sun anchored on today.
    let mut week_dates = mealplan::week_days_before(today);
    week_dates.push(today);
    week_dates.extend(mealplan::week_days_after(today));
    let today_u64 = mealplan::date_to_u64(today);

    let week_days: Vec<KitchenWeekDay> = week_dates
        .iter()
        .map(|d| KitchenWeekDay {
            date: ymd(*d),
            day_num: d.day(),
            weekday: d.weekday().to_string().chars().take(3).collect(),
            is_today: mealplan::date_to_u64(*d) == today_u64,
            meal_types: meal_types_for(*d),
        })
        .collect();

    // Tomorrow's prep reminders (premium-only section) — surface the recipes
    // that carry an advance-prep note.
    let tomorrow = today + Duration::days(1);
    let prep_remiders: Vec<DaySlotRecipe> = {
        let plan = PLANS[(tomorrow.day() as usize) % PLANS.len()];
        [Some(plan.1), Some(plan.0), plan.3]
            .into_iter()
            .flatten()
            .map(|id| dsr(id, DaySlotStatus::Idle))
            .filter(|r| !r.advance_prep.is_empty())
            .collect()
    };

    KitchenTemplate {
        user: demo_user(),
        slot: Some(slot),
        slot_recipe,
        slot_total_count,
        slot_completed_count: 0,
        prep_remiders: (!prep_remiders.is_empty()).then_some(prep_remiders),
        date: ymd(today),
        week_days,
        ..Default::default()
    }
}

// ── Cooking screen ───────────────────────────────────────────────────────

pub fn cooking(recipe_id: &str) -> CookingTemplate {
    let today = mealplan::now(TZ);
    let slot_recipe =
        find_recipe(recipe_id).unwrap_or_else(|| find_recipe("arroz-con-pollo").unwrap());

    // Start one step in so both Back and Next are visible.
    let coming_instructions: Vec<(usize, String)> = slot_recipe
        .instructions
        .iter()
        .enumerate()
        .skip(2)
        .map(|(p, i)| (p, i.description.to_owned()))
        .collect();
    let completed_instructions: Vec<(usize, String)> = slot_recipe
        .instructions
        .iter()
        .enumerate()
        .take(1)
        .map(|(p, i)| (p, i.description.to_owned()))
        .collect();
    let current_instruction = slot_recipe.instructions.get(1).map(|i| (1, i.clone()));

    CookingTemplate {
        slot_recipe,
        completed_instructions,
        coming_instructions,
        current_instruction,
        date: ymd(today),
    }
}

// ── Menu page ────────────────────────────────────────────────────────────

pub fn menu(date: Option<String>) -> MenuTemplate {
    let bounds = match date {
        Some(d) => mealplan::month_bounds_from_date(&d, TZ),
        None => mealplan::month_bounds_from_now(TZ),
    }
    .unwrap_or_else(|_| mealplan::month_bounds_from_now(TZ).expect("now bounds"));

    let (prev_month, next_month) = mealplan::prev_next_month(bounds.first).unwrap_or_default();

    let today = mealplan::now(TZ);
    let today_u64 = mealplan::date_to_u64(today);
    let is_past = mealplan::date_to_u64(bounds.date) < today_u64;

    // Mobile calendar list: leading weekday padding, then each in-month day.
    let mut menu_slots: Vec<MenuSlot> = mealplan::week_days_before(bounds.first)
        .iter()
        .map(|d| MenuSlot {
            day: d.day(),
            slot: None,
        })
        .collect();

    let mut d = bounds.first;
    while d <= bounds.last {
        menu_slots.push(MenuSlot {
            day: d.day(),
            slot: Some(slot_for(d)),
        });
        d += Duration::days(1);
    }
    for d in mealplan::week_days_after(bounds.last) {
        menu_slots.push(MenuSlot {
            day: d.day(),
            slot: None,
        });
    }

    // Desktop board: Mon..Sun padded grid grouped into weeks.
    let mut board_dates = mealplan::week_days_before(bounds.first);
    let mut d = bounds.first;
    while d <= bounds.last {
        board_dates.push(d);
        d += Duration::days(1);
    }
    board_dates.extend(mealplan::week_days_after(bounds.last));

    let bounds_month = bounds.date.month();
    let board_days: Vec<MenuBoardDay> = board_dates
        .iter()
        .map(|d| {
            let in_month = d.month() == bounds_month;
            let d_u64 = mealplan::date_to_u64(*d);
            MenuBoardDay {
                date: ymd(*d),
                weekday: d.weekday().to_string().chars().take(3).collect(),
                day_num: d.day(),
                is_today: d_u64 == today_u64,
                is_past: d_u64 < today_u64,
                is_in_month: in_month,
                slot: in_month.then(|| slot_for(*d)),
            }
        })
        .collect();

    let board_weeks: Vec<Vec<MenuBoardDay>> = board_days.chunks(7).map(|c| c.to_vec()).collect();

    let selected = slot_for(bounds.date);
    let selected_day = selected.day;

    MenuTemplate {
        user: demo_user(),
        slots: menu_slots,
        selected_slot: Some(selected),
        selected_day,
        current_date: ymd(bounds.date),
        is_past,
        first_month_day: bounds.first.unix_timestamp() as u64,
        prev_month,
        next_month,
        board_weeks,
        ..Default::default()
    }
}

// ── Groceries page ───────────────────────────────────────────────────────

fn aisle(name: &str, items: Vec<Ingredient>, checked: &HashSet<String>) -> AisleSection {
    let total = items.len();
    let checked_count = items.iter().filter(|i| checked.contains(&i.key())).count();
    let pct = (checked_count * 100).checked_div(total).unwrap_or(0);
    AisleSection {
        name: name.to_owned(),
        items,
        checked: checked_count,
        total,
        done: total > 0 && checked_count == total,
        pct,
    }
}

pub fn groceries() -> GroceriesTemplate {
    use IngredientCategory::*;
    use IngredientUnit::*;

    let produce = vec![
        ing("Bell pepper", 4, None, FruitsAndVegetables),
        ing("Onion", 3, None, FruitsAndVegetables),
        ing("Garlic", 1, None, FruitsAndVegetables),
        ing("Tomatoes", 6, None, FruitsAndVegetables),
        ing("Lemons", 6, None, FruitsAndVegetables),
        ing("Courgette", 2, None, FruitsAndVegetables),
        ing("Mint", 1, None, FruitsAndVegetables),
    ];
    let butcher = vec![
        ing("Chicken thighs", 800, Some(G), Butcher),
        ing("Chicken legs", 1000, Some(G), Butcher),
        ing("Ground beef", 500, Some(G), Butcher),
        ing("Smoked lardons", 150, Some(G), Butcher),
    ];
    let dairy = vec![
        ing("Double cream", 500, Some(ML), DairyAndEggs),
        ing("Butter", 90, Some(G), DairyAndEggs),
        ing("Parmesan", 140, Some(G), DairyAndEggs),
        ing("Eggs", 10, None, DairyAndEggs),
    ];
    let pantry = vec![
        ing("Long-grain rice", 700, Some(G), Grocery),
        ing("Spaghetti", 400, Some(G), Grocery),
        ing("Coconut milk", 400, Some(ML), Grocery),
        ing("Chopped tomatoes", 800, Some(G), Grocery),
        ing("Olive oil", 200, Some(ML), Grocery),
    ];
    let bakery = vec![
        ing("Sourdough", 4, None, Bakery),
        ing("Bread", 100, Some(G), Bakery),
    ];

    // Pre-check a handful of items so the progress bars look lived-in.
    let mut checked: HashSet<String> = HashSet::new();
    for i in produce.iter().take(3) {
        checked.insert(i.key());
    }
    for i in butcher.iter().take(1) {
        checked.insert(i.key());
    }

    let aisles = vec![
        aisle("shopping_FruitsAndVegetables", produce, &checked),
        aisle("shopping_Butcher", butcher, &checked),
        aisle("shopping_DairyAndEggs", dairy, &checked),
        aisle("shopping_Grocery", pantry, &checked),
        aisle("shopping_Bakery", bakery, &checked),
    ];

    let total_items: usize = aisles.iter().map(|a| a.total).sum();
    let checked_items: usize = aisles.iter().map(|a| a.checked).sum();
    let progress_pct = (checked_items * 100).checked_div(total_items).unwrap_or(0);

    let today = mealplan::now(TZ);
    let from = today + Duration::days(1);
    let to = from + Duration::days(6);

    GroceriesTemplate {
        user: demo_user(),
        checked,
        aisles,
        from_date: from.unix_timestamp() as u64,
        to_date: to.unix_timestamp() as u64,
        total_items,
        checked_items,
        progress_pct,
        ..Default::default()
    }
}

// ── Recipes page ─────────────────────────────────────────────────────────

fn to_read_result(nodes: Vec<UserViewList>) -> ReadResult<UserViewList> {
    ReadResult {
        edges: nodes
            .into_iter()
            .enumerate()
            .map(|(i, node)| Edge {
                cursor: Value(format!("demo-{i}")),
                node,
            })
            .collect(),
        page_info: PageInfo::default(),
    }
}

fn to_list(uv: &UserView) -> UserViewList {
    UserViewList {
        id: uv.id.clone(),
        owner_id: uv.owner_id.clone(),
        owner_name: uv.owner_name.clone(),
        recipe_type: sqlx::types::Text(uv.recipe_type.0.clone()),
        name: uv.name.clone(),
        description: uv.description.clone(),
        prep_time: uv.prep_time,
        cook_time: uv.cook_time,
        accepts_accompaniment: uv.accepts_accompaniment,
        is_shared: uv.is_shared,
        difficulty_score: uv.difficulty_score,
        ..Default::default()
    }
}

pub fn recipes(query: PageQuery) -> RecipesIndexTemplate {
    let search = query.search.clone().unwrap_or_default().to_lowercase();
    // Parse like the real handler so unknown values (e.g. the "All" radio,
    // whose empty `value=""` is dropped by HTML minification and submitted as
    // the browser default `on`) mean "no type filter" rather than matching
    // nothing.
    let recipe_type = query
        .recipe_type
        .as_deref()
        .and_then(|v| RecipeType::from_str(v).ok());

    // Type chips, search and sort behave like the real page. The Mine / Saved /
    // No-image filters never reach here — in demo they open the sign-up modal
    // instead of submitting the form.
    let mut matches: Vec<UserView> = catalog()
        .into_iter()
        .filter(|r| recipe_type.as_ref().is_none_or(|rt| &r.recipe_type.0 == rt))
        .filter(|r| {
            search.is_empty()
                || r.name.to_lowercase().contains(&search)
                || r.description.to_lowercase().contains(&search)
        })
        .collect();

    match &query.sort_by {
        Some(SortBy::Easiest) => matches.sort_by_key(|r| r.difficulty_score),
        Some(SortBy::Hardest) => matches.sort_by_key(|r| std::cmp::Reverse(r.difficulty_score)),
        _ => {}
    }

    let edges: Vec<Edge<UserViewList>> = matches
        .iter()
        .enumerate()
        .map(|(i, r)| Edge {
            cursor: Value(format!("demo-{i}")),
            node: to_list(r),
        })
        .collect();

    let recipes = ReadResult {
        edges,
        page_info: PageInfo::default(),
    };

    RecipesIndexTemplate {
        user: demo_user(),
        recipes,
        query,
        has_shared: true,
        ..Default::default()
    }
}

// ── Recipe detail page ───────────────────────────────────────────────────

pub fn recipe_detail(id: &str) -> DetailTemplate<'static> {
    let mut recipe =
        find_recipe(id).unwrap_or_else(|| find_recipe("arroz-con-pollo").expect("seed recipe"));

    // Present as a shared community recipe (non-owner view) so the page shows
    // the public layout — Save button + author card — instead of the owner's
    // edit/delete tools.
    recipe.owner_id = "imkitchen-team".to_owned();
    recipe.owner_name = Some("imkitchen".to_owned());
    recipe.is_shared = true;

    let rt = recipe.recipe_type.0.clone();
    let current_id = recipe.id.clone();

    let cook_nodes: Vec<UserViewList> = catalog()
        .iter()
        .filter(|r| r.id != current_id)
        .take(2)
        .map(to_list)
        .collect();

    let similar_nodes: Vec<UserViewList> = catalog()
        .iter()
        .filter(|r| r.id != current_id && r.recipe_type.0 == rt)
        .take(4)
        .map(to_list)
        .collect();

    DetailTemplate {
        user: demo_user(),
        recipe,
        username: "demo_chef",
        stat: UserStatView {
            shared: catalog().len() as u32,
            ..Default::default()
        },
        favorite: Favorite::default(),
        cook_recipes: to_read_result(cook_nodes),
        similar_recipes: to_read_result(similar_nodes),
        owner_description: "Home cook sharing tried-and-tested family recipes.".to_owned(),
        ..Default::default()
    }
}
