## ADDED Requirements

### Requirement: Session cost computed from token usage
The system SHALL compute session cost in USD from accumulated `input_tokens` and `output_tokens` using a model-specific pricing table. The cost SHALL be exposed as `session_cost: f64` in `SessionView`.

#### Scenario: Cost accumulates across turns
- **WHEN** a turn completes with `UsageUpdated { input_tokens: 1000, output_tokens: 500 }`
- **THEN** `SessionView::session_cost` reflects the cost for 1000 input + 500 output tokens at the current model's rate

#### Scenario: Multiple turns accumulate
- **WHEN** two turns complete with costs of $0.01 and $0.02
- **THEN** `SessionView::session_cost` is $0.03

### Requirement: Session duration tracked from first query
The system SHALL track wall-clock duration from the first `QueryStarted` event to the current time. The duration SHALL be exposed as `session_duration: Duration` in `SessionView`.

#### Scenario: Duration starts on first query
- **WHEN** the first `QueryStarted` event is applied
- **THEN** `SessionView::session_duration` begins counting from that instant

#### Scenario: Duration continues after query completes
- **WHEN** a query completes and the session is idle
- **THEN** `session_duration` continues to reflect elapsed time since the first query

### Requirement: Pricing table supports model lookup
The system SHALL maintain a pricing table mapping model names to `PricingTier { input_per_million: f64, output_per_million: f64 }`. Unknown models SHALL use a default pricing tier.

#### Scenario: Known model pricing
- **WHEN** the model name is "claude-sonnet-4-6"
- **THEN** the pricing table returns the correct per-million-token rates

#### Scenario: Unknown model fallback
- **WHEN** the model name is not in the pricing table
- **THEN** a default pricing tier is used with a warning-level log
