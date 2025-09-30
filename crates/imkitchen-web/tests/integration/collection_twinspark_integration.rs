use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use imkitchen_web::{create_app_routes, AppState};
use tower::ServiceExt;

async fn create_test_app() -> Router {
    let app_state = AppState::test_default().await;
    create_app_routes(app_state)
}

#[tokio::test]
async fn test_collections_index_endpoint() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/collections")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should contain collections page content
    assert!(body_str.contains("My Collections"));
}

#[tokio::test]
async fn test_new_collection_form_endpoint() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/collections/new")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should contain form elements with TwinSpark attributes
    assert!(body_str.contains("Create New Collection"));
    assert!(body_str.contains("Create Collection"));
}

#[tokio::test] 
async fn test_favorites_list_endpoint() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/favorites")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should contain favorites page content
    assert!(body_str.contains("⭐ My Favorites"));
    assert!(body_str.contains("Quick access to your favorite recipes"));
}

#[tokio::test]
async fn test_collection_creation_post_endpoint() {
    let app = create_test_app().await;

    let form_data = "name=Test%20Collection&description=A%20test%20collection&privacy=Private";
    
    let request = Request::builder()
        .method("POST")
        .uri("/collections")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return successful response (200 OK for fragment update)
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_collection_search_endpoint() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/collections/search?q=pasta")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    // Should return search results fragment
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let _body_str = String::from_utf8(body.to_vec()).unwrap();
}

#[tokio::test]
async fn test_collection_filter_endpoint() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/collections/filter?privacy=Public")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_collection_sort_endpoint() {
    let app = create_test_app().await;

    let request = Request::builder()
        .uri("/collections/sort?sort=name_asc")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_recipe_selector_endpoint() {
    let app = create_test_app().await;
    
    // Use a valid UUID for the collection_id parameter
    let collection_id = "123e4567-e89b-12d3-a456-426614174000";

    let request = Request::builder()
        .uri(&format!("/recipes/selector?collection_id={}", collection_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should contain recipe selector modal
    assert!(body_str.contains("Add Recipes to Collection"));
}

#[tokio::test]
async fn test_collection_archive_endpoint() {
    let app = create_test_app().await;
    
    // Use a valid UUID for the collection_id parameter  
    let collection_id = "123e4567-e89b-12d3-a456-426614174000";

    let request = Request::builder()
        .method("POST")
        .uri(&format!("/collections/{}/archive", collection_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_collection_restore_endpoint() {
    let app = create_test_app().await;
    
    // Use a valid UUID for the collection_id parameter
    let collection_id = "123e4567-e89b-12d3-a456-426614174000";

    let request = Request::builder()
        .method("POST")
        .uri(&format!("/collections/{}/restore", collection_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_add_recipe_to_collection_endpoint() {
    let app = create_test_app().await;
    
    let collection_id = "123e4567-e89b-12d3-a456-426614174000";
    let form_data = "recipe_id=456e7890-f12a-34b5-c678-901234567890";

    let request = Request::builder()
        .method("POST")
        .uri(&format!("/collections/{}/recipes", collection_id))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_remove_recipe_from_collection_endpoint() {
    let app = create_test_app().await;
    
    let collection_id = "123e4567-e89b-12d3-a456-426614174000";
    let recipe_id = "456e7890-f12a-34b5-c678-901234567890";

    let request = Request::builder()
        .method("DELETE")
        .uri(&format!("/collections/{}/recipes/{}", collection_id, recipe_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_add_to_favorites_endpoint() {
    let app = create_test_app().await;
    
    let recipe_id = "456e7890-f12a-34b5-c678-901234567890";

    let request = Request::builder()
        .method("POST")
        .uri(&format!("/favorites/{}", recipe_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_remove_from_favorites_endpoint() {
    let app = create_test_app().await;
    
    let recipe_id = "456e7890-f12a-34b5-c678-901234567890";

    let request = Request::builder()
        .method("DELETE")  
        .uri(&format!("/favorites/{}", recipe_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_collection_detail_endpoint() {
    let app = create_test_app().await;
    
    let collection_id = "123e4567-e89b-12d3-a456-426614174000";

    let request = Request::builder()
        .uri(&format!("/collections/{}", collection_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should contain collection detail content
    assert!(body_str.contains("Sample Collection"));
}

#[tokio::test]
async fn test_collection_edit_form_endpoint() {
    let app = create_test_app().await;
    
    let collection_id = "123e4567-e89b-12d3-a456-426614174000";

    let request = Request::builder()
        .uri(&format!("/collections/{}/edit", collection_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    
    // Should contain edit form
    assert!(body_str.contains("Edit Collection"));
    assert!(body_str.contains("Update Collection"));
}

#[tokio::test]
async fn test_collection_update_endpoint() {
    let app = create_test_app().await;
    
    let collection_id = "123e4567-e89b-12d3-a456-426614174000";
    let form_data = "name=Updated%20Collection&description=Updated%20description&privacy=Public";

    let request = Request::builder()
        .method("PUT")
        .uri(&format!("/collections/{}", collection_id))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_collection_delete_endpoint() {
    let app = create_test_app().await;
    
    let collection_id = "123e4567-e89b-12d3-a456-426614174000";

    let request = Request::builder()
        .method("DELETE")
        .uri(&format!("/collections/{}", collection_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}