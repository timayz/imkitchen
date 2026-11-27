site_title = imkitchen

nav_screens = Screens
nav_home = Home
nav_sign_out = Sign out

## The public pages, which are www's

public_pages = Public pages
public_about = About imkitchen
public_terms = Terms of use
public_privacy = Privacy

## Signing in and registering

sign_in_meta_title = Sign in — imkitchen
sign_in_title = Sign in
sign_in_submit = Sign in
sign_in_to_register = No account yet? Create one

register_meta_title = Create an account — imkitchen
register_title = Create an account
register_submit = Create my account
register_to_sign_in = Already have an account? Sign in

auth_email = Email
auth_password = Password
auth_code = Code
auth_code_sent = A code was mailed to this address. Type it below, with your password again.
auth_failed = This is not possible right now. Try again in a moment.

## What a user does, handed to the api

command_failed = This is not possible right now. Try again in a moment.

## The home

home_title = Your kitchen
home_text = Your recipes, the dishes you compose with them, your list for the week and its groceries.
home_recipes =
    { $count ->
        [0] The first recipes are being written.
        [one] { $count } recipe is kept in imkitchen.
       *[other] { $count } recipes are kept in imkitchen.
    }

## The screens

recipes_meta_title = Recipes — imkitchen
recipes_title = Recipes
recipes_new = New recipe
recipes_import = Import
recipes_search_label = Title or ingredient
recipes_search = Search
recipes_none = You have no recipe yet. Write your first one, or import it from a site you like.
recipes_none_found = No recipe is found.
recipes_more = More recipes

recipe_minutes = { $minutes } min
recipe_serves =
    { $count ->
        [one] For { $count }
       *[other] For { $count }
    }
recipe_prep = Preparation
recipe_cook = Cooking
recipe_servings = Servings
recipe_ingredients = Ingredients
recipe_instructions = Steps
recipe_edit = Edit
recipe_delete = Delete…
recipe_delete_warning = The recipe is gone for good, with its picture. It leaves the dishes and the list it is part of.
recipe_delete_confirm = Delete the recipe
recipe_deleted = The recipe is deleted.
recipe_not_deleted = The recipe was not deleted
recipe_back = Back

recipe_new_meta_title = New recipe — imkitchen
recipe_new_title = New recipe
recipe_edit_meta_title = Edit the recipe — imkitchen
recipe_edit_title = Edit the recipe
recipe_title = Title
recipe_description = Description
recipe_prep_minutes = Preparation (min)
recipe_cook_minutes = Cooking (min)
recipe_ingredients_hint = One ingredient per row: how much of it, in what unit, and what it is. A row left blank is left out.
recipe_ingredient_quantity = Qty
recipe_ingredient_unit = Unit
recipe_ingredient_name = Ingredient
recipe_instructions_hint = One step per row, in the order they are made, each with the minutes to wait once it is done when there is something to wait for.
recipe_instruction_text = Step { $number }
recipe_instruction_timer = Timer
recipe_more_rows = More rows
recipe_picture = Picture
recipe_picture_choose = Choose a picture
recipe_picture_hint = A JPEG, PNG, WebP or GIF of 8 MB at most. Choose it last: asking for more rows forgets it.
recipe_picture_remove = Remove this picture
recipe_save = Save the recipe
recipe_cancel = Cancel
recipe_saved_without_picture = The recipe is saved, without the picture.

import_meta_title = Import a recipe — imkitchen
import_title = Import a recipe
import_text = Paste the address of a page that holds a recipe. It becomes one of yours, to edit like one you wrote.
import_url = Address of the page
import_submit = Import the recipe

dishes_meta_title = Dishes — imkitchen
dishes_title = Dishes
dishes_new = New dish
dishes_search_label = Name of a dish
dishes_none = You have no dish yet. Compose your first one from your recipes.
dishes_none_found = No dish is found.
dishes_more = More dishes

dish_recipes = Recipes
dish_recipes_count =
    { $count ->
        [0] No recipe
        [one] { $count } recipe
       *[other] { $count } recipes
    }
dish_no_recipe = This dish has no recipe left. Edit it to compose it again.
dish_delete_warning = The dish is gone for good. Its recipes stay.
dish_delete_confirm = Delete the dish
dish_deleted = The dish is deleted.
dish_not_deleted = The dish was not deleted

dish_new_meta_title = New dish — imkitchen
dish_new_title = New dish
dish_edit_meta_title = Edit the dish — imkitchen
dish_edit_title = Edit the dish
dish_name = Name
dish_chosen = Recipes of the dish
dish_chosen_hint = In the order they are served. Untick a recipe to take it out.
dish_chosen_none = No recipe yet: tick some below.
dish_move_up = Serve { $title } earlier
dish_move_down = Serve { $title } later
dish_others = Your other recipes
dish_others_hint = Tick the recipes to add: they join the dish when you save, or search again.
dish_others_none = No other recipe is found.
dish_others_more = Only the most recent are shown here: search to find another.
dish_no_recipe_yet = You have no recipe yet, and a dish is composed of your recipes.
dish_save = Save the dish

list_meta_title = List — imkitchen
list_title = List
list_generate = Generate my list
list_generate_hint = Generating draws your meals from your dishes and your recipes, as your settings say, in place of what the list holds.
list_add = Add a meal
list_none = Your list is empty. Generate it, or add the meals you choose.
list_meals =
    { $count ->
        [one] { $count } meal
       *[other] { $count } meals
    }
list_generated = Your list is generated.
list_added = The meal is on your list.
list_replaced = The meal is replaced.
list_removed = The meal is off your list.
list_cook_of = Cook { $title }
list_replace = Replace
list_replace_of = Replace { $title }
list_remove = Remove
list_remove_of = Remove { $title }

pick_add_meta_title = Add a meal — imkitchen
pick_add_title = Add a meal
pick_replace_meta_title = Replace a meal — imkitchen
pick_replace_title = Replace a meal
pick_replaces = In place of { $title }.
pick_search_label = Name, title or ingredient
pick_choose = Choose
pick_choose_of = Choose { $title }
pick_none_found = Nothing is found.
pick_nothing = You have no recipe yet, and a meal is one of your recipes or of your dishes.
pick_more = Only the most recent are shown here: search to find another.

groceries_meta_title = Groceries — imkitchen
groceries_title = Groceries
groceries_text = The ingredients of your list, added up into what there is to buy.
groceries_none = Your list is empty: there is nothing to buy.
groceries_fill = Fill my list
groceries_in_basket = in the basket
groceries_called_for = The list calls for { $amount }
groceries_changed = Your list changed: here are its groceries as they now stand.

settings_meta_title = Settings — imkitchen
settings_title = Settings
settings_saved = Saved.
settings_save = Save

settings_list = Your list
settings_list_hint = How your list is generated. The list as it stands is left as it is: the next one generated follows.
settings_servings = People to cook for
settings_meals = Meals in a list
settings_sources = Drawn from
settings_sources_plats_and_recipes = Your dishes and your recipes
settings_sources_plats = Your dishes only
settings_sources_recipes = Your recipes only

settings_language = Language
settings_language_hint = The language imkitchen is read in, on every device you sign in on.
settings_language_device = The language of the device

settings_account = Your account
settings_account_email = You sign in with { $email }.
settings_name = Name
settings_name_save = Save the name
settings_password = Password
settings_password_hint = A new password takes the current one. Every other device is signed out.
settings_current_password = Current password
settings_new_password = New password
settings_password_save = Change the password
settings_password_changed = The password is changed.
settings_sessions = Other devices
settings_sessions_hint = Signs your account out everywhere but here.
settings_sessions_end = Sign out everywhere else
settings_sessions_ended = Every other device is signed out.

## Cooking

recipe_cook_it = Cook
cook_meta_title = Cooking — imkitchen
cook_stop = Stop cooking
cook_asleep = This browser does not keep the screen awake: it may go dark while you cook.
cook_no_step = This recipe has no step.
cook_step = Step { $number }
cook_step_of = Step { $number } of { $total }
cook_timer = Then wait { $minutes } min
cook_timer_start = Start the timer
cook_timer_stop = Stop
cook_timer_over = The timer is over
cook_timer_over_of = The timer of step { $number } is over.
cook_previous = Previous
cook_next = Next
cook_finish = Finish
cook_choose_meta_title = Cook a dish — imkitchen
cook_choose = Which recipe do you cook?

## Not found

not_found_title = This page does not exist
not_found_text = The address may have changed, or never was one.
not_found_home = Back to the home
