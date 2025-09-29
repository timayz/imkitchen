# ADR-001: Use Rust with Askama for Server-Side Rendering

## Status
Accepted

## Context
IMKitchen is a kitchen management application that needs to deliver fast, reliable web interfaces optimized for kitchen environments. The application requires:

- **Performance**: Fast page loads in resource-constrained kitchen environments
- **Reliability**: High uptime for critical kitchen operations
- **Security**: Memory safety and type safety for handling sensitive data
- **Maintainability**: Clear code structure with compile-time error checking
- **Progressive Enhancement**: Functional without JavaScript for maximum compatibility

Traditional options considered:
- **Node.js with React/Vue**: High resource usage, runtime errors, JavaScript fatigue
- **PHP with traditional templates**: Security concerns, type safety issues
- **Python with Django/Flask**: Runtime performance limitations
- **Go with templates**: Limited ecosystem for complex template logic
- **Traditional Java/Spring**: High memory usage, complex configuration

## Decision
We will use **Rust with the Askama template engine** for server-side rendering, providing:

1. **Rust Language**: Memory safety, type safety, and performance
2. **Askama Templates**: Compile-time template validation with Jinja2-like syntax
3. **Axum Framework**: Modern async HTTP server with middleware support
4. **Server-Side Rendering**: HTML generated on server for fast initial page loads

## Alternatives Considered

### Node.js with Next.js/React
**Pros:**
- Large ecosystem and community
- Familiar to many developers
- Good SSR support with Next.js

**Cons:**
- High memory usage (400-800MB baseline)
- Runtime errors and type safety issues
- Complex build processes and dependency management
- Performance overhead of JavaScript execution

### Python with Django/FastAPI + Jinja2
**Pros:**
- Rapid development with familiar syntax
- Excellent ecosystem for web development
- Good template engine with Jinja2

**Cons:**
- Runtime performance limitations
- GIL limitations for concurrency
- Memory usage higher than compiled languages
- Runtime type errors

### Go with html/template
**Pros:**
- Excellent performance and low memory usage
- Strong concurrency model
- Fast compilation

**Cons:**
- Limited template functionality
- Verbose syntax for complex logic
- Smaller ecosystem for web templates

### Java with Spring Boot + Thymeleaf
**Pros:**
- Mature ecosystem and tooling
- Strong enterprise support
- Good template engines available

**Cons:**
- High memory usage (JVM overhead)
- Complex configuration and startup time
- Not optimal for resource-constrained environments

## Consequences

### Positive
- **Compile-time Safety**: All templates validated at compile time, eliminating runtime template errors
- **Performance**: Native performance with minimal runtime overhead (~10-50MB memory usage)
- **Security**: Memory safety eliminates entire classes of vulnerabilities (buffer overflows, use-after-free)
- **Type Safety**: Full type checking across template boundaries prevents data inconsistencies
- **Developer Experience**: IDE support with syntax highlighting and error checking for templates
- **Zero-Cost Abstractions**: High-level features without runtime performance penalties
- **Deployment Simplicity**: Single binary deployment with no runtime dependencies
- **Concurrency**: Excellent async/await support for handling multiple requests efficiently

### Negative
- **Learning Curve**: Rust has a steeper learning curve than interpreted languages
- **Compilation Time**: Longer build times compared to interpreted languages (mitigated by incremental compilation)
- **Ecosystem Maturity**: Smaller ecosystem compared to Node.js or Python (but growing rapidly)
- **Template Limitations**: Askama is less feature-rich than some mature template engines
- **Development Workflow**: Requires compilation step, unlike interpreted languages

### Risks
- **Team Adoption**: Risk of slow team adoption due to Rust learning curve
  - *Mitigation*: Comprehensive documentation, training, and gradual introduction
- **Library Ecosystem**: Risk of missing required libraries
  - *Mitigation*: Careful evaluation of dependencies, willingness to implement features or use FFI
- **Template Engine Limitations**: Askama may lack specific features needed
  - *Mitigation*: Active contribution to Askama project, custom template functions as needed

## Implementation Notes

### Template Structure
```rust
#[derive(Template)]
#[template(path = "pages/dashboard.html")]
pub struct DashboardTemplate {
    pub user: User,
    pub recipes: Vec<Recipe>,
    pub active_timers: Vec<Timer>,
}
```

### Type Safety Across Boundaries
```rust
// Handler function with full type safety
pub async fn dashboard_handler(
    State(app_state): State<AppState>,
    session: UserSession,
) -> Result<Html<String>, AppError> {
    let template = DashboardTemplate {
        user: session.user,
        recipes: recipe_service.get_user_recipes(&session.user.id).await?,
        active_timers: timer_service.get_active_timers(&session.user.id).await?,
    };
    
    Ok(Html(template.render()?))
}
```

### Performance Optimizations
- **Template Caching**: Compiled templates cached at build time
- **Static Asset Optimization**: Integrated with Tailwind CSS for minimal CSS bundles
- **Database Connection Pooling**: SQLx connection pooling for optimal database performance
- **Async Request Handling**: Full async/await support for non-blocking I/O

### Development Workflow Integration
- **cargo-watch**: Auto-recompilation and restart during development
- **Template Hot Reload**: Development mode template reloading (planned)
- **IDE Integration**: rust-analyzer provides excellent template syntax support

## References
- [Askama Template Engine Documentation](https://docs.rs/askama/)
- [Axum Web Framework](https://docs.rs/axum/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Kitchen Environment Performance Requirements](../requirements/performance.md)