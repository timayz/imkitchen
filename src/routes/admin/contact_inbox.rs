//! Admin contact inbox route handlers

use crate::auth::jwt::AuthUser;
use crate::routes::{render_template, AppState};
use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::Response,
};
use imkitchen_user::command::{Command, MarkContactMessageReadInput, ResolveContactMessageInput};
use imkitchen_user::event::EventMetadata;
use serde::Deserialize;
use tracing::{error, info};
use ulid::Ulid;

/// Contact inbox page template
#[derive(Template)]
#[template(path = "pages/admin/contact_inbox.html")]
struct ContactInboxTemplate {
    messages: Vec<crate::queries::ContactMessageRow>,
    new_count: i64,
    read_count: i64,
    resolved_count: i64,
    current_filter: String,
}

/// Contact message row partial template
#[derive(Template)]
#[template(path = "partials/admin/contact_message_row.html")]
struct ContactMessageRowTemplate {
    message: crate::queries::ContactMessageRow,
}

/// Query parameters for filtering
#[derive(Deserialize)]
pub struct ContactFilterQuery {
    status: Option<String>,
}

/// GET /admin/contact - List contact messages with optional filtering
pub async fn list_contact_messages(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<ContactFilterQuery>,
) -> Response {
    info!(
        admin_user_id = %auth_user.user_id,
        status_filter = ?query.status,
        "Loading contact inbox"
    );

    // Get message counts for badges
    let counts = match crate::queries::count_messages_by_status(&state.query_pool).await {
        Ok(counts) => counts,
        Err(e) => {
            error!(error = %e, "Failed to count messages by status");
            (0, 0, 0)
        }
    };

    let (new_count, read_count, resolved_count) = counts;

    // Get filtered messages
    let messages =
        match crate::queries::list_contact_messages(&state.query_pool, query.status.as_deref())
            .await
        {
            Ok(messages) => messages,
            Err(e) => {
                error!(error = %e, "Failed to load contact messages");
                Vec::new()
            }
        };

    let current_filter = query.status.unwrap_or_else(|| "all".to_string());

    render_template(ContactInboxTemplate {
        messages,
        new_count,
        read_count,
        resolved_count,
        current_filter,
    })
}

/// POST /admin/contact/{id}/mark-read - Mark message as read
pub async fn mark_message_read(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(message_id): Path<String>,
) -> Response {
    info!(
        admin_user_id = %auth_user.user_id,
        message_id = %message_id,
        "Marking contact message as read"
    );

    // Fetch current message
    let current_message =
        match crate::queries::get_contact_message(&state.query_pool, &message_id).await {
            Ok(Some(msg)) => msg,
            Ok(None) | Err(_) => {
                error!(message_id = %message_id, "Message not found");
                return render_template(ContactMessageRowTemplate {
                    message: crate::queries::ContactMessageRow {
                        id: message_id,
                        name: "Error".to_string(),
                        email: "Message not found".to_string(),
                        subject: "".to_string(),
                        message: "".to_string(),
                        status: "new".to_string(),
                        created_at: 0,
                        created_at_formatted: "Invalid date".to_string(),
                    },
                });
            }
        };

    // Execute command
    let admin_user_id = Some(auth_user.user_id.clone());
    let input = MarkContactMessageReadInput {
        message_id: message_id.clone(),
    };
    let metadata = EventMetadata {
        user_id: admin_user_id,
        request_id: Ulid::new().to_string(),
    };

    let command = Command::new(state.evento.clone());
    if let Err(e) = command.mark_contact_message_read(input, metadata).await {
        error!(error = %e, message_id = %message_id, "Failed to mark message as read");
        return render_template(ContactMessageRowTemplate {
            message: current_message,
        });
    }

    // Return optimistic response (projection will update asynchronously)
    let optimistic_message = crate::queries::ContactMessageRow {
        status: "read".to_string(),
        ..current_message
    };

    render_template(ContactMessageRowTemplate {
        message: optimistic_message,
    })
}

/// POST /admin/contact/{id}/resolve - Mark message as resolved
pub async fn resolve_message(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(message_id): Path<String>,
) -> Response {
    info!(
        admin_user_id = %auth_user.user_id,
        message_id = %message_id,
        "Resolving contact message"
    );

    // Fetch current message
    let current_message =
        match crate::queries::get_contact_message(&state.query_pool, &message_id).await {
            Ok(Some(msg)) => msg,
            Ok(None) | Err(_) => {
                error!(message_id = %message_id, "Message not found");
                return render_template(ContactMessageRowTemplate {
                    message: crate::queries::ContactMessageRow {
                        id: message_id,
                        name: "Error".to_string(),
                        email: "Message not found".to_string(),
                        subject: "".to_string(),
                        message: "".to_string(),
                        status: "new".to_string(),
                        created_at: 0,
                        created_at_formatted: "Invalid date".to_string(),
                    },
                });
            }
        };

    // Execute command
    let admin_user_id = Some(auth_user.user_id.clone());
    let input = ResolveContactMessageInput {
        message_id: message_id.clone(),
    };
    let metadata = EventMetadata {
        user_id: admin_user_id,
        request_id: Ulid::new().to_string(),
    };

    let command = Command::new(state.evento.clone());
    if let Err(e) = command.resolve_contact_message(input, metadata).await {
        error!(error = %e, message_id = %message_id, "Failed to resolve message");
        return render_template(ContactMessageRowTemplate {
            message: current_message,
        });
    }

    // Return optimistic response (projection will update asynchronously)
    let optimistic_message = crate::queries::ContactMessageRow {
        status: "resolved".to_string(),
        ..current_message
    };

    render_template(ContactMessageRowTemplate {
        message: optimistic_message,
    })
}
