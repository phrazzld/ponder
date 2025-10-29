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

- [~] Add Converse subcommand to src/cli/mod.rs
  ```
  Files: src/cli/mod.rs
  Add: Converse variant to PonderCommand enum
  Args: ConverseArgs struct (minimal - no flags for MVP)
  Pattern: Follow Ask/Reflect subcommand patterns (lines 64, 67)
  Success criteria: `ponder converse` parsed correctly, no arguments needed for MVP
  ~15 lines
  ```

- [~] Implement cmd_converse handler in src/main.rs
  ```
  Files: src/main.rs
  Function: cmd_converse(config) -> AppResult<()>
  Logic:
    1. Initialize SessionManager (prompt for passphrase)
    2. Open Database with passphrase
    3. Create OllamaClient from config
    4. Call ops::converse::start_conversation()
    5. Handle graceful shutdown on Ctrl+C
  Success criteria: Orchestrates deps correctly, handles session lifecycle
  Pattern: Follow cmd_ask pattern (similar orchestration)
  Error handling: Print user-friendly messages for Ollama connection, session timeout
  ~60 lines
  ```

- [~] Update ops/mod.rs to export converse module
  ```
  Files: src/ops/mod.rs
  Add: pub mod converse; and pub use converse::start_conversation;
  ~2 lines
  ```

### Testing

- [ ] Add unit tests for conversation context assembly
  ```
  Files: src/ops/converse.rs (test module)
  Tests:
    - test_assemble_context_no_entries: Empty DB returns empty context
    - test_assemble_context_finds_relevant: Vector search returns matching entries
    - test_assemble_context_decrypts: Encrypted entries properly decrypted
    - test_assemble_context_limits_results: Respects limit parameter
  Pattern: Use tempfile + in-memory DB like existing ops tests
  ~120 lines
  ```

- [ ] Add integration test for conversation loop
  ```
  Files: tests/ops_integration_tests.rs
  Test: test_conversation_interactive_loop
  Setup:
    - Create temp journal with 3-5 entries
    - Mock input stream with 2-3 questions
  Verify:
    - Context assembled from relevant entries
    - Responses stream correctly
    - Message history maintained across turns
    - Exits gracefully on "quit"
  Requires: Ollama running locally (can be ignored in CI)
  ~80 lines
  ```

### Documentation

- [ ] Update CLAUDE.md with conversational interface examples
  ```
  Files: CLAUDE.md
  Section: "Conversational Operations (v2.1)"
  Add:
    - High-level flow explanation
    - Example conversation interaction
    - Explanation of how it extends existing RAG
    - Note about native Ollama streaming
  Success criteria: Future developers understand design without reading code
  ~40 lines
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
3. ✅ AI cites specific journal entries in responses
4. ✅ Conversation maintains context across multiple turns
5. ✅ Responses stream in real-time (no waiting for full response)
6. ✅ Works with existing journal entries (no migration needed)
7. ✅ No new database tables required
8. ✅ Graceful error handling (Ollama down, session timeout)

---

## Next Steps After Completion

See `BACKLOG.md` for future enhancements:
- Persistent insights storage
- Automatic insight accumulation
- Multi-session context tracking
- Advanced context windowing strategies
- Insight review and curation features
