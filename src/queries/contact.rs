//! Contact query handlers and projections

use crate::email::EmailService;
use chrono::DateTime;
use evento::{AggregatorName, Context, EventDetails, Executor};
use imkitchen_user::aggregate::ContactMessage;
use imkitchen_user::event::{
    ContactFormSubmitted, ContactMessageMarkedRead, ContactMessageResolved, EventMetadata,
};
use sqlx::SqlitePool;
use tracing::{info, warn};

/// Contact message row from projection table
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ContactMessageRow {
    pub id: String,
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
    pub status: String,
    pub created_at: i64,
    #[sqlx(skip)]
    pub created_at_formatted: String,
}

/// Handler for ContactFormSubmitted event - creates projection and sends email
#[evento::handler(ContactMessage)]
async fn on_contact_form_submitted<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<ContactFormSubmitted, EventMetadata>,
) -> anyhow::Result<()> {
    let pool = context.extract::<SqlitePool>();
    let email_service = context.extract::<EmailService>();

    info!(
        message_id = %event.aggregator_id,
        email = %event.data.email,
        subject = %event.data.subject,
        "Processing ContactFormSubmitted event"
    );

    // Insert into contact_messages projection table (idempotent)
    sqlx::query(
        "INSERT OR IGNORE INTO contact_messages (id, name, email, subject, message, status, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&event.aggregator_id)
    .bind(&event.data.name)
    .bind(&event.data.email)
    .bind(&event.data.subject)
    .bind(&event.data.message)
    .bind("new") // Initial status
    .bind(event.timestamp)
    .execute(&pool)
    .await?;

    info!(
        message_id = %event.aggregator_id,
        "Contact message projection created successfully"
    );

    // Send email notification (don't fail if email fails)
    if let Err(e) = email_service.send_contact_notification(&event.data).await {
        warn!(
            error = %e,
            message_id = %event.aggregator_id,
            "Failed to send contact notification email"
        );
    }

    Ok(())
}

/// Handler for ContactMessageMarkedRead event
#[evento::handler(ContactMessage)]
async fn on_contact_message_marked_read<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<ContactMessageMarkedRead, EventMetadata>,
) -> anyhow::Result<()> {
    let pool = context.extract::<SqlitePool>();

    info!(
        message_id = %event.aggregator_id,
        "Processing ContactMessageMarkedRead event"
    );

    // Update contact_messages table to set status = 'read'
    sqlx::query("UPDATE contact_messages SET status = ? WHERE id = ?")
        .bind("read")
        .bind(&event.aggregator_id)
        .execute(&pool)
        .await?;

    info!(
        message_id = %event.aggregator_id,
        "Contact message marked as read in projection"
    );

    Ok(())
}

/// Handler for ContactMessageResolved event
#[evento::handler(ContactMessage)]
async fn on_contact_message_resolved<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<ContactMessageResolved, EventMetadata>,
) -> anyhow::Result<()> {
    let pool = context.extract::<SqlitePool>();

    info!(
        message_id = %event.aggregator_id,
        "Processing ContactMessageResolved event"
    );

    // Update contact_messages table to set status = 'resolved'
    sqlx::query("UPDATE contact_messages SET status = ? WHERE id = ?")
        .bind("resolved")
        .bind(&event.aggregator_id)
        .execute(&pool)
        .await?;

    info!(
        message_id = %event.aggregator_id,
        "Contact message marked as resolved in projection"
    );

    Ok(())
}

/// Create subscription builder for contact query handlers
pub fn subscribe_contact_query<E: Executor + Clone>(
    pool: SqlitePool,
    email_service: EmailService,
) -> evento::SubscribeBuilder<E> {
    evento::subscribe::<E>("contact-query")
        .data(pool)
        .data(email_service)
        .handler(on_contact_form_submitted())
        .handler(on_contact_message_marked_read())
        .handler(on_contact_message_resolved())
}

/// List contact messages with optional status filter (admin only)
pub async fn list_contact_messages(
    pool: &SqlitePool,
    status_filter: Option<&str>,
) -> anyhow::Result<Vec<ContactMessageRow>> {
    let mut messages = match status_filter {
        Some(status) => {
            sqlx::query_as::<_, ContactMessageRow>(
                "SELECT id, name, email, subject, message, status, created_at
                 FROM contact_messages
                 WHERE status = ?
                 ORDER BY created_at DESC",
            )
            .bind(status)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, ContactMessageRow>(
                "SELECT id, name, email, subject, message, status, created_at
                 FROM contact_messages
                 ORDER BY created_at DESC",
            )
            .fetch_all(pool)
            .await?
        }
    };

    // Format timestamps for display
    for message in &mut messages {
        message.created_at_formatted = format_timestamp(message.created_at);
    }

    Ok(messages)
}

/// Format Unix timestamp as human-readable date
fn format_timestamp(timestamp: i64) -> String {
    DateTime::from_timestamp(timestamp, 0)
        .map(|dt| dt.format("%b %d, %Y %I:%M %p").to_string())
        .unwrap_or_else(|| "Invalid date".to_string())
}

/// Get contact message by ID (admin only)
pub async fn get_contact_message(
    pool: &SqlitePool,
    message_id: &str,
) -> anyhow::Result<Option<ContactMessageRow>> {
    let mut message = sqlx::query_as::<_, ContactMessageRow>(
        "SELECT id, name, email, subject, message, status, created_at
         FROM contact_messages
         WHERE id = ?",
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    // Format timestamp for display
    if let Some(ref mut msg) = message {
        msg.created_at_formatted = format_timestamp(msg.created_at);
    }

    Ok(message)
}

/// Count messages by status (for admin dashboard badges)
pub async fn count_messages_by_status(pool: &SqlitePool) -> anyhow::Result<(i64, i64, i64)> {
    #[derive(sqlx::FromRow)]
    struct StatusCount {
        status: String,
        count: i64,
    }

    // Single query with GROUP BY for better performance
    let counts = sqlx::query_as::<_, StatusCount>(
        "SELECT status, COUNT(*) as count FROM contact_messages GROUP BY status",
    )
    .fetch_all(pool)
    .await?;

    // Map results to expected format, defaulting to 0 for missing statuses
    let mut new_count = 0i64;
    let mut read_count = 0i64;
    let mut resolved_count = 0i64;

    for status_count in counts {
        match status_count.status.as_str() {
            "new" => new_count = status_count.count,
            "read" => read_count = status_count.count,
            "resolved" => resolved_count = status_count.count,
            _ => {} // Ignore unknown statuses
        }
    }

    Ok((new_count, read_count, resolved_count))
}
