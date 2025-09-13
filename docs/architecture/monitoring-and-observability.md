# Monitoring and Observability

## Monitoring Stack

- **Frontend Monitoring:** Real User Monitoring (RUM) with performance metrics, error tracking with Sentry
- **Backend Monitoring:** Prometheus metrics collection, Grafana dashboards for visualization  
- **Error Tracking:** Structured logging with tracing crate, error aggregation and alerting
- **Performance Monitoring:** APM with request tracing, database query performance analysis

## Key Metrics

**Frontend Metrics:**
- Core Web Vitals (LCP, FID, CLS)
- JavaScript errors and stack traces  
- API response times from client perspective
- User interaction funnel metrics (recipe import → cook → rate)

**Backend Metrics:**
- Request rate, error rate, response time (RED metrics)
- Recipe parsing success rate and accuracy
- Timing prediction accuracy vs actual cooking times
- Database query performance and slow query detection
