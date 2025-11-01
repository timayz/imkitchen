# Story 1.6: Contact Form and Admin Notifications

Status: drafted

## Story

As a visitor,
I want to submit questions or feedback through a contact form,
So that I can reach the platform administrators without needing an account.

## Acceptance Criteria

1. Public contact form route (no authentication required) with fields: name, email, subject, message
2. ContactFormSubmitted event stores submission data with timestamp
3. Query handler projects submissions to contact_messages table
4. Admin panel displays contact form inbox with read/resolved status
5. Email notification sent to configured admin email(s) on new submission
6. Admin can mark messages as read/resolved and filter by status
7. Tests verify form submission, projection, and admin access

## Tasks / Subtasks

- [ ] Task 1: Define ContactFormSubmitted event (AC: 2)
  - [ ] Create crates/imkitchen-contact/ bounded context (or add to imkitchen-user)
  - [ ] Define ContactFormSubmitted event with fields: name, email, subject, message
  - [ ] Create ContactMessage aggregate root
  - [ ] Implement aggregate handler for ContactFormSubmitted

- [ ] Task 2: Implement contact form submission command (AC: 1, 2)
  - [ ] Create SubmitContactFormInput struct with validation
  - [ ] Validate email format, required fields
  - [ ] Implement submit_contact_form command using evento::create
  - [ ] No authentication required (public route)
  - [ ] Generate message_id (ULID)

- [ ] Task 3: Create contact_messages projection table (AC: 3)
  - [ ] Create migration: migrations/queries/20250101000008_contact_messages.sql
  - [ ] Table fields: id, name, email, subject, message, status ('new'|'read'|'resolved'), created_at
  - [ ] Add index on status for filtering
  - [ ] Add index on created_at for sorting

- [ ] Task 4: Implement query handler for ContactFormSubmitted (AC: 3)
  - [ ] Create src/queries/contact.rs
  - [ ] Implement on_contact_form_submitted handler
  - [ ] Insert into contact_messages table with status = 'new'
  - [ ] Create subscription builder for contact query handlers

- [ ] Task 5: Create public contact form page (AC: 1)
  - [ ] Create templates/pages/contact.html
  - [ ] Form fields: name, email, subject (dropdown), message (textarea)
  - [ ] Add client-side validation (required fields, email format)
  - [ ] Style with Tailwind CSS
  - [ ] Twinspark form submission with success message
  - [ ] Add FAQ section below form (optional)

- [ ] Task 6: Implement contact form route handler (AC: 1)
  - [ ] Create src/routes/contact.rs
  - [ ] GET /contact - Render contact form (public access)
  - [ ] POST /contact - Submit contact form
  - [ ] Return success template after submission
  - [ ] Handle validation errors with error display

- [ ] Task 7: Implement email notification service (AC: 5)
  - [ ] Create src/email.rs module
  - [ ] Configure lettre SMTP client from config
  - [ ] Implement send_contact_notification function
  - [ ] Email template includes: submitter name, email, subject, message, timestamp
  - [ ] Load admin email addresses from config
  - [ ] Create event handler for ContactFormSubmitted that sends email

- [ ] Task 8: Configure SMTP settings (AC: 5)
  - [ ] Add [email] section to config/default.toml
  - [ ] Settings: smtp_host, smtp_port, smtp_username, smtp_password (empty), from_address, admin_emails
  - [ ] Document SMTP configuration in config/dev.toml example
  - [ ] Handle email sending failures gracefully (log error, don't block)

- [ ] Task 9: Create admin contact inbox (AC: 4, 6)
  - [ ] Create templates/pages/admin/contact_inbox.html
  - [ ] Display messages with: name, email, subject, message snippet, status, created_at
  - [ ] Filter by status: all / new / read / resolved
  - [ ] Sort by created_at (newest first)
  - [ ] Show message count badges (12 new, 34 read, 89 resolved)

- [ ] Task 10: Implement admin inbox route handlers (AC: 4, 6)
  - [ ] Create src/routes/admin/contact_inbox.rs
  - [ ] GET /admin/contact - List messages with filtering
  - [ ] POST /admin/contact/{id}/mark-read - Update status to 'read'
  - [ ] POST /admin/contact/{id}/resolve - Update status to 'resolved'
  - [ ] Return updated message row template (Twinspark partial)

- [ ] Task 11: Define status update events (AC: 6)
  - [ ] Add ContactMessageMarkedRead event
  - [ ] Add ContactMessageResolved event
  - [ ] Implement aggregate handlers for status changes
  - [ ] Implement commands: mark_contact_message_read, resolve_contact_message
  - [ ] Update query handler to handle status events

- [ ] Task 12: Write comprehensive tests (AC: 7)
  - [ ] Create tests/contact_test.rs
  - [ ] Test: Visitor can submit contact form without authentication
  - [ ] Test: Form validation (email format, required fields)
  - [ ] Test: Submission creates ContactFormSubmitted event
  - [ ] Test: Query handler projects submission to contact_messages table
  - [ ] Test: Admin can view contact inbox
  - [ ] Test: Admin can mark message as read
  - [ ] Test: Admin can resolve message
  - [ ] Test: Non-admin cannot access inbox (403)
  - [ ] Test: Email notification sent on submission (mock SMTP)

## Dev Notes

### Architecture Patterns

From [architecture.md](../architecture.md):

**Contact Message Aggregate:**

May be part of imkitchen-user bounded context or separate imkitchen-contact context. Recommendation: Add to imkitchen-user for simplicity.

**contact_messages table:**
```sql
CREATE TABLE contact_messages (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    subject TEXT NOT NULL,
    message TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'new',  -- 'new' | 'read' | 'resolved'
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_contact_messages_status ON contact_messages(status);
CREATE INDEX idx_contact_messages_created_at ON contact_messages(created_at DESC);
```

**Email Configuration:**

config/default.toml:
```toml
[email]
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_username = ""  # Set in dev.toml
smtp_password = ""  # Set in dev.toml (never commit)
from_address = "noreply@imkitchen.app"
admin_emails = ["admin@imkitchen.app"]
```

config/dev.toml (.gitignored):
```toml
[email]
smtp_username = "your-email@gmail.com"
smtp_password = "your-app-specific-password"
admin_emails = ["your-dev-email@example.com"]
```

### Email Service Implementation

From [architecture.md](../architecture.md#adr-008-configurable-smtp-with-lettre):

**lettre Integration:**
```rust
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;

pub struct EmailService {
    mailer: SmtpTransport,
    from: String,
    admin_emails: Vec<String>,
}

impl EmailService {
    pub fn new(config: &EmailConfig) -> anyhow::Result<Self> {
        let creds = Credentials::new(
            config.smtp_username.clone(),
            config.smtp_password.clone(),
        );

        let mailer = SmtpTransport::relay(&config.smtp_host)?
            .port(config.smtp_port)
            .credentials(creds)
            .build();

        Ok(Self {
            mailer,
            from: config.from_address.clone(),
            admin_emails: config.admin_emails.clone(),
        })
    }

    pub async fn send_contact_notification(
        &self,
        submission: &ContactFormSubmitted,
    ) -> anyhow::Result<()> {
        for admin_email in &self.admin_emails {
            let email = Message::builder()
                .from(self.from.parse()?)
                .to(admin_email.parse()?)
                .subject(format!("New Contact Form: {}", submission.subject))
                .body(format!(
                    "From: {} <{}>\nSubject: {}\n\n{}",
                    submission.name, submission.email,
                    submission.subject, submission.message
                ))?;

            self.mailer.send(&email)?;
        }
        Ok(())
    }
}
```

**Event Handler for Email:**
```rust
#[evento::handler(ContactMessage)]
async fn on_contact_form_submitted<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<ContactFormSubmitted, EventMetadata>,
) -> anyhow::Result<()> {
    let email_service = context.extract::<EmailService>();

    // Send email notification (don't fail if email fails)
    if let Err(e) = email_service.send_contact_notification(&event.data).await {
        warn!("Failed to send contact notification email: {}", e);
    }

    Ok(())
}
```

### Public Route Pattern

Contact form is publicly accessible (no authentication):
```rust
let public_routes = Router::new()
    .route("/contact", get(contact::get_form).post(contact::submit_form));

let protected_routes = Router::new()
    .route("/admin/contact", get(admin::contact_inbox::list))
    .layer(middleware::from_fn(auth_middleware));
```

From [CLAUDE.md](/home/snapiz/projects/github/timayz/imkitchen/CLAUDE.md#server-side-rendering):
- Always render HTML with status 200 (no REST API patterns)
- Use Twinspark for form submission and partial updates

### References

- [PRD: FR049-FR050](../PRD.md#contact--support) - Public contact form requirements
- [PRD: FR046](../PRD.md#admin-panel) - Admin contact inbox
- [Architecture: Email](../architecture.md#email) - lettre configuration
- [Architecture: ADR-008](../architecture.md#adr-008-configurable-smtp-with-lettre) - SMTP decision
- [Mockups: contact.html](../../mockups/contact.html) - Visual reference for contact form
- [Mockups: admin-contact.html](../../mockups/admin-contact.html) - Visual reference for admin inbox

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Will be filled by Dev agent -->

### Debug Log References

<!-- Dev agent logs will be added here -->

### Completion Notes List

<!-- Dev agent completion notes will be added here -->

### File List

<!-- List of files created/modified will be added here -->
