//! Admin batch recipe import from a ZIP archive.
//!
//! Expected archive layout (a wrapping top-level folder is tolerated):
//! ```text
//! {author}/{recipe_slug}.json                      # recipe payload (same schema as user import)
//! {author}/{recipe_slug}.{jpg|jpeg|png|webp}       # optional thumbnail for that recipe
//! ```
//!
//! Each `{author}` folder maps to a Chef account `recipes+{username}@imkitchen.app`, where
//! `username` is the folder name sanitized to the username rules. The account is created (as a
//! Chef) if it does not exist; an existing non-Chef account is reported as an error and skipped.

use std::collections::{BTreeMap, HashMap};
use std::io::{Cursor, Read};

use evento::Executor;
use imkitchen_identity::RegisterInput;
use imkitchen_identity::types::user::Role;
use imkitchen_types::recipe::{DietaryRestriction, Ingredient, Instruction, RecipeType};
use imkitchen_web_shared::template::SERVER_ERROR_MESSAGE;
use imkitchen_web_shared::{AdminImportError, AdminImportProgress};
use serde::Deserialize;

/// One recipe payload, matching the user-facing import schema (`schema.json` v1.0).
#[derive(Deserialize)]
struct RecipeJson {
    recipe_type: RecipeType,
    name: String,
    origin: Option<String>,
    description: String,
    household_size: u16,
    prep_time: u16,
    cook_time: u16,
    ingredients: Vec<Ingredient>,
    instructions: Vec<Instruction>,
    advance_prep: Option<String>,
    #[serde(default)]
    dietary_restrictions: Vec<DietaryRestriction>,
    accepts_accompaniment: bool,
}

#[derive(Default)]
struct ZipEntry {
    json: Option<Vec<u8>>,
    image: Option<Vec<u8>>,
}

/// Sanitize an author folder name into a valid username (3-25 chars, `[a-z0-9_]`).
///
/// Letters and digits are lowercased and kept; spaces, hyphens and underscores collapse to a
/// single underscore; everything else is dropped. Returns `None` when the result cannot satisfy
/// the minimum length.
pub fn sanitize_username(folder: &str) -> Option<String> {
    let mut out = String::new();
    let mut prev_underscore = false;

    for c in folder.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_underscore = false;
        } else if (c == '_' || c == '-' || c == ' ') && !out.is_empty() && !prev_underscore {
            out.push('_');
            prev_underscore = true;
        }
    }

    while out.ends_with('_') {
        out.pop();
    }

    if out.chars().count() > 25 {
        out = out.chars().take(25).collect();
        while out.ends_with('_') {
            out.pop();
        }
    }

    if out.chars().count() < 3 {
        return None;
    }

    Some(out)
}

/// Convert a command error into a user-facing message, logging server errors.
fn user_message(err: imkitchen_core::Error) -> String {
    match err {
        imkitchen_core::Error::Server(e) => {
            tracing::error!(err = %e, "admin batch import server error");
            SERVER_ERROR_MESSAGE.to_string()
        }
        other => other.to_string(),
    }
}

fn parse_recipe(bytes: &[u8]) -> Result<imkitchen_core::recipe::ImportInput, String> {
    let recipe: RecipeJson =
        serde_json::from_slice(bytes).map_err(|e| format!("Invalid JSON: {e}"))?;

    Ok(imkitchen_core::recipe::ImportInput {
        recipe_type: recipe.recipe_type,
        name: recipe.name,
        origin: recipe.origin,
        description: recipe.description,
        household_size: recipe.household_size,
        prep_time: recipe.prep_time,
        cook_time: recipe.cook_time,
        ingredients: recipe.ingredients,
        instructions: recipe.instructions,
        advance_prep: recipe.advance_prep.unwrap_or_default(),
        accepts_accompaniment: recipe.accepts_accompaniment,
        dietary_restrictions: recipe.dietary_restrictions,
    })
}

/// Resolve the Chef account for `username`, creating it if necessary.
///
/// Returns the chef id on success, or a user-facing error message describing why this author
/// could not be processed.
async fn resolve_or_create_chef<E: Executor + Clone>(
    identity: &imkitchen_identity::Module<E>,
    admin_id: &str,
    username: &str,
    password: &str,
    cache: &mut HashMap<String, String>,
) -> Result<String, String> {
    if let Some(id) = cache.get(username) {
        return Ok(id.clone());
    }

    let email = format!("recipes+{username}@imkitchen.app");

    let id = match identity.find_account(&email).await.map_err(user_message)? {
        Some(account) => {
            if account.role != Role::Chef {
                return Err("An account already exists for this author but is not a Chef.".into());
            }
            account.id
        }
        None => {
            let id = identity
                .register(RegisterInput {
                    email,
                    password: password.to_string(),
                    lang: "en".into(),
                    timezone: "UTC".into(),
                })
                .await
                .map_err(user_message)?;

            identity
                .change_role(&id, Role::Chef, admin_id)
                .await
                .map_err(user_message)?;

            identity
                .set_username(&id, username.to_string())
                .await
                .map_err(user_message)?;

            id
        }
    };

    cache.insert(username.to_string(), id.clone());
    Ok(id)
}

/// Process a recipe ZIP archive end to end and return the resulting progress (with `done = true`).
///
/// `password` is used as the password for any Chef account created during the import.
pub async fn process_zip<E: Executor + Clone>(
    identity: &imkitchen_identity::Module<E>,
    recipe: &imkitchen_core::recipe::Module<E>,
    admin_id: &str,
    password: &str,
    zip_bytes: Vec<u8>,
) -> AdminImportProgress {
    let mut progress = AdminImportProgress::default();

    let mut archive = match zip::ZipArchive::new(Cursor::new(zip_bytes)) {
        Ok(archive) => archive,
        Err(e) => {
            progress.done = true;
            progress.errors.push(AdminImportError {
                scope: "author".into(),
                name: "archive".into(),
                message: format!("Invalid ZIP archive: {e}"),
            });
            return progress;
        }
    };

    // Group entries by author folder, then by recipe slug.
    let mut authors: BTreeMap<String, BTreeMap<String, ZipEntry>> = BTreeMap::new();

    for i in 0..archive.len() {
        let Ok(mut file) = archive.by_index(i) else {
            continue;
        };
        if file.is_dir() {
            continue;
        }

        let name = file.name().replace('\\', "/");
        let parts: Vec<&str> = name.split('/').filter(|s| !s.is_empty()).collect();
        if parts.len() < 2 {
            continue;
        }
        let author = parts[parts.len() - 2].to_string();
        let filename = parts[parts.len() - 1];
        let Some((slug, ext)) = filename.rsplit_once('.') else {
            continue;
        };
        let ext = ext.to_ascii_lowercase();
        let is_json = ext == "json";
        let is_image = matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp");
        if !is_json && !is_image {
            continue;
        }

        let mut buf = Vec::new();
        if file.read_to_end(&mut buf).is_err() {
            continue;
        }

        let entry = authors
            .entry(author)
            .or_default()
            .entry(slug.to_string())
            .or_default();
        if is_json {
            entry.json = Some(buf);
        } else {
            entry.image = Some(buf);
        }
    }

    progress.authors_total = authors.len();
    let mut cache: HashMap<String, String> = HashMap::new();

    for (author, recipes) in authors {
        let Some(username) = sanitize_username(&author) else {
            progress.errors.push(AdminImportError {
                scope: "author".into(),
                name: author,
                message:
                    "Could not derive a valid username (needs 3-25 letters, digits or underscores)."
                        .into(),
            });
            continue;
        };

        let chef_id =
            match resolve_or_create_chef(identity, admin_id, &username, password, &mut cache).await
            {
                Ok(id) => id,
                Err(message) => {
                    progress.errors.push(AdminImportError {
                        scope: "author".into(),
                        name: author,
                        message,
                    });
                    continue;
                }
            };

        for (slug, entry) in recipes {
            let label = format!("{author}/{slug}");

            let Some(json) = entry.json else {
                progress.errors.push(AdminImportError {
                    scope: "recipe".into(),
                    name: label,
                    message: "Image has no matching .json file.".into(),
                });
                continue;
            };

            let input = match parse_recipe(&json) {
                Ok(input) => input,
                Err(message) => {
                    progress.errors.push(AdminImportError {
                        scope: "recipe".into(),
                        name: label,
                        message,
                    });
                    continue;
                }
            };

            let recipe_name = input.name.clone();
            match recipe.import(input, &chef_id, Some(username.clone())).await {
                Ok(recipe_id) => {
                    progress.recipes_imported += 1;

                    if let Some(image) = entry.image
                        && let Err(e) = recipe.upload_thunmnail(&recipe_id, image, &chef_id).await
                    {
                        progress.errors.push(AdminImportError {
                            scope: "recipe".into(),
                            name: recipe_name.clone(),
                            message: format!(
                                "Recipe imported, but image failed: {}",
                                user_message(e)
                            ),
                        });
                    }

                    // Imported recipes are published to the community (idempotent: a recipe that
                    // is already shared is left unchanged).
                    if let Err(e) = recipe
                        .share_to_community(&recipe_id, &chef_id, username.clone())
                        .await
                    {
                        progress.errors.push(AdminImportError {
                            scope: "recipe".into(),
                            name: recipe_name,
                            message: format!(
                                "Recipe imported, but sharing failed: {}",
                                user_message(e)
                            ),
                        });
                    }
                }
                Err(e) => {
                    progress.errors.push(AdminImportError {
                        scope: "recipe".into(),
                        name: recipe_name,
                        message: user_message(e),
                    });
                }
            }
        }
    }

    progress.done = true;
    progress
}

#[cfg(test)]
mod tests {
    use super::sanitize_username;

    #[test]
    fn sanitizes_common_names() {
        assert_eq!(
            sanitize_username("Jean Dupont").as_deref(),
            Some("jean_dupont")
        );
        assert_eq!(
            sanitize_username("Chef-Marie!").as_deref(),
            Some("chef_marie")
        );
        assert_eq!(sanitize_username("chef_123").as_deref(), Some("chef_123"));
    }

    #[test]
    fn rejects_too_short() {
        assert_eq!(sanitize_username("a"), None);
        assert_eq!(sanitize_username("!!"), None);
        assert_eq!(sanitize_username("__"), None);
    }

    #[test]
    fn truncates_to_twenty_five() {
        let out = sanitize_username("abcdefghijklmnopqrstuvwxyzabcdefghijklmn").unwrap();
        assert_eq!(out, "abcdefghijklmnopqrstuvwxy");
        assert!(out.chars().count() <= 25);
    }

    #[test]
    fn collapses_separators_and_trims() {
        assert_eq!(
            sanitize_username("  John   Doe  ").as_deref(),
            Some("john_doe")
        );
        assert_eq!(sanitize_username("--mike--").as_deref(), Some("mike"));
    }
}
