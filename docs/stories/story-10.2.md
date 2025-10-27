# Story 10.2: Performance Testing and Optimization

Status: Approved

## Story

As a performance engineer,
I want to validate system performance benchmarks through comprehensive load testing,
so that the enhanced meal planning system meets performance targets under realistic concurrent load before production deployment.

## Acceptance Criteria

1. **AC1**: Load test with 100 concurrent multi-week generation requests
   - Verify: k6 script `e2e/performance/load-test.js` configured with 100 virtual users
   - Verify: Test runs for 5 minutes sustained load with 30-second ramp-up

2. **AC2**: P95 generation time <5 seconds
   - Verify: k6 output shows `http_req_duration{route=generate-multi-week} P95 < 5000ms`
   - Verify: Benchmark meets PRD NFR-1 requirement

3. **AC3**: P95 route response time <500ms
   - Verify: k6 output shows `http_req_duration{route=plan-calendar} P95 < 500ms`
   - Verify: k6 output shows `http_req_duration{route=shopping-list} P95 < 500ms`

4. **AC4**: Database query performance profiled (no N+1 queries)
   - Verify: SQL query logs analyzed (SQLx tracing output)
   - Verify: Zero N+1 query patterns detected (ingredient loading, recipe fetching)
   - Verify: `EXPLAIN QUERY PLAN` output shows index usage for all queries

5. **AC5**: Memory usage profiled (no leaks, bounded growth)
   - Verify: Heap profile generated via `cargo-flamegraph` or valgrind
   - Verify: Memory growth <100MB per 1000 requests (linear, bounded)
   - Verify: No leaks reported by valgrind (if used for C dependencies)

6. **AC6**: Performance regression tests added to CI
   - Verify: GitHub Actions workflow includes performance benchmarking step
   - Verify: Baseline latencies stored (JSON file in repo)
   - Verify: CI fails if P95 latency exceeds baseline by >20%

7. **AC7**: Optimization recommendations documented (if targets not met)
   - Verify: `docs/performance-report.md` exists if benchmarks fail
   - Verify: Document includes: bottleneck analysis, optimization suggestions, action items

## Tasks / Subtasks

- [ ] Task 1: Setup k6 load testing infrastructure (AC: #1)
  - [ ] Subtask 1.1: Install k6 (binary download or Docker: `grafana/k6`)
  - [ ] Subtask 1.2: Create `e2e/performance/load-test.js` with virtual user configuration (100 concurrent)
  - [ ] Subtask 1.3: Configure test stages: 30s ramp-up (0→100 users), 5min sustained (100 users), 30s ramp-down
  - [ ] Subtask 1.4: Create test data generator: seed 100 test users with 50 favorite recipes each
  - [ ] Subtask 1.5: Implement authentication in k6 script (JWT cookie handling)

- [ ] Task 2: Implement load test scenarios (AC: #1, #2, #3)
  - [ ] Subtask 2.1: Scenario 1: Multi-week meal plan generation (`POST /plan/generate-multi-week`)
  - [ ] Subtask 2.2: Scenario 2: Week navigation (`GET /plan?week=YYYY-MM-DD`)
  - [ ] Subtask 2.3: Scenario 3: Single week regeneration (`POST /plan/regenerate-week?week=YYYY-MM-DD`)
  - [ ] Subtask 2.4: Scenario 4: Shopping list access (`GET /shopping?week=YYYY-MM-DD`)
  - [ ] Subtask 2.5: Configure k6 thresholds: P95 generation <5s, P95 routes <500ms, error rate <1%
  - [ ] Subtask 2.6: Run load test and capture metrics (JSON output for trend analysis)

- [ ] Task 3: Database query profiling (AC: #4)
  - [ ] Subtask 3.1: Enable SQLx query logging (RUST_LOG=sqlx=debug or trace)
  - [ ] Subtask 3.2: Analyze query logs during load test for N+1 query patterns
  - [ ] Subtask 3.3: Run `EXPLAIN QUERY PLAN` for all meal planning queries (verify index usage)
  - [ ] Subtask 3.4: Profile database connection pool usage (WAL mode, busy timeout effectiveness)
  - [ ] Subtask 3.5: Document any missing indexes or optimization opportunities

- [ ] Task 4: Memory profiling (AC: #5)
  - [ ] Subtask 4.1: Generate heap profile using `cargo-flamegraph` during load test
  - [ ] Subtask 4.2: Analyze memory growth pattern (verify bounded, linear growth <100MB/1000 requests)
  - [ ] Subtask 4.3: Run valgrind on C dependencies (if applicable) to detect memory leaks
  - [ ] Subtask 4.4: Profile evento aggregate loading/unloading (verify proper cleanup)
  - [ ] Subtask 4.5: Document memory usage baseline and acceptable growth bounds

- [ ] Task 5: Performance regression testing in CI (AC: #6)
  - [ ] Subtask 5.1: Create baseline metrics file: `e2e/performance/baseline.json` (P95 latencies for all routes)
  - [ ] Subtask 5.2: Update `.github/workflows/performance.yml` with k6 benchmark step
  - [ ] Subtask 5.3: Implement comparison script: compare current P95 vs baseline
  - [ ] Subtask 5.4: Configure failure threshold: warn if >10% slower, fail if >20% slower
  - [ ] Subtask 5.5: Enable manual trigger for performance tests (not on every commit)

- [ ] Task 6: Optimization (if benchmarks not met) (AC: #7)
  - [ ] Subtask 6.1: Identify bottlenecks from profiling data (algorithm, database, network)
  - [ ] Subtask 6.2: Optimize critical paths (e.g., batch database queries, cache frequently accessed data)
  - [ ] Subtask 6.3: Re-run load tests to validate improvements
  - [ ] Subtask 6.4: Document optimization recommendations in `docs/performance-report.md`
  - [ ] Subtask 6.5: Create backlog items for deferred optimizations (non-critical)

## Dev Notes

### Architecture Patterns and Constraints

**Performance Requirements** (PRD NFR-1):
- Multi-week meal plan generation: P95 <5 seconds (100 concurrent users)
- Route response times: P95 <500ms (GET requests)
- Error rate under load: <1%
- Memory usage: Bounded growth, no leaks

**SQLite Performance Configuration**:
- **WAL Mode**: Enables concurrent reads during writes (critical for load testing)
- **Connection Pools**: Write pool (1 connection), Read pool (5 connections)
- **PRAGMAs**: `journal_mode=WAL, cache_size=-20000, synchronous=NORMAL, busy_timeout=5000`
- **Busy Timeout**: 5000ms (5 seconds) prevents SQLITE_BUSY errors under load

**Load Testing Strategy**:
- **Tool**: k6 (chosen over JMeter for HTTP/2 support, better Rust ecosystem integration)
- **Virtual Users**: 100 concurrent (represents 20% of estimated peak load for 10K MAU)
- **Test Duration**: 5 minutes sustained load (validates system stability)
- **Ramp-up**: 30 seconds (gradual load increase prevents artificial spikes)
- **Test Data**: Realistic dataset (50 favorite recipes per user, matching expected user behavior)

**Profiling Tools**:
- **cargo-flamegraph**: Heap profiling for memory usage analysis
- **valgrind**: Memory leak detection for C dependencies (SQLite, OpenSSL)
- **SQLx tracing**: Query logging for N+1 query detection
- **OpenTelemetry**: Distributed tracing for latency analysis

### Project Structure Notes

**Performance Testing Directory Structure**:
```
e2e/performance/
├── load-test.js                  # k6 script with 100 virtual users
├── baseline.json                 # Baseline P95 latencies for regression detection
├── test-data-generator.js        # Seed 100 users with 50 recipes each
└── results/
    ├── load-test-YYYY-MM-DD.json # k6 metrics output
    └── flamegraph.svg            # Heap profile visualization
```

**CI/CD Integration**:
- GitHub Actions workflow: `.github/workflows/performance.yml`
- Manual trigger only (not on every commit to save CI resources)
- Comparison script: `scripts/compare-performance.sh` (baseline vs current)

**Alignment with Unified Project Structure**:
- Performance tests isolated in `e2e/performance/` (separate from unit/integration tests)
- Profiling outputs stored in `.gitignore`d `results/` directory
- Baseline metrics versioned in Git for regression tracking

### Testing Standards Summary

**k6 Load Testing Best Practices**:
1. **Realistic Data**: Use production-like dataset (50 recipes per user, not 5 or 500)
2. **Gradual Ramp-up**: 30-second ramp-up prevents artificial load spikes
3. **Sustained Load**: 5-minute duration validates stability (not just burst capacity)
4. **Thresholds**: Fail test if P95 >5s (generation) or >500ms (routes)
5. **Metrics Export**: JSON output for trend analysis over time

**Profiling Best Practices**:
1. **Query Logging**: Enable SQLx debug/trace logging during load test (not in production)
2. **Heap Profiling**: Run cargo-flamegraph with release build (not debug for accurate results)
3. **Baseline Comparison**: Always compare against baseline (absolute numbers misleading)
4. **Reproducibility**: Use deterministic test data (fixed random seed)

**Performance Regression CI Requirements**:
- Warn if P95 latency increases by 10-20% (acceptable variance)
- Fail if P95 latency increases by >20% (unacceptable regression)
- Manual trigger for performance tests (not on every PR to save resources)
- Store baseline metrics in Git (versioned, auditable)

### References

**Tech Spec**: [Source: /home/snapiz/projects/github/timayz/imkitchen/docs/tech-spec-epic-10.md]
- Section "Non-Functional Requirements - Performance" (lines 521-542): Performance benchmarks table
- Section "Database Performance Constraints" (lines 106-115): Connection pool configuration, PRAGMA optimizations
- Section "Workflow 2: Performance Testing Flow" (lines 430-473): k6 load testing workflow
- Section "Story 10.2: Performance Testing and Optimization" (lines 726-758): Authoritative acceptance criteria

**Epics Document**: [Source: /home/snapiz/projects/github/timayz/imkitchen/docs/epics.md#L2315-2336]
- Epic 10, Story 10.2: User story statement, prerequisites, technical notes

**Architecture Dependencies**:
- [Source: docs/solution-architecture.md]: Event-sourced monolith architecture, SQLite optimization strategies
- [Source: docs/database-schema.md]: SQLite schema, indexes, query patterns

**Related Performance Requirements**:
- [Source: docs/PRD.md]: NFR-1 (performance targets), NFR-4 (scalability to 10K users)
- PRD Performance Requirements: Generation <5s P95, Routes <500ms P95, 100 concurrent users

**Related Implementation Epics**:
- Epic 6: Multi-week meal plan generation (primary performance bottleneck)
- Epic 7: Week-specific regeneration (performance validation required)
- Epic 8: Meal planning preferences (database query optimization)
- Epic 9: Shopping list generation (potential N+1 query risk)

## Dev Agent Record

### Context Reference

- Story Context: `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-10.2.xml`

### Agent Model Used

<!-- To be filled by dev agent -->

### Debug Log References

<!-- To be filled by dev agent during implementation -->

### Completion Notes List

<!-- To be filled by dev agent during implementation -->

### File List

<!-- Files created/modified during implementation -->
