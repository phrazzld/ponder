# TODO: AI Enhancements - Progressive Summaries, Patterns, and Conversational Mode

## Context
- **Approach**: Extend existing v2.0 architecture with three independent feature tracks
- **Key Files**: src/config/mod.rs, src/db/schema.rs, src/cli/mod.rs, src/ai/
- **Patterns**: Follow existing ops/ pattern (ask.rs, reflect.rs, search.rs)
- **Module Philosophy**: Deep modules with minimal interface complexity

## Phase 1: Foundation (Infrastructure)

### Configuration System

- [~] Add AIModels struct to src/config/mod.rs with model fields
  ```
  Files: src/config/mod.rs:23-87
  Approach: Extend Config struct, follow session_timeout pattern
  Success: AIModels struct with 4 model fields, Default trait implemented
  Test: Unit test in config::tests, env var loading validation
  Module: Single config module, no leakage of env var details
  Time: 30min
  ```

- [ ] Implement env var loading for AI model configuration
  ```
  Files: src/config/mod.rs:217-273, src/constants.rs:129-147
  Approach: Follow OLLAMA_URL pattern (line 261-262), add to Config::load()
  Success: All 4 model env vars load with graceful defaults
  Test: Test env var precedence, defaults, invalid values
  Module: Hides env var complexity, exposes clean config API
  Time: 30min
  ```

- [ ] Add model name constants to src/constants.rs
  ```
  Files: src/constants.rs:147+
  Approach: Follow DEFAULT_CHAT_MODEL pattern (line 142-146)
  Success: 4 new constants for reasoning/summary models
  Test: Constants referenced in tests, documentation clear
  Module: Centralized constants, single source of truth
  Time: 15min
  ```

### Database Schema

- [ ] Create summaries table migration in src/db/schema.rs
  ```
  Files: src/db/schema.rs:36-224
  Approach: Follow backup_log table pattern (lines 132-147)
  Success: summaries table with indexes, CHECK constraints
  Test: Test in schema::tests, verify constraints, indexes
  Module: Clean DDL, no coupling to other tables except FK
  Time: 45min
  ```

- [ ] Create patterns table migration in src/db/schema.rs
  ```
  Files: src/db/schema.rs:36-224
  Approach: Follow insights table pattern (lines 96-112)
  Success: patterns table with type enum, JSON metadata
  Test: Test type constraint, singleton state if needed
  Module: Self-contained pattern storage, encrypted at rest
  Time: 45min
  ```

- [ ] Create src/db/summaries.rs for summary DB operations
  ```
  Files: NEW src/db/summaries.rs
  Approach: Follow src/db/entries.rs structure (CRUD operations)
  Success: upsert_summary, get_summary, list_summaries functions
  Test: Unit tests with in-memory DB, test encryption
  Module: Deep module - hides SQL, exposes domain objects
  Time: 1hr
  ```

- [ ] Create src/db/patterns.rs for pattern DB operations
  ```
  Files: NEW src/db/patterns.rs
  Approach: Follow src/db/summaries.rs pattern
  Success: insert_pattern, get_patterns, update_pattern functions
  Test: Unit tests for all CRUD, test JSON serialization
  Module: Single responsibility - pattern persistence only
  Time: 1hr
  ```

### AI Client Enhancements

- [ ] Add model selection parameter to OllamaClient::chat
  ```
  Files: src/ai/ollama.rs:285-321
  Approach: Add optional model param, default to DEFAULT_CHAT_MODEL
  Success: Backwards compatible, new chat_with_model() method
  Test: Test with different models, test fallback behavior
  Module: Maintains existing interface, extends cleanly
  Time: 30min
  ```

- [ ] Add sentiment analysis method to OllamaClient
  ```
  Files: src/ai/ollama.rs:322+
  Approach: Use chat() internally with SENTIMENT_PROMPT
  Success: analyze_sentiment() returns f32 in [-1.0, 1.0]
  Test: Test with positive/negative/neutral text samples
  Module: Hides prompt engineering, exposes simple API
  Time: 45min
  ```

- [ ] Add topic extraction method to OllamaClient
  ```
  Files: src/ai/ollama.rs:322+
  Approach: Use chat() with TOPIC_EXTRACTION_PROMPT, parse JSON
  Success: extract_topics() returns Vec<String>
  Test: Test JSON parsing errors, empty results, valid extraction
  Module: Encapsulates LLM interaction, returns domain types
  Time: 45min
  ```

### Prompt Engineering

- [ ] Add summary prompts to src/ai/prompts.rs
  ```
  Files: src/ai/prompts.rs:115+
  Approach: Follow reflect_prompt pattern (lines 42-64)
  Success: SUMMARY_DAILY, SUMMARY_WEEKLY, SUMMARY_MONTHLY prompts
  Test: Test prompt structure includes entry content
  Module: Centralized prompt library, consistent format
  Time: 30min
  ```

- [ ] Add analysis prompts to src/ai/prompts.rs
  ```
  Files: src/ai/prompts.rs:115+
  Approach: Follow ask_prompt pattern (lines 79-114)
  Success: SENTIMENT_ANALYSIS, TOPIC_EXTRACTION prompts
  Test: Test prompts produce parseable output
  Module: Isolated prompt engineering from business logic
  Time: 30min
  ```

## Phase 2: Progressive Summaries

### Core Summarization Logic

- [ ] Create src/ops/summarize.rs with daily summary function
  ```
  Files: NEW src/ops/summarize.rs
  Approach: Follow src/ops/ask.rs RAG pipeline (lines 46-52)
  Success: generate_daily_summary() encrypts and stores result
  Test: Integration test with temp DB, verify encryption
  Module: Deep - orchestrates crypto, DB, AI without leaking details
  Time: 1.5hr
  ```

- [ ] Implement weekly summary aggregation in summarize.rs
  ```
  Files: src/ops/summarize.rs
  Approach: Fetch 7 daily summaries, aggregate with AI
  Success: generate_weekly_summary() builds from dailies
  Test: Test with mock daily summaries, verify aggregation
  Module: Hides complexity of multi-entry aggregation
  Time: 1hr
  ```

- [ ] Implement monthly summary aggregation in summarize.rs
  ```
  Files: src/ops/summarize.rs
  Approach: Follow weekly pattern, aggregate weekly summaries
  Success: generate_monthly_summary() with topic/sentiment
  Test: Test full hierarchy (monthly from weeklies from dailies)
  Module: Consistent interface across all summary levels
  Time: 1hr
  ```

### CLI Integration

- [ ] Add Summarize subcommand to src/cli/mod.rs
  ```
  Files: src/cli/mod.rs:58-89
  Approach: Follow Reflect pattern (lines 67, 127-133)
  Success: SummarizeArgs with period enum, date option
  Test: Test CLI parsing for all period types, date validation
  Module: Declarative clap args, no business logic
  Time: 45min
  ```

- [ ] Add Summaries subcommand for viewing past summaries
  ```
  Files: src/cli/mod.rs:58-89
  Approach: Follow Search pattern (lines 70, 135-152)
  Success: SummariesArgs with list/show modes
  Test: Test subcommand parsing, option combinations
  Module: Clean CLI interface, delegates to ops layer
  Time: 30min
  ```

- [ ] Implement cmd_summarize handler in src/main.rs
  ```
  Files: src/main.rs
  Approach: Follow cmd_reflect pattern (similar to ask/reflect)
  Success: Dispatches to ops::summarize, handles all periods
  Test: Integration test via CLI, verify output format
  Module: Thin handler - initializes deps, calls ops
  Time: 1hr
  ```

- [ ] Implement cmd_summaries handler for browsing summaries
  ```
  Files: src/main.rs
  Approach: Follow cmd_search pattern
  Success: Lists or shows summaries, decrypts on demand
  Test: Test list vs show modes, pagination if needed
  Module: Presentation layer only, ops handles logic
  Time: 1hr
  ```

## Phase 3: Pattern Detection

### Pattern Analysis Logic

- [ ] Create src/ops/patterns.rs with temporal pattern detection
  ```
  Files: NEW src/ops/patterns.rs
  Approach: Query entries DB for time-based stats, use AI for insights
  Success: detect_temporal_patterns() finds writing time patterns
  Test: Test with synthetic entry metadata across time
  Module: Self-contained analysis, no side effects
  Time: 2hr
  ```

- [ ] Implement topic clustering in patterns.rs
  ```
  Files: src/ops/patterns.rs
  Approach: Cluster embeddings by cosine similarity, label with AI
  Success: detect_topic_patterns() groups similar entries
  Test: Test clustering algorithm, verify topics extracted
  Module: Hides ML complexity, returns domain objects
  Time: 2hr
  ```

- [ ] Implement sentiment trend analysis in patterns.rs
  ```
  Files: src/ops/patterns.rs
  Approach: Iterate entries, analyze_sentiment(), track trends
  Success: detect_sentiment_patterns() with time series data
  Test: Test trend calculation, statistical significance
  Module: Encapsulates sentiment analysis pipeline
  Time: 1.5hr
  ```

- [ ] Implement correlation discovery in patterns.rs
  ```
  Files: src/ops/patterns.rs
  Approach: Find co-occurring topics, temporal correlations
  Success: detect_correlations() identifies meaningful relationships
  Test: Test correlation algorithm with known patterns
  Module: Statistical analysis hidden, clean pattern output
  Time: 2hr
  ```

### CLI Integration

- [ ] Add Analyze subcommand to src/cli/mod.rs
  ```
  Files: src/cli/mod.rs:58-89
  Approach: Follow Ask subcommand pattern (lines 64, 112-125)
  Success: AnalyzeArgs with pattern type flags
  Test: Test flag combinations, mutual exclusivity if needed
  Module: Declarative CLI, minimal validation
  Time: 30min
  ```

- [ ] Add Patterns subcommand for viewing patterns
  ```
  Files: src/cli/mod.rs:58-89
  Approach: Similar to Summaries subcommand
  Success: PatternsArgs with type filter, limit
  Test: Test filtering, pagination, output format
  Module: Read-only pattern browsing interface
  Time: 30min
  ```

- [ ] Add Insights subcommand for AI narrative
  ```
  Files: src/cli/mod.rs:58-89
  Approach: Follow Reflect pattern (simple subcommand)
  Success: InsightsArgs with optional time window
  Test: Test with/without date range
  Module: Single-purpose insight generation trigger
  Time: 20min
  ```

- [ ] Implement cmd_analyze handler in src/main.rs
  ```
  Files: src/main.rs
  Approach: Follow cmd_ask pattern
  Success: Runs pattern detection, stores results, displays
  Test: Integration test for each pattern type
  Module: Orchestrates session, DB, AI client setup
  Time: 1hr
  ```

- [ ] Implement cmd_patterns and cmd_insights handlers
  ```
  Files: src/main.rs
  Approach: Follow cmd_summaries pattern
  Success: Lists patterns, generates AI narrative from patterns
  Test: Test pattern display, insights generation
  Module: Thin handlers delegating to ops layer
  Time: 1hr
  ```

## Phase 4: Conversational Mode (Advanced)

### Conversation Management

- [ ] Create src/ai/conversation.rs with context tracking
  ```
  Files: NEW src/ai/conversation.rs
  Approach: Maintain Vec<Message>, reference recent entries
  Success: ConversationContext with add_message, build_prompt
  Test: Test message accumulation, context window limits
  Module: Stateful conversation manager, encapsulates history
  Time: 1.5hr
  ```

- [ ] Implement question generation based on patterns
  ```
  Files: src/ai/conversation.rs
  Approach: Use pattern data to generate contextual questions
  Success: generate_question() uses patterns, time of day, gaps
  Test: Test question variety, pattern integration
  Module: Hides prompt engineering complexity
  Time: 1.5hr
  ```

- [ ] Implement follow-up question logic
  ```
  Files: src/ai/conversation.rs
  Approach: Analyze previous response, generate deeper question
  Success: generate_followup() maintains conversation flow
  Test: Test with various user response types
  Module: Encapsulates conversation state machine
  Time: 1hr
  ```

- [ ] Implement conversation synthesis to journal entry
  ```
  Files: src/ai/conversation.rs
  Approach: Use LLM to transform conversation into structured entry
  Success: synthesize_entry() generates markdown from messages
  Test: Test with multi-turn conversations, verify quality
  Module: Clean transformation, no conversation details leak
  Time: 1.5hr
  ```

### Conversational Operations

- [ ] Create src/ops/converse.rs with main conversation loop
  ```
  Files: NEW src/ops/converse.rs
  Approach: Follow src/ops/edit.rs interactive pattern
  Success: start_conversation() runs chat loop, saves entry
  Test: Integration test with mock input/output
  Module: Deep - hides conversation complexity, clean start/end
  Time: 2hr
  ```

- [ ] Implement pattern-aware conversation seeding
  ```
  Files: src/ops/converse.rs
  Approach: Load recent patterns, seed conversation context
  Success: Conversation references past topics, trends
  Test: Test with various pattern states, empty DB
  Module: Integration point - patterns inform conversation
  Time: 1hr
  ```

- [ ] Implement quick mode (fewer questions)
  ```
  Files: src/ops/converse.rs
  Approach: Parameterize question count, shorter prompts
  Success: quick_conversation() with 3-5 questions max
  Test: Test question count limits, output quality
  Module: Behavioral variant, shared core logic
  Time: 45min
  ```

- [ ] Implement topic-focused conversation mode
  ```
  Files: src/ops/converse.rs
  Approach: Filter questions to specific topic area
  Success: focused_conversation(topic) stays on topic
  Test: Test topic adherence, relevance
  Module: Constrained conversation, same interface
  Time: 1hr
  ```

### CLI Integration

- [ ] Add Converse subcommand to src/cli/mod.rs
  ```
  Files: src/cli/mod.rs:58-89
  Approach: Follow Edit pattern (lines 61, 92-110)
  Success: ConverseArgs with quick, topic, continue flags
  Test: Test flag parsing, mutual exclusivity
  Module: Declarative CLI definition
  Time: 30min
  ```

- [ ] Implement cmd_converse handler in src/main.rs
  ```
  Files: src/main.rs
  Approach: Follow cmd_edit pattern (interactive)
  Success: Starts conversation, handles modes, saves result
  Test: End-to-end test with simulated user input
  Module: Thin orchestration, delegates to ops::converse
  Time: 1.5hr
  ```

- [ ] Add review/edit flow before saving conversation
  ```
  Files: src/main.rs cmd_converse handler
  Approach: Use crypto::temp + launch_editor like edit.rs
  Success: User can review/modify synthesized entry before save
  Test: Test edit acceptance, rejection, modification
  Module: Reuses existing temp file + editor infrastructure
  Time: 1hr
  ```

## Design Iteration Checkpoints

### After Phase 2 (Summaries Complete)
- Review summary quality with real journal data
- Evaluate topic extraction accuracy
- Consider chunking strategy for long entries
- Assess encryption overhead

### After Phase 3 (Patterns Complete)
- Review pattern signal-to-noise ratio
- Evaluate statistical significance thresholds
- Consider pattern persistence strategy (cache vs recompute)
- Assess query performance with large DBs

### After Phase 4 (Conversational Complete)
- Review conversation flow naturalness
- Evaluate entry synthesis quality
- Consider streaming response implementation
- Assess user experience with real usage

## Quality Gates

### All Phases Must Meet
- [ ] No regression in existing tests (cargo test passes)
- [ ] All new code has >80% test coverage
- [ ] No new clippy warnings (cargo clippy clean)
- [ ] All encryption/decryption tested with real passphrases
- [ ] Session timeout applies to new features
- [ ] Error messages are actionable and user-friendly
- [ ] Documentation updated (inline and CLAUDE.md)

### Performance Requirements
- [ ] Daily summary generation: <5 seconds
- [ ] Weekly summary generation: <10 seconds
- [ ] Pattern detection (90 days): <15 seconds
- [ ] Conversation response: <3 seconds per message
- [ ] Database queries remain <100ms

### Security Requirements
- [ ] All summaries encrypted at rest (age format)
- [ ] All patterns encrypted at rest
- [ ] No plaintext in temporary files beyond session
- [ ] Passphrase never logged or exposed
- [ ] Session manager clears all new state on lock

## Module Boundary Validation

### RED FLAGS to avoid:
- [ ] **Shallow modules**: If new module interface complexity ≈ implementation, reconsider
- [ ] **Pass-through methods**: Each layer should transform, not just forward
- [ ] **Information leakage**: Implementation details (SQL, prompts) visible at call sites
- [ ] **Temporal coupling**: Task order matters beyond data dependencies

### GREEN LIGHTS:
- [x] Deep modules with simple interfaces (Config, OllamaClient, Database)
- [x] Each abstraction layer changes vocabulary (DB row → Entry → domain object)
- [x] Callers don't break when implementation changes
- [x] Tests focus on behavior, not implementation details

## Next Steps

1. **Review and approve** this plan
2. **Create feature branch**: `git checkout -b feature/ai-enhancements-mvp`
3. **Start Phase 1**: Foundation tasks (config, schema, AI client)
4. **Incremental commits**: Each checkbox gets a commit with tests
5. **Test with real data**: Use actual journal throughout development
6. **Iterate on quality**: Adjust prompts, thresholds based on usage

## Open Questions

1. **Summary encryption**: Store only encrypted or also plaintext hash for indexing?
   - **Decision needed**: Affects search performance vs security posture

2. **Pattern auto-refresh**: Manual trigger or background update?
   - **Recommendation**: Start manual, consider auto after usage patterns emerge

3. **Conversation history**: Ephemeral or persistent across sessions?
   - **Recommendation**: Ephemeral (privacy-first), user can save manually

4. **Model availability**: Hard fail or graceful degradation?
   - **Recommendation**: Graceful degradation with clear user guidance

5. **Streaming responses**: Implement for conversational mode?
   - **Recommendation**: Phase 4 nice-to-have, not MVP blocker
