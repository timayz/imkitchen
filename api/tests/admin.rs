//! What an administrator does with what the users keep, through the api's
//! routes: deleting anyone's recipe, and taking a picture off every recipe
//! that shows it. Nobody else does either.

mod helpers;

use std::io::Cursor;

use axum::http::{Method, StatusCode};
use helpers::api::{Api, refusal, register, sign_in_root, start};
use helpers::eventually;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

/// A PNG of `width` × `height` in one colour.
fn png(width: u32, height: u32, rgb: [u8; 3]) -> Vec<u8> {
    let image = image::RgbImage::from_pixel(width, height, image::Rgb(rgb));
    let mut out = Cursor::new(Vec::new());
    image.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

/// What the stream of the picture `bytes` are is named: its id.
fn image_id(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Writes a recipe of the user `token` signs in, shown with `picture` when
/// there is one; its id.
async fn write(api: &Api, token: &str, title: &str, picture: Option<&[u8]>) -> String {
    let recipe = json!({ "title": title, "servings": 2 });
    let (status, body) = api.post("/v1/recipes", Some(token), recipe).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let id = body["id"].as_str().unwrap().to_owned();
    if let Some(picture) = picture {
        let path = format!("/v1/recipes/{id}/image");
        let (status, body) = api.put_bytes(&path, Some(token), picture.to_vec()).await;
        assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    }
    id
}

async fn delete(api: &Api, token: &str, path: &str) -> (StatusCode, Value) {
    api.call(Method::DELETE, path, Some(token), None).await
}

/// What the recipe `id` answers its owner under `image`.
async fn image_of(api: &Api, token: &str, id: &str) -> Value {
    let (status, recipe) = api.get(&format!("/v1/recipes/{id}"), Some(token)).await;
    assert_eq!(status, StatusCode::OK, "{recipe}");
    recipe["image"].clone()
}

/// Waits until the list of the user `token` signs in shows `count` recipes
/// with a picture.
async fn listed_with_a_picture(api: &Api, token: &str, count: usize) {
    eventually(
        &format!("{count} recipes listed with a picture"),
        || async {
            let (_, list) = api.get("/v1/recipes", Some(token)).await;
            let recipes = list["recipes"].as_array()?;
            let shown = recipes
                .iter()
                .filter(|recipe| recipe.get("image").is_some());
            (shown.count() == count).then_some(())
        },
    )
    .await;
}

#[tokio::test]
async fn an_administrator_deletes_the_recipe_of_anyone() {
    let (api, outbox, _node, _dir) = start().await;
    let (alice, _) = register(&api, &outbox, "alice@example.com").await;
    let (bob, _) = register(&api, &outbox, "bob@example.com").await;
    let root = sign_in_root(&api).await;
    let soup = write(&api, &alice, "Soup", None).await;
    let toast = write(&api, &alice, "Toast", None).await;
    let path = format!("/v1/admin/recipes/{soup}");

    // Not someone else who is no administrator, nor its owner this way.
    for token in [&bob, &alice] {
        assert_eq!(
            delete(&api, token, &path).await,
            (StatusCode::FORBIDDEN, refusal("not_allowed"))
        );
    }
    let (status, _) = api.get(&format!("/v1/recipes/{soup}"), Some(&alice)).await;
    assert_eq!(status, StatusCode::OK);

    // Deleted, it is gone for its owner, and the other is left.
    assert_eq!(delete(&api, &root, &path).await.0, StatusCode::NO_CONTENT);
    let (status, body) = api.get(&format!("/v1/recipes/{soup}"), Some(&alice)).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, refusal("recipe_not_found"))
    );
    eventually("the list to hold the other recipe alone", || async {
        let (_, list) = api.get("/v1/recipes", Some(&alice)).await;
        let recipes = list["recipes"].as_array()?;
        (recipes.len() == 1 && recipes[0]["id"] == toast.as_str()).then_some(())
    })
    .await;

    // There is nothing left to delete, as for one that never was.
    for path in [path.as_str(), "/v1/admin/recipes/nothing"] {
        assert_eq!(
            delete(&api, &root, path).await,
            (StatusCode::NOT_FOUND, refusal("recipe_not_found")),
            "{path}"
        );
    }

    api.shutdown().await;
}

#[tokio::test]
async fn an_administrator_takes_a_picture_off_every_recipe_that_shows_it() {
    let (api, outbox, node, _dir) = start().await;
    let (alice, _) = register(&api, &outbox, "alice@example.com").await;
    let (bob, _) = register(&api, &outbox, "bob@example.com").await;
    let root = sign_in_root(&api).await;
    let shared = png(800, 600, [200, 120, 40]);
    let other = png(600, 800, [20, 20, 200]);
    let hers = write(&api, &alice, "Toast", Some(&shared)).await;
    let his = write(&api, &bob, "Toasts", Some(&shared)).await;
    let kept = write(&api, &bob, "Soup", Some(&other)).await;
    assert_eq!(node.image_files().len(), 4, "two pictures, a file a size");
    listed_with_a_picture(&api, &alice, 1).await;
    listed_with_a_picture(&api, &bob, 2).await;
    let path = format!("/v1/admin/images/{}", image_id(&shared));

    for token in [&alice, &bob] {
        assert_eq!(
            delete(&api, token, &path).await,
            (StatusCode::FORBIDDEN, refusal("not_allowed"))
        );
    }
    assert_ne!(image_of(&api, &alice, &hers).await, Value::Null);

    // Taken off both recipes, which stay; the other picture is left alone.
    assert_eq!(delete(&api, &root, &path).await.0, StatusCode::NO_CONTENT);
    assert_eq!(image_of(&api, &alice, &hers).await, Value::Null);
    assert_eq!(image_of(&api, &bob, &his).await, Value::Null);
    let left = image_of(&api, &bob, &kept).await;
    assert_ne!(left, Value::Null);
    listed_with_a_picture(&api, &alice, 0).await;
    listed_with_a_picture(&api, &bob, 1).await;

    // Its files go with its last use, and then there is no such picture.
    eventually("the files of the picture to be deleted", || async {
        (node.image_files().len() == 2).then_some(())
    })
    .await;
    let (status, _, _) = api.get_bytes(left["small"]["url"].as_str().unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    eventually("the picture to be known no more", || async {
        let answer = delete(&api, &root, &path).await;
        (answer == (StatusCode::NOT_FOUND, refusal("image_not_found"))).then_some(())
    })
    .await;
    assert_eq!(
        delete(&api, &root, "/v1/admin/images/nothing").await,
        (StatusCode::NOT_FOUND, refusal("image_not_found"))
    );

    api.shutdown().await;
}
