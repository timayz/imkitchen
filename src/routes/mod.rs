pub mod assets;
pub mod auth;
pub mod collections;
pub mod dashboard;
pub mod health;
pub mod landing;
pub mod legal;
pub mod meal_plan;
pub mod notifications;
pub mod profile;
pub mod recipes;
pub mod shopping;

pub use assets::AssetsService;
pub use auth::{
    get_check_user, get_login, get_password_reset, get_password_reset_complete, get_register,
    post_login, post_logout, post_password_reset, post_password_reset_complete, post_register,
    post_stripe_webhook, AppState,
};
pub use collections::{
    get_collections, post_add_recipe_to_collection, post_create_collection, post_delete_collection,
    post_remove_recipe_from_collection, post_update_collection,
};
pub use dashboard::dashboard_handler;
pub use health::{browser_support, health, offline, ready};
pub use landing::get_landing;
pub use legal::{get_contact, get_help, get_privacy, get_terms, post_contact};
pub use meal_plan::{
    get_meal_alternatives, get_meal_plan, get_regenerate_confirm, post_generate_meal_plan,
    post_regenerate_meal_plan, post_replace_meal,
};
pub use notifications::{
    complete_prep_task_handler, dismiss_notification, get_notification_status, list_notifications,
    notifications_page, record_permission_change, snooze_notification, subscribe_push,
};
pub use profile::{
    get_onboarding, get_onboarding_skip, get_profile, get_subscription, get_subscription_success,
    post_onboarding_step_1, post_onboarding_step_2, post_onboarding_step_3, post_profile,
    post_subscription_upgrade,
};
pub use recipes::{
    check_recipe_exists, get_discover, get_discover_detail, get_import_modal, get_ingredient_row,
    get_instruction_row, get_recipe_detail, get_recipe_edit_form, get_recipe_form, get_recipe_list,
    get_recipe_waiting, post_add_to_library, post_create_recipe, post_delete_recipe,
    post_delete_review, post_favorite_recipe, post_import_recipes, post_rate_recipe,
    post_share_recipe, post_update_recipe, post_update_recipe_tags,
};
pub use shopping::{
    check_shopping_item, refresh_shopping_list, reset_shopping_list_handler, show_shopping_list,
};
