//! The pictures of the recipes, through the api's routes: uploaded by the
//! owner of a recipe or imported with it, stored once however many recipes
//! show them, kept as long as one does, and served by every node.

mod helpers;

use std::io::Cursor;
use std::sync::Arc;

use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::{Method, StatusCode};
use helpers::api::{Api, Outbox, refusal, register, start};
use helpers::{Authority, TestCluster, TestNode, eventually};
use imkitchen_aggregates::image::Image;
use imkitchen_api::images::{Peers, attach, release};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const PAGE: &str = "https://recipes.example.com/toast";
const PICTURE: &str = "https://cdn.example.com/img/toast.png";

/// A PNG of `width` × `height` in one colour, as a camera or a site gives
/// a picture.
fn png(width: u32, height: u32, rgb: [u8; 3]) -> Vec<u8> {
    let image = image::RgbImage::from_pixel(width, height, image::Rgb(rgb));
    let mut out = Cursor::new(Vec::new());
    image.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

/// A page that describes a recipe and says where its picture is.
fn page(image: &str) -> String {
    let recipe = json!({
        "@context": "https://schema.org",
        "@type": "Recipe",
        "name": "Toast",
        "image": image,
        "recipeIngredient": ["2 slices of bread"],
        "recipeInstructions": "Toast the bread.",
    });
    format!(r#"<html><head><script type="application/ld+json">{recipe}</script></head></html>"#)
}

async fn write(api: &Api, token: &str, title: &str) -> String {
    let recipe = json!({ "title": title, "servings": 2 });
    let (status, body) = api.post("/v1/recipes", Some(token), recipe).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body["id"].as_str().unwrap().to_owned()
}

async fn import(api: &Api, token: &str, url: &str) -> Value {
    let (status, body) = api
        .post("/v1/recipes/import", Some(token), json!({ "url": url }))
        .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body
}

async fn upload(api: &Api, token: &str, id: &str, bytes: Vec<u8>) {
    let path = format!("/v1/recipes/{id}/image");
    let (status, body) = api.put_bytes(&path, Some(token), bytes).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
}

async fn delete(api: &Api, token: &str, path: &str) {
    let (status, body) = api.call(Method::DELETE, path, Some(token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
}

/// What the recipe `id` answers under `image`.
async fn image_of(api: &Api, token: &str, id: &str) -> Value {
    let (status, recipe) = api.get(&format!("/v1/recipes/{id}"), Some(token)).await;
    assert_eq!(status, StatusCode::OK, "{recipe}");
    recipe["image"].clone()
}

/// The picture at `url` as the api serves it, checked to be what its recipe
/// says of it: a WebP of `size`, which a cache may keep for good.
async fn served(api: &Api, file: &Value) {
    let url = file["url"].as_str().unwrap();
    let (status, headers, bytes) = api.get_bytes(url).await;
    assert_eq!(status, StatusCode::OK, "{url}");
    assert_eq!(headers[CONTENT_TYPE], "image/webp");
    assert_eq!(
        headers[CACHE_CONTROL],
        "public, max-age=31536000, immutable"
    );
    let picture = image::load_from_memory(&bytes).unwrap();
    assert_eq!(
        json!({ "url": url, "width": picture.width(), "height": picture.height() }),
        *file
    );
}

/// Waits until nothing answers at `url` any more, and checks no cache keeps
/// that.
async fn gone(api: &Api, url: &str) {
    eventually(&format!("{url} to be gone"), || async {
        let (status, headers, _) = api.get_bytes(url).await;
        (status == StatusCode::NOT_FOUND).then(|| assert_eq!(headers[CACHE_CONTROL], "no-store"))
    })
    .await;
}

/// How many events the stream of the picture `bytes` are holds on `node`,
/// once it holds `count`.
async fn image_events(node: &TestNode, bytes: &[u8], count: usize) {
    let digest: String = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    eventually(&format!("the picture to have {count} events"), || async {
        let events = evento::read::<Image>(&digest).execute(&node.store).await;
        (events.unwrap().len() == count).then_some(())
    })
    .await;
}

#[tokio::test]
async fn the_owner_of_a_recipe_gives_it_a_picture_and_takes_it_off() {
    let (api, outbox, node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;
    let id = write(&api, &token, "Toast").await;
    assert_eq!(image_of(&api, &token, &id).await, Value::Null);

    upload(&api, &token, &id, png(3000, 2000, [200, 120, 40])).await;

    // Kept in two sizes, neither as heavy as what was sent.
    let image = image_of(&api, &token, &id).await;
    let name = image["small"]["url"]
        .as_str()
        .unwrap()
        .strip_prefix("/v1/images/400/")
        .unwrap()
        .to_owned();
    assert_eq!(
        image,
        json!({
            "small": { "url": format!("/v1/images/400/{name}"), "width": 400, "height": 267 },
            "large": { "url": format!("/v1/images/1200/{name}"), "width": 1200, "height": 800 },
        })
    );
    served(&api, &image["small"]).await;
    served(&api, &image["large"]).await;
    assert_eq!(
        node.image_files(),
        [format!("1200/{name}"), format!("400/{name}")]
    );
    // The list shows it too, a moment later.
    eventually("the list to show the picture", || async {
        let (_, list) = api.get("/v1/recipes", Some(&token)).await;
        (list["recipes"][0]["image"] == image).then_some(())
    })
    .await;

    // Rewriting the recipe leaves its picture where it is.
    let rewritten = json!({ "title": "Toasts", "servings": 4 });
    let path = format!("/v1/recipes/{id}");
    let (status, _) = api
        .call(Method::PUT, &path, Some(&token), Some(rewritten))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(image_of(&api, &token, &id).await, image);

    // Another picture is another address, and the first one goes.
    upload(&api, &token, &id, png(300, 600, [20, 20, 200])).await;
    let other = image_of(&api, &token, &id).await;
    assert_ne!(other["small"]["url"], image["small"]["url"]);
    assert_eq!(
        (&other["small"]["width"], &other["small"]["height"]),
        (&json!(200), &json!(400))
    );
    assert_eq!(
        (&other["large"]["width"], &other["large"]["height"]),
        (&json!(300), &json!(600))
    );
    served(&api, &other["large"]).await;
    gone(&api, image["small"]["url"].as_str().unwrap()).await;
    gone(&api, image["large"]["url"].as_str().unwrap()).await;

    // Taken off, as often as asked.
    delete(&api, &token, &format!("{path}/image")).await;
    delete(&api, &token, &format!("{path}/image")).await;
    assert_eq!(image_of(&api, &token, &id).await, Value::Null);
    gone(&api, other["large"]["url"].as_str().unwrap()).await;
    eventually("the list to show no picture", || async {
        let (_, list) = api.get("/v1/recipes", Some(&token)).await;
        list["recipes"][0].get("image").is_none().then_some(())
    })
    .await;
    eventually("the files to be deleted", || async {
        node.image_files().is_empty().then_some(())
    })
    .await;

    api.shutdown().await;
}

#[tokio::test]
async fn what_is_not_a_picture_of_ones_own_recipe_is_refused() {
    let (api, outbox, node, _dir) = start().await;
    let (alice, _) = register(&api, &outbox, "alice@example.com").await;
    let (bob, _) = register(&api, &outbox, "bob@example.com").await;
    let id = write(&api, &alice, "Toast").await;
    let path = format!("/v1/recipes/{id}/image");
    let unprocessable = |code| (StatusCode::UNPROCESSABLE_ENTITY, refusal(code));

    for not_a_picture in [
        Vec::new(),
        b"not a picture".to_vec(),
        br#"<svg xmlns="http://www.w3.org/2000/svg"/>"#.to_vec(),
    ] {
        assert_eq!(
            api.put_bytes(&path, Some(&alice), not_a_picture).await,
            unprocessable("unsupported_image")
        );
    }
    // A header that claims more pixels than a picture may have.
    let mut huge = png(1, 1, [0; 3]);
    huge[16..20].copy_from_slice(&6000u32.to_be_bytes());
    huge[20..24].copy_from_slice(&6000u32.to_be_bytes());
    assert_eq!(
        api.put_bytes(&path, Some(&alice), huge).await,
        unprocessable("image_too_many_pixels")
    );
    assert_eq!(
        api.put_bytes(&path, Some(&alice), vec![0; 8 * 1024 * 1024 + 1])
            .await,
        unprocessable("image_too_large")
    );

    // Someone else's recipe is one that does not exist, whatever is sent.
    let picture = png(40, 40, [1, 2, 3]);
    let not_found = (StatusCode::NOT_FOUND, refusal("recipe_not_found"));
    assert_eq!(
        api.put_bytes(&path, Some(&bob), picture.clone()).await,
        not_found
    );
    assert_eq!(
        api.put_bytes(&path, Some(&bob), b"x".to_vec()).await,
        not_found
    );
    assert_eq!(
        api.call(Method::DELETE, &path, Some(&bob), None).await,
        not_found
    );
    let (status, _) = api.put_bytes(&path, None, picture).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    assert_eq!(image_of(&api, &alice, &id).await, Value::Null);
    assert!(node.image_files().is_empty(), "nothing was kept");

    // An address that names no picture is not found, and not for long.
    for path in [
        "/v1/images/400/01JDQ0B6V5K1X5C3E8W4T9Z2YH.webp",
        "/v1/images/800/01JDQ0B6V5K1X5C3E8W4T9Z2YH.webp",
        "/v1/images/400/01JDQ0B6V5K1X5C3E8W4T9Z2YH.png",
        "/v1/images/400/..%2F..%2Fimkitchen.db",
    ] {
        let (status, headers, body) = api.get_bytes(path).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
        assert_eq!(headers[CACHE_CONTROL], "no-store", "{path}");
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, refusal("not_found"), "{path}");
    }

    api.shutdown().await;
}

#[tokio::test]
async fn two_users_who_import_the_same_recipe_share_one_picture() {
    let (api, outbox, node, _dir) = start().await;
    let (alice, _) = register(&api, &outbox, "alice@example.com").await;
    let (bob, _) = register(&api, &outbox, "bob@example.com").await;
    let picture = png(1600, 1200, [180, 140, 90]);
    node.web.put(PAGE, &page(PICTURE));
    node.web.put_image(PICTURE, &picture);

    let hers = import(&api, &alice, PAGE).await;
    let his = import(&api, &bob, PAGE).await;

    // Two recipes, one picture: the same files, stored once.
    assert_ne!(hers["id"], his["id"]);
    assert_eq!(hers["image"]["large"]["width"], 1200);
    assert_eq!(hers["image"], his["image"]);
    assert_eq!(hers["updated_at"], hers["created_at"]);
    assert_eq!(node.image_files().len(), 2, "one file a size");
    served(&api, &hers["image"]["small"]).await;
    image_events(&node, &picture, 2).await;

    // The same picture uploaded is that one again.
    let own = write(&api, &alice, "My toast").await;
    upload(&api, &alice, &own, picture.clone()).await;
    assert_eq!(image_of(&api, &alice, &own).await, hers["image"]);
    assert_eq!(node.image_files().len(), 2);
    image_events(&node, &picture, 3).await;

    // Deleting one of the recipes keeps the picture for the others...
    delete(
        &api,
        &alice,
        &format!("/v1/recipes/{}", hers["id"].as_str().unwrap()),
    )
    .await;
    image_events(&node, &picture, 4).await;
    delete(&api, &alice, &format!("/v1/recipes/{own}/image")).await;
    image_events(&node, &picture, 5).await;
    let id = his["id"].as_str().unwrap();
    assert_eq!(image_of(&api, &bob, id).await, his["image"]);
    served(&api, &his["image"]["small"]).await;
    served(&api, &his["image"]["large"]).await;
    assert_eq!(node.image_files().len(), 2);

    // ...and the last one takes it along.
    delete(&api, &bob, &format!("/v1/recipes/{id}")).await;
    gone(&api, his["image"]["large"]["url"].as_str().unwrap()).await;
    eventually("the files to be deleted", || async {
        node.image_files().is_empty().then_some(())
    })
    .await;

    // Imported again, it is stored again, under another name.
    let again = import(&api, &bob, PAGE).await;
    assert_ne!(again["image"]["small"]["url"], his["image"]["small"]["url"]);
    served(&api, &again["image"]["small"]).await;

    api.shutdown().await;
}

/// The files of a picture are on a node before the event that names them,
/// and a node may only get to that event once the picture was let go of.
#[tokio::test]
async fn a_node_that_was_behind_deletes_the_files_nobody_uses_any_more() {
    let (node, _dir) = TestNode::alone().await;
    let picture = png(600, 400, [1, 2, 3]);

    // Stored, released and stored again, with nothing following yet.
    let bytes = bytes::Bytes::from(picture.clone());
    let first = attach(&node.executor, &node.images, bytes.clone()).await;
    let first = first.unwrap();
    release(&node.executor, &first.image_id, &first.use_id)
        .await
        .unwrap();
    let again = attach(&node.executor, &node.images, bytes).await.unwrap();
    assert_ne!(again.file_id, first.file_id);
    assert_eq!(node.image_files().len(), 4);

    let api = Api::over(&node, "api-0", Arc::new(Outbox::default()), 3600).await;
    let kept = [
        format!("1200/{}.webp", again.file_id),
        format!("400/{}.webp", again.file_id),
    ];
    // The handler deletes the files it was let go of before it records the
    // ones the picture has now: both are waited for.
    eventually(
        "the files let go of to be deleted and the picture to be known",
        || async { (node.image_files() == kept && node.images_known().await == 1).then_some(()) },
    )
    .await;

    api.shutdown().await;
}

#[tokio::test]
async fn a_recipe_is_imported_without_the_picture_that_cannot_be_had() {
    let (api, outbox, node, _dir) = start().await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;

    // Nothing answers where the page says its picture is.
    node.web.put(PAGE, &page(PICTURE));
    let imported = import(&api, &token, PAGE).await;
    assert_eq!(imported["title"], "Toast");
    assert_eq!(imported.get("image"), None);

    // What answers is not a picture.
    node.web.put_image(PICTURE, b"<html>Not found</html>");
    assert_eq!(import(&api, &token, PAGE).await.get("image"), None);

    // The page writes the address from where it is.
    node.web.put(PAGE, &page("/img/toast.png"));
    node.web.put_image(
        "https://recipes.example.com/img/toast.png",
        &png(8, 8, [9, 9, 9]),
    );
    let imported = import(&api, &token, PAGE).await;
    assert_eq!(imported["image"]["small"]["width"], 8);
    served(&api, &imported["image"]["small"]).await;

    api.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_picture_given_to_one_node_is_served_by_every_node() {
    let cluster = TestCluster::start(3).await;
    let outbox = Arc::new(Outbox::default());
    let mut apis = Vec::new();
    for (id, node) in cluster.nodes.iter().enumerate() {
        apis.push(Api::over(node, &format!("api-{id}"), outbox.clone(), 3600).await);
    }
    let (alice, _) = register(&apis[0], &outbox, "alice@example.com").await;
    let (bob, _) = register(&apis[0], &outbox, "bob@example.com").await;
    let picture = png(1600, 1200, [30, 160, 90]);

    let id = write(&apis[0], &alice, "Toast").await;
    upload(&apis[0], &alice, &id, picture.clone()).await;
    let image = image_of(&apis[0], &alice, &id).await;
    let (small, large) = (&image["small"], &image["large"]);

    // On a majority of the nodes before the recipe showed it, and on every
    // node a moment later: whichever is asked serves it.
    let holding = |count: usize| {
        let nodes = cluster.nodes.iter();
        nodes
            .filter(|node| node.image_files().len() == count)
            .count()
    };
    assert!(holding(2) >= 2, "a majority holds both files");
    eventually("every node to hold the picture", || async {
        (holding(2) == 3).then_some(())
    })
    .await;
    for (api, node) in apis.iter().zip(&cluster.nodes) {
        assert_eq!(image_of(api, &alice, &id).await, image);
        served(api, small).await;
        served(api, large).await;
        eventually("the node to know the picture", || async {
            (node.images_known().await == 1).then_some(())
        })
        .await;
    }

    // A node that lost a file takes it from another when it is asked.
    let last = &cluster.nodes[2];
    for file in last.image_files() {
        std::fs::remove_file(last.images_dir.join(file)).unwrap();
    }
    served(&apis[2], large).await;
    assert_eq!(last.image_files().len(), 1, "only what was asked for");
    served(&apis[2], small).await;

    // The same picture given to another node is the one already stored.
    cluster.nodes[1].web.put(PAGE, &page(PICTURE));
    cluster.nodes[1].web.put_image(PICTURE, &picture);
    let imported = import(&apis[1], &bob, PAGE).await;
    assert_eq!(imported["image"], image);
    assert_eq!(holding(2), 3, "nothing more is stored");

    // One of the two recipes deleted, on any node: the picture stays for
    // the other, on every node.
    delete(&apis[2], &alice, &format!("/v1/recipes/{id}")).await;
    for node in &cluster.nodes {
        image_events(node, &picture, 3).await;
    }
    for api in &apis {
        served(api, small).await;
        served(api, large).await;
    }
    assert_eq!(holding(2), 3);

    // The last one deleted: gone from every node.
    let path = format!("/v1/recipes/{}", imported["id"].as_str().unwrap());
    delete(&apis[0], &bob, &path).await;
    for api in &apis {
        gone(api, large["url"].as_str().unwrap()).await;
    }
    eventually("every node to delete the files", || async {
        (holding(0) == 3).then_some(())
    })
    .await;

    for api in apis {
        api.shutdown().await;
    }
    cluster.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_nodes_copy_pictures_to_each_other_over_their_certificates() {
    let authority = Authority::new();
    let cluster = TestCluster::start_over_tls(3, &authority).await;
    let outbox = Arc::new(Outbox::default());
    let mut apis = Vec::new();
    for (id, node) in cluster.nodes.iter().enumerate() {
        apis.push(Api::over(node, &format!("api-{id}"), outbox.clone(), 3600).await);
    }
    let (token, _) = register(&apis[0], &outbox, "alice@example.com").await;

    let id = write(&apis[0], &token, "Toast").await;
    upload(&apis[0], &token, &id, png(600, 400, [90, 30, 160])).await;
    let image = image_of(&apis[0], &token, &id).await;
    eventually("every node to hold the picture", || async {
        let mut nodes = cluster.nodes.iter();
        nodes
            .all(|node| node.image_files().len() == 2)
            .then_some(())
    })
    .await;
    for api in &apis {
        served(api, &image["large"]).await;
    }

    // The port of the nodes is theirs alone: a certificate of another
    // authority, or one of theirs that names no node, is let in nowhere.
    let file = image["large"]["url"].as_str().unwrap();
    let path = file.strip_prefix("/v1").unwrap();
    let asked = |tls| {
        let urls = cluster.image_urls.clone();
        let client = Peers::new(urls, 4, Some(tls)).unwrap().client();
        let url = format!("{}{path}", cluster.image_urls[0]);
        async move { client.get(url).send().await.map(|answer| answer.status()) }
    };
    let member = asked(authority.node("127.0.0.1")).await;
    assert_eq!(member.unwrap(), StatusCode::OK);
    let stranger = asked(Authority::new().node_trusting(&authority, "127.0.0.1")).await;
    assert!(stranger.is_err(), "{stranger:?}");
    let removed = asked(authority.node("10.9.8.7")).await;
    assert!(removed.is_err(), "{removed:?}");

    for api in apis {
        api.shutdown().await;
    }
    cluster.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_picture_too_few_nodes_took_is_not_recorded() {
    let mut cluster = TestCluster::start(3).await;
    let outbox = Arc::new(Outbox::default());
    let api = Api::over(&cluster.nodes[0], "api-0", outbox.clone(), 3600).await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;
    let id = write(&api, &token, "Toast").await;
    let path = format!("/v1/recipes/{id}/image");
    let picture = png(600, 400, [90, 30, 160]);

    // The store takes writes, but no other node takes the picture: this
    // node alone is no majority.
    cluster.nodes[1].stop_taking_pictures().await;
    cluster.nodes[2].stop_taking_pictures().await;
    assert_eq!(
        api.put_bytes(&path, Some(&token), picture.clone()).await,
        (StatusCode::SERVICE_UNAVAILABLE, refusal("write_not_taken"))
    );
    assert_eq!(image_of(&api, &token, &id).await, Value::Null);
    assert!(cluster.nodes[0].image_files().is_empty(), "nothing is kept");
    image_events(&cluster.nodes[0], &picture, 0).await;

    api.shutdown().await;
    cluster.shutdown().await;
}
