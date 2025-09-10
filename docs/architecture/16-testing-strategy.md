# 16. Testing Strategy

## Testing Pyramid
- **Unit Tests**: 80% coverage target
- **Integration Tests**: API endpoint and database testing
- **End-to-End Tests**: Critical user journey automation
- **Performance Tests**: Load testing for meal plan generation (<2s)
- **Security Tests**: Vulnerability scanning and penetration testing

## Test Technologies
- **Frontend**: Jest, React Native Testing Library, Detox (E2E)
- **Backend**: Go testing package, testify, dockertest
- **Database**: Test containers for isolation
- **API**: Postman/Newman for API testing
- **Performance**: K6 for load testing
