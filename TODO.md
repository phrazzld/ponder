# TODO: Conversational Journal Assistant (2025 LLM-Native Approach)

## Context & Philosophy

**Original Plan (DEPRECATED)**: Build sentiment classifiers, topic extractors, statistical pattern detection
**Ultrathink Verdict**: Overengineered. We're treating 2025 LLMs like 2016 ML models.

**New Approach**: Conversational interface with Chain-of-Thought reasoning
- Let LLMs reason naturally about journal content
- Extend existing RAG pipeline (ops/ask.rs) rather than rebuild
- Use native Ollama streaming (no wrapper modules needed)
- Store insights as entries in existing embeddings table (no new tables)

**Key Insight**: We already have 80% of what we need. Just add conversation loop.

---

## Completed Work (Keep & Build Upon)

✅ **Phase 1: Foundation**
- AIModels config with env var loading (src/config/mod.rs)
- Model name constants (src/constants.rs)
- Database schema for summaries and patterns (src/db/schema.rs)
- CRUD operations (src/db/summaries.rs, src/db/patterns.rs)
- AI client enhancements (sentiment analysis, topic extraction methods)

✅ **Phase 2: Progressive Summaries**
- Daily/weekly/monthly summary generation (src/ops/summarize.rs)
- Summarize and Summaries CLI subcommands
- Command handlers (cmd_summarize, cmd_summaries)

✅ **Phase 3: Pattern Detection (Partial)**
- Temporal pattern detection (src/ops/patterns.rs)
- Topic clustering via embeddings (src/ops/patterns.rs)

---

## Phase 3: Deprecate 2016-Style Classifiers

**DECISION**: Stop building statistical pattern analyzers. LLMs can reason about patterns naturally when asked conversationally.

### Tasks to SKIP (moved to BACKLOG.md)

~~- [ ] Implement sentiment trend analysis in patterns.rs~~
~~- [ ] Implement correlation discovery in patterns.rs~~
~~- [ ] Add Analyze subcommand for pattern types~~
~~- [ ] Add Patterns subcommand for viewing~~
~~- [ ] Add Insights subcommand for AI narrative~~
~~- [ ] Implement cmd_analyze, cmd_patterns, cmd_insights handlers~~

**Rationale**: These duplicate what conversational interface will do better:
- User: "Do you notice any emotional patterns in my entries?"
- AI: "Let me think through this step-by-step... [CoT reasoning about sentiment trends]"

---

## Phase 4: Conversational Interface (Minimal Implementation)

### Core Conversation Loop

- [x] Add Chain-of-Thought system prompt to src/ai/prompts.rs
  ```
  Files: src/ai/prompts.rs
  Add: COT_SYSTEM_PROMPT constant with step-by-step reasoning instructions
  Format: "Think step-by-step: First... Second... This suggests..."
  Success criteria: Prompt encourages reasoning visibility, cites entries, acknowledges uncertainty
  Pattern: Similar to existing SYSTEM_PROMPT but adds explicit CoT instructions
  ~20 lines
  ```

- [x] Create conversational operation in src/ops/converse.rs
  ```
  Files: NEW src/ops/converse.rs
  Function: start_conversation(db, session, ai_client, config)
  Logic:
    1. Interactive loop reading user input
    2. Assemble context using existing ops::ask::get_relevant_context() pattern
    3. Build CoT prompt with context + user question
    4. Stream response using ai_client.chat() with native streaming
    5. Print chunks as they arrive
    6. Maintain message history in-memory (Vec<Message>) for conversation continuity
    7. Exit on "quit", "exit", or empty input
  Success criteria:
    - User can chat interactively with AI about their journal
    - AI responses cite specific entries
    - AI shows step-by-step reasoning
    - Context assembled from existing RAG pipeline
    - No new database tables needed
  Error handling: Session timeout, Ollama unreachable, decryption failure
  Testing: Integration test with mock input/output, verify context assembly
  ~341 lines (includes helper function and tests)
  Work Log:
  - Implemented start_conversation() with interactive loop
  - Included assemble_conversation_context() helper (next task) in same file
  - Non-streaming for MVP (can add --stream flag later)
  - Conversation history pruned to 20 messages to prevent context overflow
  - Unit tests for context assembly (no entries, limit validation)
  ```

- [x] Add helper function to ops/converse.rs for context assembly
  ```
  Files: src/ops/converse.rs
  Function: assemble_conversation_context(db, session, query, limit) -> AppResult<Vec<(NaiveDate, String)>>
  Note: Completed as part of previous task (included in same file)
  ```

### CLI Integration

- [x] Add Converse subcommand to src/cli/mod.rs
  ```
  Files: src/cli/mod.rs
  Status: COMPLETE - Converse variant exists (line 79), ConverseArgs struct (lines 225-231)
  Work Log:
  - Already implemented with --no-context flag
  - Follows existing subcommand patterns
  ```

- [x] Implement cmd_converse handler in src/main.rs
  ```
  Files: src/main.rs
  Status: COMPLETE - cmd_converse() exists (line 608)
  Work Log:
  - Orchestrates SessionManager → Database → OllamaClient
  - Calls ops::converse::start_conversation()
  - Error handling for Ollama connection, session timeout
  ```

- [x] Update ops/mod.rs to export converse module
  ```
  Files: src/ops/mod.rs
  Status: COMPLETE - Module declared (line 9), re-exported (line 23)
  ```

### Testing

- [x] Add unit tests for conversation context assembly
  ```
  Files: src/ops/converse.rs (test module)
  Tests:
    - test_assemble_context_no_entries: Empty DB returns empty context [DONE]
    - test_assemble_context_finds_relevant: Vector search returns matching entries [DONE]
    - test_assemble_context_decrypts: Encrypted entries properly decrypted [DONE]
    - test_assemble_context_limits_results: Respects limit parameter [DONE]
  Pattern: Use tempfile + in-memory DB like existing ops tests
  Completed: 284 lines (3 tests marked #[ignore = "requires Ollama"])
  Commit: 2eadad2
  ```

- [x] Add integration test for conversation loop
  ```
  Files: tests/ops_integration_tests.rs
  Test: test_conversation_context_assembly_integration
  Setup:
    - Create temp journal with 5 entries (presentation narrative)
    - Generate embeddings via Ollama
  Verify:
    - Context assembled from relevant entries across 5 queries
    - Semantic relevance maintained
    - Limit enforcement works
    - Dates from relevant entries returned
  Requires: Ollama running locally (marked #[ignore])
  Completed: 189 lines
  Commit: 47341aa
  ```

### Documentation

- [x] Update CLAUDE.md with conversational interface examples
  ```
  Files: CLAUDE.md
  Section: "Conversational Operations (v2.1)"
  Added:
    - Philosophy: 2025 LLMs vs 2016 classifiers
    - High-level flow (5-step process)
    - Example conversation with CoT reasoning
    - Extends existing RAG explanation
    - Implementation details (no new tables, no wrappers)
    - Design rationale with ultrathink references
  Also updated module flow diagram with cmd_converse
  Completed: 81 lines
  Commit: c346076
  ```

---

## Implementation Notes

### What We're NOT Building (Per Ultrathink)

❌ **streaming.rs module**: Use Ollama's native `stream=True` parameter directly
❌ **context.rs module**: Context assembly logic lives in converse.rs (~80 lines)
❌ **db/memory.rs module**: No new table needed - use existing embeddings
❌ **conversation_memory table**: Insights stored as regular entries if needed
❌ **Session management**: Use existing 30-min timeout, no new session concept
❌ **Complex context windowing**: Start simple, optimize if needed

### Reusing Existing Infrastructure

✅ **Vector search**: `db::embeddings::search_similar_chunks()` - already deep module
✅ **RAG pipeline**: `ops::ask.rs` patterns for context assembly
✅ **Decryption**: `crypto::age::decrypt_with_passphrase()` - secure and tested
✅ **Streaming**: Ollama client library native support (no wrapper needed)
✅ **Entry storage**: Existing `entries` table + `embeddings` table

### Design Principles Applied

- **Module depth**: Converse.rs is deep (simple interface: `start_conversation()`, complex orchestration inside)
- **Information hiding**: Streaming, context selection, decryption all hidden from CLI layer
- **Minimal new modules**: 1 new file (converse.rs), 0 new tables, 0 wrapper modules
- **Extend don't rebuild**: Reuses 80% of existing RAG infrastructure

---

## Timeline Estimate

- **Phase 4 (Conversational Interface)**: 1 week
  - Core loop: 2 days
  - CLI integration: 1 day
  - Testing: 2 days
  - Documentation: 0.5 days

**Total for minimal conversational interface: ~5 days**

Compare to original plan: 4 weeks → 1 week (80% reduction via simplification)

---

## Success Criteria for Phase 4

When this phase is complete:

1. ✅ Users can run `ponder converse` to start interactive chat
2. ✅ AI responds with step-by-step reasoning (Chain-of-Thought)
3. ✅ AI cites specific journal entries in responses (via context assembly)
4. ✅ Conversation maintains context across multiple turns (in-memory history)
5. ⚠️  Responses stream in real-time → Non-streaming for MVP (can add later)
6. ✅ Works with existing journal entries (no migration needed)
7. ✅ No new database tables required (uses existing embeddings)
8. ✅ Graceful error handling (Ollama down, session timeout, context errors)

---

---

## Phase 5: Intent-Aware Context Retrieval (3-Phase Workflow)

### Context & Rationale

**Problem**: Conversational interface retrieves context for ALL queries, including meta-questions ("what do you think?") that don't need journal context. This wastes time and pollutes responses with irrelevant entries.

**Ultrathink Verdict**: Use LLM-driven reflection phase with structured JSON output (NOT tool calling - that's over-engineered for 2 tools). Three-phase workflow:
1. **Reflection**: LLM decides search vs. respond directly
2. **Conditional Execution**: Fetch context only if needed
3. **Response**: Stream answer with/without context

**Key Design Decision**: Structured JSON → Rust enum, reuse existing TemporalConstraint type. Simpler than tool calling, fewer failure modes, faster.

### Cleanup Work

- [~] Revert tool calling infrastructure from ollama.rs
  ```
  Files: src/ai/ollama.rs
  Remove:
    - ToolDefinition, FunctionDefinition structs (lines 83-97)
    - ChatWithToolsRequest struct (lines 99-106)
    - ToolCall, ToolCallFunction structs (lines 108-119)
    - ChatWithToolsResponse, MessageWithTools structs (lines 121-132)
    - chat_with_tools() method (lines 716-811)
  Reason: Tool calling is over-engineered for binary decision (search vs respond)
  Success criteria: ollama.rs only contains streaming support, no tool definitions
  ~100 lines removed
  ```

### Core Types

- [ ] Create ReflectionDecision enum in ops/converse.rs
  ```
  Files: src/ops/converse.rs (top of file after imports)
  Add:
    #[derive(Debug, Clone, Deserialize)]
    #[serde(tag = "action", rename_all = "lowercase")]
    enum ReflectionDecision {
        Search {
            #[serde(default)]
            temporal_constraint: TemporalConstraint,
            reasoning: String,
        },
        Respond {
            reasoning: String,
        },
    }
  Reason: Type-safe decision from LLM, reuses existing TemporalConstraint
  Success criteria: Deserializes from JSON {"action": "search"|"respond", ...}
  Pattern: Similar to serde tagged enum pattern in query_analysis.rs
  ~15 lines
  ```

- [ ] Add reflection system prompt constant
  ```
  Files: src/ops/converse.rs (after imports, before ReflectionDecision)
  Add:
    const REFLECTION_SYSTEM_PROMPT: &str = r#"Analyze this query and decide how to respond.

    If user is asking ABOUT their journal (activities, events, feelings recorded):
    → Action: "search"
    → Specify temporal_constraint if mentioned:
      - "past week" → {"type": "relative", "days_ago": 7}
      - "last month" → {"type": "relative", "days_ago": 30}
      - no time → {"type": "none"}

    If user is asking FOR YOUR OPINION (meta-questions, advice, your thoughts):
    → Action: "respond"

    Respond with JSON:
    {
      "action": "search" | "respond",
      "temporal_constraint": {...},  // only if action="search"
      "reasoning": "brief explanation"
    }"#;
  Reason: Clear decision criteria, reuses TemporalConstraint schema
  Success criteria: LLM understands distinction, outputs valid JSON
  Pattern: Similar to QUERY_ANALYSIS_PROMPT in query_analysis.rs
  ~25 lines
  ```

### Phase 1: Reflection (Decision Making)

- [ ] Implement reflection phase in conversation loop
  ```
  Files: src/ops/converse.rs (in start_conversation(), after user input, before context assembly)
  Location: Replace lines 113-123 (current context assembly)
  Logic:
    1. Build reflection messages: [system(REFLECTION_SYSTEM_PROMPT), ...last 3 turns, user(input)]
    2. Call ai_client.chat_with_json_format(&reflection_messages)
    3. Parse JSON into ReflectionDecision enum
    4. Handle parse errors gracefully (show user error, continue loop)
    5. Print reasoning to user: "💭 Searching journal: [reasoning]" or "💭 Responding directly: [reasoning]"
  Success criteria:
    - LLM correctly classifies "what did I do yesterday?" as Search
    - LLM correctly classifies "what do you think?" as Respond
    - User sees reasoning for transparency
    - Parse errors shown clearly to user
  Error handling:
    - serde_json parse failure → print user-friendly message, continue loop
    - Ollama unreachable → propagate error (existing handling)
  Testing: Unit test with mock JSON responses for both action types
  ~40 lines (includes error handling and reasoning display)
  ```

### Phase 2: Conditional Execution

- [ ] Implement conditional context assembly
  ```
  Files: src/ops/converse.rs (after reflection phase)
  Location: Replace existing context assembly (lines 115-123)
  Logic:
    match decision {
        ReflectionDecision::Search { temporal_constraint, .. } => {
            // Call updated assemble_conversation_context (next task)
            assemble_conversation_context(db, session, ai_client, user_input, Some(temporal_constraint), 10)?
        },
        ReflectionDecision::Respond { .. } => {
            Vec::new()  // No context needed
        },
    }
  Success criteria:
    - Search action → context assembled with temporal filter
    - Respond action → empty context, no DB query
    - Existing error handling preserved
  Pattern: Simple match expression on enum
  ~15 lines
  ```

- [ ] Update assemble_conversation_context to accept optional temporal constraint
  ```
  Files: src/ops/converse.rs (function signature and implementation)
  Location: Lines 215-280 (existing function)
  Changes:
    1. Add parameter: temporal_constraint: Option<TemporalConstraint>
    2. If Some(constraint), apply date filtering before/after vector search:
       - constraint.to_date_range(today()) → (start, end)
       - Filter entry dates to be within range
    3. If None, search all dates (current behavior)
  Success criteria:
    - Temporal constraint correctly filters entries by date
    - None constraint searches all dates (backward compatible)
    - Existing tests still pass
  Error handling: Invalid date range → log warning, proceed without filter
  Testing: Add unit test with relative/absolute/none constraints
  ~20 lines modified + 30 lines new test
  ```

### Phase 3: Response Generation (Already Implemented)

- [ ] Verify streaming response works with new workflow
  ```
  Files: src/ops/converse.rs (lines 167-210)
  Verification: Existing streaming code (Phase 3) unchanged
  Success criteria:
    - Responses stream word-by-word regardless of context presence
    - Empty context (Respond action) still generates coherent answers
    - Context (Search action) properly integrated into prompt
  Testing: Manual QA with both action types
  Notes: No code changes needed - just verification
  ```

### Testing

- [ ] Unit tests for ReflectionDecision deserialization
  ```
  Files: src/ops/converse.rs (test module)
  Tests:
    - test_reflection_decision_search_with_relative: Valid search JSON with days_ago
    - test_reflection_decision_search_with_none: Valid search JSON without temporal constraint
    - test_reflection_decision_respond: Valid respond JSON
    - test_reflection_decision_invalid_action: Invalid action type errors gracefully
    - test_reflection_decision_missing_reasoning: Missing required field errors
  Success criteria: All valid JSON deserializes, invalid JSON returns clear errors
  Pattern: Similar to test_query_analysis_json_* tests in query_analysis.rs
  ~60 lines (5 tests)
  ```

- [ ] Integration test for meta-question workflow
  ```
  Files: tests/ops_integration_tests.rs
  Test: test_conversation_meta_question_skips_retrieval
  Setup:
    - Create temp journal with entries
    - Generate embeddings
  Test flow:
    1. Query: "what do you think about this?" (meta-question)
    2. Verify: ReflectionDecision::Respond returned
    3. Verify: No DB context query executed (empty context)
    4. Verify: Response generated from conversation history only
  Success criteria:
    - Meta-questions don't trigger DB queries
    - Responses still coherent without context
    - Reasoning displayed to user
  Requires: Ollama running (mark #[ignore])
  ~80 lines
  ```

- [ ] Integration test for temporal-filtered search
  ```
  Files: tests/ops_integration_tests.rs
  Test: test_conversation_search_with_temporal_filter
  Setup:
    - Create entries across 3 months (Jan, Feb, March)
    - Generate embeddings for all
  Test flow:
    1. Query: "what did I do last month?" (assuming today is March)
    2. Verify: ReflectionDecision::Search with relative constraint (30 days)
    3. Verify: Context only includes February entries, excludes Jan/March
    4. Verify: Response cites only February entries
  Success criteria:
    - Temporal filtering works correctly
    - LLM extracts correct temporal constraint
    - Context assembly respects date range
  Requires: Ollama running (mark #[ignore])
  ~100 lines
  ```

### Documentation

- [ ] Update CLAUDE.md with three-phase workflow explanation
  ```
  Files: CLAUDE.md
  Section: Update "Conversational Operations (v2.1)" section
  Add:
    - Subsection: "Intent-Aware Context Retrieval (v2.2)"
    - Three-phase workflow diagram
    - Example: Meta-question vs. factual query handling
    - ReflectionDecision enum explanation
    - Design rationale (why not tool calling)
  Success criteria: Developers understand workflow, design decisions clear
  Pattern: Similar to existing RAG pipeline explanation
  ~50 lines
  ```

- [ ] Add inline documentation for reflection phase
  ```
  Files: src/ops/converse.rs
  Add:
    - Module-level doc comment explaining three phases
    - Doc comments on ReflectionDecision enum variants
    - Doc comment on REFLECTION_SYSTEM_PROMPT explaining criteria
    - Inline comments in reflection logic explaining flow
  Success criteria: Code self-documents workflow, easy to understand
  Pattern: Follow existing ops module doc style
  ~30 lines total
  ```

---

## Next Steps After Completion

See `BACKLOG.md` for future enhancements:
- Persistent insights storage
- Automatic insight accumulation
- Multi-session context tracking
- Advanced context windowing strategies
- Insight review and curation features
