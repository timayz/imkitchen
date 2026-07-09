use std::io::{Cursor, Write};
use std::str::FromStr;

use evento::Sqlite;
use evento::migrator::{Migrate, Plan};
use image::{DynamicImage, ImageFormat, RgbImage};
use imkitchen_core::State;
use imkitchen_identity::RegisterInput;
use imkitchen_identity::types::user::Role;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use temp_dir::TempDir;
use zip::write::SimpleFileOptions;

async fn setup_test_state(path: std::path::PathBuf) -> anyhow::Result<State<Sqlite>> {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", path.to_str().unwrap()))?
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;
    let mut conn = pool.acquire().await?;
    imkitchen_db::migrator::<sqlx::Sqlite>()?
        .run(&mut conn, &Plan::apply_all())
        .await?;

    Ok(State {
        executor: pool.clone().into(),
        read_db: pool.clone(),
        write_db: pool,
    })
}

fn recipe_json(name: &str) -> String {
    format!(
        r#"{{
            "recipe_type": "MainCourse",
            "name": "{name}",
            "description": "A delicious test recipe",
            "household_size": 4,
            "prep_time": 20,
            "cook_time": 40,
            "ingredients": [{{ "quantity": 500, "unit": "G", "name": "Beef", "category": null }}],
            "instructions": [{{ "description": "Cook everything thoroughly", "time_next": 0 }}],
            "accepts_accompaniment": false,
            "dietary_restrictions": []
        }}"#
    )
}

fn png_bytes() -> Vec<u8> {
    let img = RgbImage::new(2, 2);
    let mut out = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut out, ImageFormat::Png)
        .unwrap();
    out.into_inner()
}

fn build_zip() -> anyhow::Result<Vec<u8>> {
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

        // New chef, recipe with an image.
        zip.start_file("gordon_ramsay/beef-wellington.json", opts)?;
        zip.write_all(recipe_json("Beef Wellington").as_bytes())?;
        zip.start_file("gordon_ramsay/beef-wellington.png", opts)?;
        zip.write_all(&png_bytes())?;

        // Same chef, recipe without an image.
        zip.start_file("gordon_ramsay/scrambled-eggs.json", opts)?;
        zip.write_all(recipe_json("Scrambled Eggs").as_bytes())?;

        // Maps to an existing non-chef account -> should be skipped.
        zip.start_file("existing_user/soup.json", opts)?;
        zip.write_all(recipe_json("Tomato Soup").as_bytes())?;

        // Folder name cannot be sanitized to a valid username -> author error.
        zip.start_file("a!/orphan.json", opts)?;
        zip.write_all(recipe_json("Orphan").as_bytes())?;

        zip.finish()?;
    }
    Ok(buf)
}

#[tokio::test]
async fn imports_recipes_and_creates_chef_accounts() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let state = setup_test_state(dir.child("db.sqlite3")).await?;

    let identity = imkitchen_identity::Module::new(state.clone());
    let recipe = imkitchen_core::recipe::Module::new(state.clone());

    // Pre-create a plain (non-chef) account that author "existing_user" maps to.
    identity
        .register(RegisterInput {
            email: "recipes+existing_user@imkitchen.app".to_owned(),
            password: "my_password".to_owned(),
            lang: "en".to_owned(),
            timezone: "UTC".to_owned(),
        })
        .await?;

    let zip_bytes = build_zip()?;

    let jobs = imkitchen_web_shared::AdminImportJobs::default();
    let progress = imkitchen_web_admin::import::process_zip(
        &identity,
        &recipe,
        "admin-id",
        "root_password",
        zip_bytes,
        &jobs,
        "test-job",
    )
    .await;

    assert!(progress.done);
    assert_eq!(
        progress.authors_total, 3,
        "gordon_ramsay, existing_user, a!"
    );
    assert_eq!(progress.recipes_imported, 2, "both gordon_ramsay recipes");

    // Two author-scope errors: non-chef account + un-sanitizable folder name.
    assert_eq!(progress.errors.len(), 2);
    assert!(progress.errors.iter().all(|e| e.scope == "author"));
    assert!(
        progress
            .errors
            .iter()
            .any(|e| e.name == "existing_user" && e.message.contains("not a Chef"))
    );
    assert!(progress.errors.iter().any(|e| e.name == "a!"));

    // A Chef account was created for the new author.
    let chef = identity
        .find_account("recipes+gordon_ramsay@imkitchen.app")
        .await?
        .expect("chef account created");
    assert_eq!(chef.role, Role::Chef);

    // The pre-existing account was left untouched (still not a chef).
    let existing = identity
        .find_account("recipes+existing_user@imkitchen.app")
        .await?
        .expect("existing account");
    assert_ne!(existing.role, Role::Chef);

    Ok(())
}
