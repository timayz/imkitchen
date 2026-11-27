site_title = imkitchen

nav_screens = Écrans
nav_home = Accueil
nav_sign_out = Se déconnecter

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
recipes_text = Les recettes que vous cuisinez, écrites à la main ou importées des sites que vous aimez.

dishes_meta_title = Plats — imkitchen
dishes_title = Plats
dishes_text = Les plats que vous servez, composés de vos recettes.

list_meta_title = Liste — imkitchen
list_title = Liste
list_text = Les plats et les recettes de votre semaine, générés ou choisis un par un.

groceries_meta_title = Courses — imkitchen
groceries_title = Courses
groceries_text = Les ingrédients de votre liste, additionnés en ce qu’il y a à acheter.

settings_meta_title = Réglages — imkitchen
settings_title = Réglages
settings_text = Comment vous aimez votre liste, et votre compte.

## Not found

not_found_title = Cette page n’existe pas
not_found_text = Son adresse a peut-être changé, ou n’en a jamais été une.
not_found_home = Retour à l’accueil
