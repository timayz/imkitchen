# Security Policy

## Current Security Status

This document tracks known security vulnerabilities and our mitigation strategies.

### Active Security Vulnerabilities

#### RUSTSEC-2023-0071: RSA Crate Marvin Attack (Medium Severity)
- **Crate**: rsa v0.9.8
- **Dependency Chain**: sqlx -> sqlx-mysql -> rsa
- **Impact**: Potential key recovery through timing sidechannels
- **CVSS Score**: 5.9 (Medium)
- **Status**: No fix available from upstream
- **Mitigation**: 
  - Currently used only for MySQL connections (not primary database)
  - SQLite is our primary database (unaffected)
  - Monitor sqlx updates for rsa crate replacement
- **Timeline**: Monitor weekly for updates

#### RUSTSEC-2024-0437: Protobuf Uncontrolled Recursion (High Priority)
- **Crate**: protobuf v2.28.0  
- **Dependency Chain**: prometheus -> protobuf
- **Impact**: DoS through stack overflow on malicious input
- **Status**: Fix available (upgrade to protobuf >=3.7.2)
- **Mitigation**: 
  - Prometheus metrics endpoint should be internal-only
  - Input validation on metrics collection
- **Action Required**: Update prometheus crate or find alternative
- **Timeline**: Address within 2 weeks

### Security Scanning

We use `cargo-audit` for dependency vulnerability scanning:

```bash
# Run security audit
cargo audit

# Run with configuration
cargo audit --config audit.toml

# Generate report
cargo audit --format json > security-report.json
```

### Development Security Practices

1. **Dependency Management**
   - Regular dependency updates
   - Minimize external dependencies  
   - Use well-maintained crates with good security track records

2. **Code Review**
   - All changes require review
   - Security-focused review for authentication/authorization code
   - Validate input handling and data sanitization

3. **Configuration Security**
   - Session secrets must be >= 32 characters
   - Force HTTPS in production
   - Rate limiting enabled
   - Trusted proxy configuration

4. **Database Security**
   - SQLite database with restricted file permissions
   - No raw SQL query construction
   - Use SQLx for compile-time checked queries

### Reporting Security Issues

If you discover a security vulnerability, please report it to:
- Email: security@imkitchen.dev (if exists)
- GitHub Security Advisory (preferred)

Please include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact assessment
- Suggested mitigation if known

### Security Update Process

1. **Assessment**: Evaluate severity and impact
2. **Mitigation**: Implement temporary workarounds if needed
3. **Fix Development**: Develop and test security fixes
4. **Testing**: Thorough security testing of fixes
5. **Deployment**: Coordinated deployment of security updates
6. **Disclosure**: Responsible disclosure after fixes are deployed

### Security Tools and Automation

- **cargo-audit**: Dependency vulnerability scanning
- **cargo-clippy**: Code quality and security linting  
- **cargo-deny**: License and dependency policy enforcement
- **Integration Tests**: Security configuration validation

### Security Configuration Checklist

- [ ] SESSION_SECRET environment variable set (>= 32 chars)
- [ ] HTTPS enabled in production (force_https = true)
- [ ] Rate limiting configured appropriately
- [ ] Database file permissions restricted
- [ ] Log files secured and rotated
- [ ] Metrics endpoint access controlled
- [ ] Input validation on all user inputs
- [ ] CORS configured restrictively
- [ ] Security headers enabled

### Incident Response

In case of a security incident:

1. **Immediate Response**
   - Isolate affected systems
   - Preserve evidence
   - Assess scope and impact

2. **Investigation**
   - Root cause analysis
   - Timeline reconstruction
   - Data impact assessment

3. **Remediation**
   - Apply fixes
   - Verify security
   - Monitor for recurrence

4. **Communication**
   - Internal stakeholder notification
   - User communication if needed
   - Regulatory reporting if required

5. **Post-Incident**
   - Lessons learned documentation
   - Process improvements
   - Additional security measures

---

**Last Updated**: 2025-09-27  
**Next Review**: 2025-10-27  
**Responsible Team**: IMKitchen Development Team