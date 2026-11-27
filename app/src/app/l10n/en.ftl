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

screen_coming = This screen is on its way.

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
dishes_text = The dishes you serve, composed from your recipes.

list_meta_title = List — imkitchen
list_title = List
list_text = The dishes and the recipes of your week, generated or chosen one by one.

groceries_meta_title = Groceries — imkitchen
groceries_title = Groceries
groceries_text = The ingredients of your list, added up into what there is to buy.

settings_meta_title = Settings — imkitchen
settings_title = Settings
settings_text = How you like your list, and your account.

## Not found

not_found_title = This page does not exist
not_found_text = The address may have changed, or never was one.
not_found_home = Back to the home
