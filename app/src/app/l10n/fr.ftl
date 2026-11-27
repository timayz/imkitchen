site_title = imkitchen

nav_screens = Écrans
nav_home = Accueil
nav_sign_out = Se déconnecter

## The public pages, which are www's

public_pages = Pages publiques
public_about = À propos d’imkitchen
public_terms = Conditions d’utilisation
public_privacy = Confidentialité

## Signing in and registering

sign_in_meta_title = Connexion — imkitchen
sign_in_title = Connexion
sign_in_submit = Se connecter
sign_in_to_register = Pas encore de compte ? Créez-en un

register_meta_title = Créer un compte — imkitchen
register_title = Créer un compte
register_submit = Créer mon compte
register_to_sign_in = Déjà un compte ? Connectez-vous

auth_email = Adresse e-mail
auth_password = Mot de passe
auth_code = Code
auth_code_sent = Un code a été envoyé à cette adresse. Saisissez-le ci-dessous, avec votre mot de passe à nouveau.
auth_failed = Ce n’est pas possible pour le moment. Réessayez dans un instant.

## What a user does, handed to the api

command_failed = Ce n’est pas possible pour le moment. Réessayez dans un instant.

## The home

home_title = Votre cuisine
home_text = Vos recettes, les plats que vous composez avec, votre liste de la semaine et ses courses.
home_recipes =
    { $count ->
        [0] Les premières recettes sont en train d’être écrites.
        [one] { $count } recette est gardée dans imkitchen.
       *[other] { $count } recettes sont gardées dans imkitchen.
    }

## The screens

screen_coming = Cet écran arrive.

recipes_meta_title = Recettes — imkitchen
recipes_title = Recettes
recipes_new = Nouvelle recette
recipes_import = Importer
recipes_search_label = Titre ou ingrédient
recipes_search = Chercher
recipes_none = Vous n’avez pas encore de recette. Écrivez la première, ou importez-la d’un site que vous aimez.
recipes_none_found = Aucune recette n’est trouvée.
recipes_more = Plus de recettes

recipe_minutes = { $minutes } min
recipe_serves =
    { $count ->
        [one] Pour { $count }
       *[other] Pour { $count }
    }
recipe_prep = Préparation
recipe_cook = Cuisson
recipe_servings = Portions
recipe_ingredients = Ingrédients
recipe_instructions = Étapes
recipe_edit = Modifier
recipe_delete = Supprimer…
recipe_delete_warning = La recette disparaît pour de bon, avec sa photo. Elle quitte les plats et la liste dont elle fait partie.
recipe_delete_confirm = Supprimer la recette
recipe_deleted = La recette est supprimée.
recipe_not_deleted = La recette n’a pas été supprimée
recipe_back = Retour

recipe_new_meta_title = Nouvelle recette — imkitchen
recipe_new_title = Nouvelle recette
recipe_edit_meta_title = Modifier la recette — imkitchen
recipe_edit_title = Modifier la recette
recipe_title = Titre
recipe_description = Description
recipe_prep_minutes = Préparation (min)
recipe_cook_minutes = Cuisson (min)
recipe_ingredients_hint = Un ingrédient par ligne : combien, dans quelle unité, et ce que c’est. Une ligne laissée vide n’est pas gardée.
recipe_ingredient_quantity = Qté
recipe_ingredient_unit = Unité
recipe_ingredient_name = Ingrédient
recipe_instructions_hint = Une étape par ligne, dans l’ordre où elles se font, chacune avec les minutes à attendre une fois faite quand il y a quelque chose à attendre.
recipe_instruction_text = Étape { $number }
recipe_instruction_timer = Minuteur
recipe_more_rows = Plus de lignes
recipe_picture = Photo
recipe_picture_choose = Choisir une photo
recipe_picture_hint = Un JPEG, PNG, WebP ou GIF de 8 Mo au plus. Choisissez-la en dernier : demander plus de lignes l’oublie.
recipe_picture_remove = Retirer cette photo
recipe_save = Enregistrer la recette
recipe_cancel = Annuler
recipe_saved_without_picture = La recette est enregistrée, sans la photo.

import_meta_title = Importer une recette — imkitchen
import_title = Importer une recette
import_text = Collez l’adresse d’une page qui contient une recette. Elle devient l’une des vôtres, à modifier comme une recette que vous avez écrite.
import_url = Adresse de la page
import_submit = Importer la recette

dishes_meta_title = Plats — imkitchen
dishes_title = Plats
dishes_new = Nouveau plat
dishes_search_label = Nom d’un plat
dishes_none = Vous n’avez pas encore de plat. Composez le premier à partir de vos recettes.
dishes_none_found = Aucun plat n’est trouvé.
dishes_more = Plus de plats

dish_recipes = Recettes
dish_recipes_count =
    { $count ->
        [0] Aucune recette
        [one] { $count } recette
       *[other] { $count } recettes
    }
dish_no_recipe = Ce plat n’a plus de recette. Modifiez-le pour le recomposer.
dish_delete_warning = Le plat disparaît pour de bon. Ses recettes restent.
dish_delete_confirm = Supprimer le plat
dish_deleted = Le plat est supprimé.
dish_not_deleted = Le plat n’a pas été supprimé

dish_new_meta_title = Nouveau plat — imkitchen
dish_new_title = Nouveau plat
dish_edit_meta_title = Modifier le plat — imkitchen
dish_edit_title = Modifier le plat
dish_name = Nom
dish_chosen = Recettes du plat
dish_chosen_hint = Dans l’ordre où elles sont servies. Décochez une recette pour la retirer.
dish_chosen_none = Pas encore de recette : cochez-en ci-dessous.
dish_move_up = Servir { $title } plus tôt
dish_move_down = Servir { $title } plus tard
dish_others = Vos autres recettes
dish_others_hint = Cochez les recettes à ajouter : elles rejoignent le plat quand vous enregistrez, ou cherchez à nouveau.
dish_others_none = Aucune autre recette n’est trouvée.
dish_others_more = Seules les plus récentes sont affichées ici : cherchez pour en trouver une autre.
dish_no_recipe_yet = Vous n’avez pas encore de recette, et un plat se compose de vos recettes.
dish_save = Enregistrer le plat

list_meta_title = Liste — imkitchen
list_title = Liste
list_text = Les plats et les recettes de votre semaine, générés ou choisis un par un.

groceries_meta_title = Courses — imkitchen
groceries_title = Courses
groceries_text = Les ingrédients de votre liste, additionnés en ce qu’il y a à acheter.

settings_meta_title = Réglages — imkitchen
settings_title = Réglages
settings_saved = Enregistré.
settings_save = Enregistrer

settings_list = Votre liste
settings_list_hint = Comment votre liste est générée. La liste actuelle reste telle quelle : la prochaine générée suivra.
settings_servings = Personnes à table
settings_meals = Repas dans une liste
settings_sources = Tirés de
settings_sources_plats_and_recipes = Vos plats et vos recettes
settings_sources_plats = Vos plats seulement
settings_sources_recipes = Vos recettes seulement

settings_language = Langue
settings_language_hint = La langue dans laquelle imkitchen se lit, sur tous les appareils où vous vous connectez.
settings_language_device = La langue de l’appareil

settings_account = Votre compte
settings_account_email = Vous vous connectez avec { $email }.
settings_name = Nom
settings_name_save = Enregistrer le nom
settings_password = Mot de passe
settings_password_hint = Un nouveau mot de passe demande l’actuel. Tous les autres appareils sont déconnectés.
settings_current_password = Mot de passe actuel
settings_new_password = Nouveau mot de passe
settings_password_save = Changer le mot de passe
settings_password_changed = Le mot de passe est changé.
settings_sessions = Autres appareils
settings_sessions_hint = Déconnecte votre compte partout sauf ici.
settings_sessions_end = Se déconnecter partout ailleurs
settings_sessions_ended = Tous les autres appareils sont déconnectés.

## Not found

not_found_title = Cette page n’existe pas
not_found_text = Son adresse a peut-être changé, ou n’en a jamais été une.
not_found_home = Retour à l’accueil
