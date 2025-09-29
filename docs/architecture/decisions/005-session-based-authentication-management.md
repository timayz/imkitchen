# ADR-005: Session-Based Authentication and Authorization

## Status
Accepted

## Context
IMKitchen requires a robust authentication and authorization system that:

- **Works in kitchen environments**: Resistant touchscreen devices, potential network interruptions
- **Supports multiple users**: Shared kitchen devices with quick user switching
- **Maintains security**: Protect sensitive recipe data and user information
- **Enables offline access**: Core functionality available without network connectivity
- **Simplifies development**: Clear authentication patterns across the application
- **Scales appropriately**: Handle multiple concurrent users efficiently

Kitchen environments present unique challenges:
- **Shared devices**: Multiple cooks may use the same tablet/terminal
- **Quick sessions**: Users need fast login/logout for shift changes
- **Offline periods**: Network connectivity may be intermittent
- **Security vs. convenience**: Balance between security and kitchen workflow efficiency

Traditional authentication approaches:
- **JWT tokens**: Stateless but complex refresh logic, difficult revocation
- **OAuth2/OpenID Connect**: Too complex for kitchen environment needs
- **Basic auth**: Insecure, no session management
- **No authentication**: Inappropriate for multi-user environment

## Decision
We will implement **Session-Based Authentication** with:

1. **Secure server-side sessions**: Session data stored securely on server
2. **HTTP-only cookies**: Prevent XSS attacks, automatic inclusion in requests
3. **Session timeout management**: Automatic logout after inactivity
4. **Role-based authorization**: Different access levels for different user types
5. **Device-aware sessions**: Support for shared devices with quick user switching
6. **Offline session caching**: Limited offline access with cached session data

## Alternatives Considered

### JWT (JSON Web Tokens) Authentication
**Pros:**
- Stateless authentication
- No server-side session storage required
- Good for microservices and APIs
- Built-in expiration handling

**Cons:**
- Complex refresh token logic
- Difficult to revoke tokens immediately
- Vulnerable to XSS if stored in localStorage
- Payload size overhead in every request
- Stateless nature makes user session management complex

### OAuth2 with OpenID Connect
**Pros:**
- Industry standard for authentication
- Supports multiple identity providers
- Good for enterprise integration
- Comprehensive authorization flows

**Cons:**
- Complex implementation and maintenance
- Overkill for kitchen management application
- Requires external identity provider
- Poor user experience for quick kitchen workflows
- Network dependency for authentication

### Basic HTTP Authentication
**Pros:**
- Simple implementation
- Built into HTTP standard
- No session management complexity

**Cons:**
- Credentials sent with every request
- No logout functionality
- Poor user experience
- No role-based access control
- Security vulnerabilities

### Client-Side Cookie Storage with Local State
**Pros:**
- Simple implementation
- Fast client-side access
- No server-side session storage

**Cons:**
- Vulnerable to XSS attacks
- No server-side session control
- Difficult to implement security policies
- Poor shared device support

## Consequences

### Positive
- **Security**: HTTP-only cookies prevent XSS attacks, server-side session control
- **User Experience**: Seamless authentication across requests, automatic session handling
- **Shared Device Support**: Easy user switching with proper session isolation
- **Session Management**: Server can control session lifetime, immediate revocation possible
- **Kitchen Workflow**: Quick login/logout suitable for kitchen environments
- **Development Simplicity**: Clear session patterns, integrated with Axum middleware
- **Offline Capability**: Session data can be cached for limited offline access
- **Role-Based Access**: Easy implementation of different permission levels

### Negative
- **Server State**: Requires server-side session storage and management
- **Scalability**: Session storage can become bottleneck at very high scale
- **Complexity**: More complex than stateless approaches
- **Memory Usage**: Server memory required for active sessions

### Risks
- **Session Storage Scalability**: Risk of session storage becoming bottleneck
  - *Mitigation*: Use efficient session storage (Redis), implement session cleanup
- **Session Hijacking**: Risk of session cookie theft
  - *Mitigation*: HTTPS only, secure cookie flags, IP validation
- **Session Fixation**: Risk of session ID prediction or fixation
  - *Mitigation*: Secure random session generation, session regeneration on login

## Implementation Notes

### Session Storage Architecture
```rust
// Session storage abstraction
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create_session(&self, user_id: &UserId, session_data: SessionData) -> Result<SessionId, SessionError>;
    async fn get_session(&self, session_id: &SessionId) -> Result<Option<SessionData>, SessionError>;
    async fn update_session(&self, session_id: &SessionId, session_data: SessionData) -> Result<(), SessionError>;
    async fn delete_session(&self, session_id: &SessionId) -> Result<(), SessionError>;
    async fn cleanup_expired_sessions(&self) -> Result<u64, SessionError>;
}

// In-memory session store for development
pub struct MemorySessionStore {
    sessions: Arc<RwLock<HashMap<SessionId, (SessionData, Instant)>>>,
    cleanup_interval: Duration,
}

// Redis session store for production
pub struct RedisSessionStore {
    redis: redis::Client,
    key_prefix: String,
    ttl: Duration,
}
```

### Session Data Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: UserId,
    pub user_email: String,
    pub user_name: String,
    pub user_roles: Vec<UserRole>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub device_info: DeviceInfo,
    pub preferences: UserPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: Option<String>,
    pub user_agent: String,
    pub ip_address: String,
    pub is_shared_device: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Admin,          // Full system access
    KitchenManager, // Kitchen operations management
    Cook,           // Recipe access and cooking features
    ViewOnly,       // Read-only access
}
```

### Authentication Middleware
```rust
use axum::{
    extract::{Request, State},
    http::{header::COOKIE, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};

pub async fn auth_middleware(
    State(session_store): State<Arc<dyn SessionStore>>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Extract session cookie
    let session_id = extract_session_cookie(&request)
        .ok_or_else(|| Redirect::to("/login").into_response())?;
    
    // Validate session
    let session_data = session_store
        .get_session(&session_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
        .ok_or_else(|| Redirect::to("/login").into_response())?;
    
    // Check session expiration
    if is_session_expired(&session_data) {
        session_store.delete_session(&session_id).await.ok();
        return Err(Redirect::to("/login").into_response());
    }
    
    // Update last activity
    let mut updated_session = session_data.clone();
    updated_session.last_activity = Utc::now();
    session_store.update_session(&session_id, updated_session).await.ok();
    
    // Add session data to request extensions
    request.extensions_mut().insert(UserSession {
        session_id,
        user_id: session_data.user_id,
        user_email: session_data.user_email,
        user_name: session_data.user_name,
        user_roles: session_data.user_roles,
        device_info: session_data.device_info,
    });
    
    Ok(next.run(request).await)
}

fn extract_session_cookie(request: &Request) -> Option<SessionId> {
    request
        .headers()
        .get(COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|cookie| {
            let mut parts = cookie.trim().splitn(2, '=');
            let name = parts.next()?;
            let value = parts.next()?;
            if name == "session_id" {
                Some(SessionId(value.to_string()))
            } else {
                None
            }
        })
}
```

### Login/Logout Handlers
```rust
// Login handler
pub async fn login_handler(
    State(app_state): State<AppState>,
    Form(login_form): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    // Validate credentials
    let user = app_state
        .user_service
        .authenticate(&login_form.email, &login_form.password)
        .await?;
    
    // Create session
    let session_data = SessionData {
        user_id: user.id.clone(),
        user_email: user.email.clone(),
        user_name: user.display_name.clone(),
        user_roles: user.roles.clone(),
        created_at: Utc::now(),
        last_activity: Utc::now(),
        device_info: DeviceInfo {
            device_id: login_form.device_id,
            user_agent: login_form.user_agent,
            ip_address: login_form.ip_address,
            is_shared_device: login_form.is_shared_device.unwrap_or(false),
        },
        preferences: user.preferences,
    };
    
    let session_id = app_state
        .session_store
        .create_session(&user.id, session_data)
        .await?;
    
    // Set secure cookie
    let cookie = create_session_cookie(&session_id, app_state.config.session_config());
    
    Ok((
        AppendHeaders([(SET_COOKIE, cookie)]),
        Redirect::to("/dashboard"),
    ))
}

// Logout handler
pub async fn logout_handler(
    State(session_store): State<Arc<dyn SessionStore>>,
    session: UserSession,
) -> Result<impl IntoResponse, AppError> {
    // Delete session
    session_store.delete_session(&session.session_id).await?;
    
    // Clear cookie
    let clear_cookie = "session_id=; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=0";
    
    Ok((
        AppendHeaders([(SET_COOKIE, clear_cookie)]),
        Redirect::to("/login"),
    ))
}

fn create_session_cookie(session_id: &SessionId, config: &SessionConfig) -> String {
    format!(
        "session_id={}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age={}",
        session_id.0,
        config.max_age_seconds
    )
}
```

### Authorization System
```rust
// Authorization middleware for role-based access
pub fn require_role(required_role: UserRole) -> impl Clone + Fn(Request, Next) -> Pin<Box<dyn Future<Output = Result<Response, Response>> + Send>> {
    move |request: Request, next: Next| {
        let required_role = required_role.clone();
        Box::pin(async move {
            let session = request
                .extensions()
                .get::<UserSession>()
                .ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())?;
            
            if !session.has_role(&required_role) {
                return Err(StatusCode::FORBIDDEN.into_response());
            }
            
            Ok(next.run(request).await)
        })
    }
}

impl UserSession {
    pub fn has_role(&self, role: &UserRole) -> bool {
        self.user_roles.contains(role) || self.user_roles.contains(&UserRole::Admin)
    }
    
    pub fn can_access_recipe(&self, recipe: &Recipe) -> bool {
        match &self.user_roles[..] {
            roles if roles.contains(&UserRole::Admin) => true,
            roles if roles.contains(&UserRole::KitchenManager) => true,
            roles if roles.contains(&UserRole::Cook) => {
                // Cooks can access public recipes or their own recipes
                recipe.is_public || recipe.created_by == self.user_id
            }
            _ => recipe.is_public, // ViewOnly users can only see public recipes
        }
    }
}
```

### Session Configuration
```rust
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub max_age_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub absolute_timeout_seconds: u64,
    pub secure_cookies: bool,
    pub same_site: SameSite,
    pub cleanup_interval_seconds: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_age_seconds: 24 * 60 * 60,      // 24 hours
            idle_timeout_seconds: 2 * 60 * 60,  // 2 hours idle
            absolute_timeout_seconds: 8 * 60 * 60, // 8 hours absolute
            secure_cookies: true,
            same_site: SameSite::Lax,
            cleanup_interval_seconds: 60 * 60,  // 1 hour cleanup
        }
    }
}

// Environment-specific overrides
impl SessionConfig {
    pub fn for_development() -> Self {
        Self {
            secure_cookies: false, // Allow HTTP in development
            max_age_seconds: 60 * 60, // 1 hour for development
            ..Default::default()
        }
    }
    
    pub fn for_production() -> Self {
        Self {
            secure_cookies: true,
            same_site: SameSite::Strict,
            ..Default::default()
        }
    }
}
```

### Quick User Switching for Shared Devices
```rust
// Quick switch handler for shared kitchen devices
pub async fn quick_switch_handler(
    State(app_state): State<AppState>,
    current_session: UserSession,
    Form(switch_form): Form<QuickSwitchForm>,
) -> Result<impl IntoResponse, AppError> {
    // Verify device is marked as shared
    if !current_session.device_info.is_shared_device {
        return Err(AppError::Forbidden("Quick switch not enabled for this device".to_string()));
    }
    
    // Authenticate new user
    let new_user = app_state
        .user_service
        .authenticate(&switch_form.email, &switch_form.password)
        .await?;
    
    // Delete old session
    app_state.session_store.delete_session(&current_session.session_id).await?;
    
    // Create new session with same device info
    let session_data = SessionData {
        user_id: new_user.id.clone(),
        user_email: new_user.email.clone(),
        user_name: new_user.display_name.clone(),
        user_roles: new_user.roles.clone(),
        created_at: Utc::now(),
        last_activity: Utc::now(),
        device_info: current_session.device_info, // Preserve device info
        preferences: new_user.preferences,
    };
    
    let session_id = app_state
        .session_store
        .create_session(&new_user.id, session_data)
        .await?;
    
    let cookie = create_session_cookie(&session_id, app_state.config.session_config());
    
    Ok((
        AppendHeaders([(SET_COOKIE, cookie)]),
        Json(json!({ "success": true, "user": new_user.display_name })),
    ))
}
```

### Session Cleanup and Monitoring
```rust
// Background task for session cleanup
pub async fn session_cleanup_task(session_store: Arc<dyn SessionStore>) {
    let mut interval = tokio::time::interval(Duration::from_secs(3600)); // 1 hour
    
    loop {
        interval.tick().await;
        
        match session_store.cleanup_expired_sessions().await {
            Ok(cleaned_count) => {
                tracing::info!("Cleaned up {} expired sessions", cleaned_count);
            }
            Err(error) => {
                tracing::error!("Failed to cleanup sessions: {}", error);
            }
        }
    }
}

// Session metrics for monitoring
pub struct SessionMetrics {
    pub active_sessions: u64,
    pub sessions_created_last_hour: u64,
    pub sessions_expired_last_hour: u64,
    pub average_session_duration: Duration,
}

impl SessionStore for RedisSessionStore {
    async fn get_metrics(&self) -> Result<SessionMetrics, SessionError> {
        // Implementation to gather session statistics
        // Used for monitoring and alerting
    }
}
```

## References
- [OWASP Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
- [HTTP Cookies Security](https://developer.mozilla.org/en-US/docs/Web/HTTP/Cookies#security)
- [Axum Session Management](https://docs.rs/axum/latest/axum/)
- [Redis Session Storage](https://redis.io/docs/manual/keyspace-notifications/)
- [Kitchen Environment Security Requirements](../requirements/security.md)