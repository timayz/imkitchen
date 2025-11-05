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

- [ ] Task 1: Create Contact bounded context (AC: 1, 2)
  - [ ] Create crates/imkitchen-contact/ directory with Cargo.toml
  - [ ] Create src/lib.rs, src/event.rs, src/aggregate.rs, src/command.rs
  - [ ] Define EventMetadata struct (same pattern as User)
  - [ ] Define ContactFormSubmitted event with fields: name, email, subject, message
  - [ ] Define ContactMessage aggregate (minimal state tracking)
  - [ ] Implement contact_form_submitted handler in aggregate.rs
  - [ ] Add imkitchen-contact to workspace members

- [ ] Task 2: Implement submit contact form command (AC: 1)
  - [ ] Define SubmitContactFormInput struct with validation
  - [ ] Validate: email format, name/subject/message not empty, message max 2000 chars
  - [ ] Create Command struct with Executor (no validation DB needed)
  - [ ] Implement submit_contact_form command
  - [ ] Use evento::create pattern to emit ContactFormSubmitted event
  - [ ] Command returns contact_message_id on success

- [ ] Task 3: Create contact_messages projection (AC: 3)
  - [ ] Create migration: migrations/queries/TIMESTAMP_contact_messages.sql
  - [ ] Define contact_messages table: id (TEXT PK), name (TEXT), email (TEXT), subject (TEXT), message (TEXT), status (TEXT DEFAULT 'new'), created_at (INTEGER)
  - [ ] Create src/queries/contact.rs with query handler on_contact_form_submitted
  - [ ] Handler inserts into contact_messages with status='new'
  - [ ] Use event.timestamp for created_at field
  - [ ] Create subscribe_contact_query function returning SubscriptionBuilder

- [ ] Task 4: Create public contact form route (AC: 1)
  - [ ] Create src/routes/contact.rs
  - [ ] GET /contact renders public contact form (no auth required)
  - [ ] POST /contact submits SubmitContactFormInput via Form extractor
  - [ ] Call submit_contact_form command
  - [ ] Return success template or error message on validation failure
  - [ ] Form includes: name, email, subject dropdown (Question/Feedback/Bug Report/Other), message textarea

- [ ] Task 5: Create contact form templates (AC: 1)
  - [ ] Create templates/pages/contact.html with Askama template
  - [ ] Public page accessible without login
  - [ ] Form fields with Tailwind CSS styling
  - [ ] Success message displayed after submission
  - [ ] Link to /contact in footer of all pages
  - [ ] FAQ section below form with common questions

- [ ] Task 6: Implement admin contact inbox (AC: 4, 6)
  - [ ] Create src/routes/admin/contact_inbox.rs
  - [ ] GET /admin/contact displays all contact messages with pagination
  - [ ] Display: name, email, subject, message preview, status, created_at
  - [ ] Filter by status: all / new / read / resolved
  - [ ] POST /admin/contact/{id}/mark-read updates status to 'read'
  - [ ] POST /admin/contact/{id}/mark-resolved updates status to 'resolved'
  - [ ] Query function: get_contact_messages(pool, status_filter, page) -> Vec<ContactMessage>

- [ ] Task 7: Update contact message status (AC: 6)
  - [ ] Define ContactMessageStatusUpdated event with new_status field
  - [ ] Implement update_contact_status command in Contact aggregate
  - [ ] Create query handler on_contact_message_status_updated
  - [ ] Handler updates contact_messages.status in projection
  - [ ] Admin actions trigger status updates via command

- [ ] Task 8: Email notification service (AC: 5)
  - [ ] Add lettre 0.11.14 to workspace dependencies
  - [ ] Create src/email.rs with EmailService struct
  - [ ] Configure SMTP settings in config/default.toml: smtp_host, smtp_port, smtp_username, smtp_password (empty), from_address, admin_emails (array)
  - [ ] Implement send_contact_notification(&self, contact: ContactMessage) -> Result
  - [ ] Email includes: submitter name, email, subject, message, timestamp, link to admin panel
  - [ ] Add EmailService to AppState

- [ ] Task 9: Trigger email on contact submission (AC: 5)
  - [ ] Create event handler on_contact_form_submitted_email in src/queries/contact.rs
  - [ ] Handler calls EmailService.send_contact_notification
  - [ ] If email fails, log error but don't fail projection (fire-and-forget pattern)
  - [ ] Add email handler to contact subscription (same subscription as projection handler)
  - [ ] Note: Email failures won't block contact submission

- [ ] Task 10: Create admin inbox template (AC: 4, 6)
  - [ ] Create templates/pages/admin/contact_inbox.html
  - [ ] Message table with status badges (new=yellow, read=blue, resolved=green)
  - [ ] Action buttons: Mark Read, Mark Resolved
  - [ ] Filter tabs: All / New / Read / Resolved
  - [ ] Message preview shows first 100 chars, click to expand
  - [ ] Use Twinspark for inline status updates (no page reload)

- [ ] Task 11: Testing (AC: 7)
  - [ ] Create tests/contact_test.rs
  - [ ] Test: Submit contact form creates projection
  - [ ] Test: Invalid email rejected by validation
  - [ ] Test: Message exceeding 2000 chars rejected
  - [ ] Test: Admin can view contact inbox
  - [ ] Test: Admin can mark message as read
  - [ ] Test: Admin can mark message as resolved
  - [ ] Test: Non-admin cannot access /admin/contact (403)
  - [ ] Test: Email notification sent on submission (mock SMTP)

- [ ] Task 12: Code quality validation
  - [ ] Run cargo clippy and fix all warnings
  - [ ] Run cargo fmt --all
  - [ ] Verify all tests pass: cargo test
  - [ ] Manual test: Submit contact form, verify email received, mark as read in admin panel

## Dev Notes

### Architecture Patterns

**Public Form (No Authentication):**
- Contact form route does NOT require authentication
- Accessible to visitors and logged-in users alike
- No CAPTCHA in MVP (can be added post-MVP if spam becomes issue)

**Email Notification (Fire-and-Forget):**
- Email service called in event handler, failures logged but don't block
- Contact submission succeeds even if email fails
- Admin can still see messages in inbox if email delivery fails
- SMTP configuration in config/default.toml, credentials in config/dev.toml (.gitignored)

**Admin Inbox Management:**
- Status flow: new → read → resolved
- Status transitions reversible (can mark resolved as new again)
- Pagination for large message volumes (20 per page)
- Filter by status for efficient inbox management

**SMTP Configuration:**
```toml
# config/default.toml
[email]
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_username = "admin@imkitchen.app"
smtp_password = ""  # Override in dev.toml
from_address = "noreply@imkitchen.app"
admin_emails = ["admin@imkitchen.app"]
```

**Event Handler Pattern (Multiple Handlers, One Subscription):**
- Same subscription handles both projection and email notification
- on_contact_form_submitted → updates contact_messages table
- on_contact_form_submitted_email → sends email notification
- Both handlers process same event concurrently

### Project Structure Notes

New directories and files added:
- `crates/imkitchen-contact/` - Contact bounded context
- `src/routes/contact.rs` - Public contact form route
- `src/routes/admin/contact_inbox.rs` - Admin inbox management
- `src/queries/contact.rs` - Contact projections and email handler
- `src/email.rs` - Email service (lettre integration)
- `templates/pages/contact.html` - Public contact form
- `templates/pages/admin/contact_inbox.html` - Admin inbox view
- `migrations/queries/TIMESTAMP_contact_messages.sql` - Contact projection table
- `tests/contact_test.rs` - Contact form and inbox tests

**Email Testing:**
- Use mock SMTP server for tests (e.g., MailHog, smtp4dev)
- Or use lettre's test transport for unit tests
- Manual testing with real SMTP during development

### References

- [Source: docs/epics.md#Story 1.6] - Complete acceptance criteria
- [Source: docs/architecture.md#Email] - lettre configuration
- [Source: docs/architecture.md#Deployment Architecture] - SMTP configuration pattern
- [Source: docs/PRD.md#Requirements FR049-FR050] - Contact form and notifications
- [Source: CLAUDE.md#Email Guidelines] - Askama for email templates
- [Source: CLAUDE.md#Command Guidelines] - evento::create pattern

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Will be populated during implementation -->

### Debug Log References

<!-- Will be populated during implementation -->

### Completion Notes List

<!-- Will be populated during implementation -->

### File List

<!-- Will be populated during implementation -->
