# BACKLOG: Conversational Journal Assistant

Last Updated: 2025-10-31
Context: Minimal conversational interface (Phase 4) implements core functionality. This backlog contains advanced features deferred until core is validated.

---

## Code Quality & Polish (v2.1.1)

### From PR #56 Review Feedback

These items were identified during code review as minor improvements but deemed non-blocking for merge.

#### Extract Conversation Context Pruning Constant
- **Location**: src/ops/converse.rs:327
- **Issue**: Hard-coded magic number `21` for message history limit
- **Fix**: Extract to named constant `MAX_CONVERSATION_HISTORY`
- **Effort**: 5 minutes
- **Value**: Code maintainability
- **Source**: PR #56 review comments

#### Extract Reflection Fallback Helper
- **Location**: src/ops/converse.rs:180-207
- **Issue**: Reflection fallback handling is verbose (28 lines)
- **Fix**: Extract to `get_reflection_decision_with_fallback()` helper function
- **Effort**: 30 minutes
- **Value**: Code clarity and reusability
- **Source**: PR #56 comprehensive review

#### Add Week Boundary Edge Case Tests
- **Location**: src/ops/summarize.rs:354 (week end date calculation)
- **Issue**: Complex date logic lacks edge case coverage
- **Fix**: Add tests for month boundaries (Jan 31 → Feb 1), year boundaries (Dec 29 → Jan 5), leap years
- **Effort**: 1 hour
- **Value**: Robustness and confidence in date arithmetic
- **Source**: PR #56 comprehensive review

#### Add Progress Indicators for Auto-Generation
- **Location**: src/ops/summarize.rs:185-220 (weekly), src/ops/summarize.rs:381-408 (monthly)
- **Issue**: Monthly summary can trigger 30+ LLM calls with no progress indication
- **Fix**: Add progress bar (e.g., "Generating summaries... [5/12] 42%")
- **Considerations**:
  - User confirmation for >10 auto-generations?
  - Batch size limits to prevent runaway generation?
- **Effort**: 2 hours
- **Value**: UX improvement, prevents "frozen" perception
- **Source**: PR #56 review - identified as potential bottleneck

#### Ollama Version Validation at Startup
- **Location**: Startup validation (new function)
- **Issue**: Assumes Ollama >= 0.5 for schema enforcement but silently degrades
- **Fix**: Check Ollama version via API at startup, warn if < 0.5
- **Effort**: 1 hour
- **Value**: Better error messages, prevents silent degradation
- **Source**: PR #56 comprehensive review

#### Parallel Decryption with Rayon
- **Location**: src/ops/ask.rs, src/ops/converse.rs, src/ops/search.rs
- **Issue**: Entries decrypted serially (blocking)
- **Fix**: Use Rayon for parallel decryption when 10+ entries
- **Effort**: 4 hours
- **Value**: 3-5x speedup on multi-core systems for large result sets
- **Trigger**: Defer until journals have 100+ entries or users report slowness
- **Source**: Already in BACKLOG, reinforced by PR #56 review

---

## Deferred from Original Plan

### Statistical Pattern Analysis (Deprecated Approach)

**Why Deferred**: These are 2016-style ML classifiers. Modern LLMs can reason about patterns conversationally without building rigid analyzers.

#### Sentiment Trend Analysis
- **Original Task**: `detect_sentiment_patterns()` function calculating sentiment scores over time
- **Better Approach**: User asks "How has my mood changed this month?" → AI reasons with CoT
- **Effort if needed**: 1.5 hours
- **Priority**: LOW - Conversational interface should handle this naturally

#### Correlation Discovery
- **Original Task**: Statistical correlation between topics, temporal patterns
- **Better Approach**: User asks "Do you see any connections between work stress and sleep quality?" → AI analyzes
- **Effort if needed**: 2 hours
- **Priority**: LOW - Requires proving users need statistical rigor vs natural language exploration

#### Pattern Browsing CLI
- **Original Tasks**: `ponder analyze`, `ponder patterns list`, `ponder insights`
- **Better Approach**: Single `ponder converse` interface handles all exploration
- **Effort if needed**: 2 hours for dedicated commands
- **Priority**: MEDIUM - Might want batch analysis mode later, but start conversational

---

## Future Enhancements (Build After Validation)

### Persistent Insights Storage

**Current State**: Conversations are ephemeral (in-memory message history only)
**Enhancement**: Store AI-generated insights for future reference

**Implementation Options**:
1. **Insights as Special Entries** (Recommended)
   - Store insights as `.insight.md.age` files in journal directory
   - Embed in existing embeddings table with metadata tag
   - Retrieve via existing vector search
   - **Effort**: 4 hours
   - **Value**: HIGH - Preserves insights without new infrastructure

2. **Dedicated conversation_memory Table**
   - New table: `conversation_memory(insight_type, content, confidence, source_dates)`
   - New module: `src/db/memory.rs` with CRUD operations
   - **Effort**: 8 hours
   - **Value**: MEDIUM - More structure but adds complexity (Ultrathink concern)

**User Workflow**:
```bash
# During conversation
> Do you notice patterns in when I journal?
🤖 [AI analyzes and explains patterns]

> save that insight
✅ Saved as insight: "Weekend reflection pattern (confidence: 0.85)"

# Later retrieval
$ ponder converse
🤖 I remember noticing you journal more on weekends. Want to explore that further?
```

**Decision Point**: Implement only if users request it after trying ephemeral conversations.

---

### Automatic Insight Accumulation

**Current State**: AI analyzes on-demand per conversation
**Enhancement**: AI proactively builds "working memory" of accumulated insights over time

**Concept** (from Agentic Context Engineering research):
- After each conversation, AI synthesizes key takeaways
- Insights stored and retrieved in future conversations
- Forms evolving understanding of user's journal patterns

**Implementation**:
- `ponder converse --accumulate` flag to enable
- At conversation end, AI generates synthesis prompt: "Summarize new insights from this conversation"
- Store synthesis as insight entry (see Persistent Insights above)
- Future conversations load recent insights as context

**Effort**: 12 hours (requires persistent storage + synthesis pipeline)
**Value**: HIGH - This is the "second brain" use case
**Risk**: Insight drift over time - old insights may become stale or misrepresentative

**Decision Point**: Build after 2+ weeks of user feedback on basic conversations.

---

### Tool Calling Infrastructure (For Extensibility)

**Current State**: Phase 5 uses simplified structured JSON output (2 actions: search, respond)
**Enhancement**: Full Ollama tool calling if we add 5+ different tools/actions

**Why Deferred**: Tool calling is **over-engineered for binary decision** (search vs. respond). Current approach is simpler, faster, fewer failure modes.

**When to Build**: If we add many more actions beyond search/respond:
- `search_summaries` - Retrieve from summaries table
- `search_patterns` - Query detected patterns
- `generate_insight` - Create new insight entry
- `compare_timeframes` - "Compare July vs August"
- `cite_specific_entry` - "Show me the entry from Oct 15"

**Implementation** (full tool calling):
1. Tool registry in ollama.rs:
   ```rust
   struct ToolRegistry {
       tools: HashMap<String, ToolDefinition>,
   }

   impl ToolRegistry {
       fn get_available_tools(&self) -> Vec<ToolDefinition> { ... }
       fn execute(&self, tool_call: ToolCall, ctx: &Context) -> AppResult<ToolResult> { ... }
   }
   ```

2. Update reflection phase to use `chat_with_tools()`:
   ```rust
   let tools = registry.get_available_tools();
   let tool_call = ai_client.chat_with_tools(&model, &messages, &tools)?;
   let result = registry.execute(tool_call, &context)?;
   ```

3. Dynamic argument validation via JSON schemas
4. Better error messages for tool selection failures

**Benefits over current approach**:
- ✅ Idiomatic Ollama usage (native tool calling API)
- ✅ Better model support (Qwen3, Llama3.1 optimize for this)
- ✅ Extensible (easy to add new tools)
- ✅ Self-documenting (tools have descriptions, schemas)

**Tradeoffs**:
- ❌ More complex (tool registry, schemas, argument parsing)
- ❌ More failure modes (tool not found, invalid args, schema mismatch)
- ❌ Slower (extra round-trip for tool selection)
- ❌ Only worth it with 5+ tools

**Effort**: 2-3 days (reimplement reflection phase with tool calling)
**Value**: LOW until we have many tools, HIGH once we do
**Decision Point**: Only if we add 3+ new action types beyond search/respond

**Note**: Tool calling infrastructure partially implemented in src/ai/ollama.rs (lines 83-132, 716-811) but will be removed in Phase 5 cleanup per ultrathink recommendation. Can be restored from git history if needed.

---

### Multi-Session Context Tracking

**Current State**: Each `ponder converse` session starts fresh
**Enhancement**: Maintain context across multiple CLI invocations

**Challenges**:
- Session lifecycle: When do sessions expire?
- Storage: Where to persist message history?
- Security: How to ensure passphrase-protected conversation history?

**Implementation Options**:
1. **Session Files** (Simplest)
   - Store encrypted `.session.age` files with message history
   - Auto-load most recent session on startup
   - **Effort**: 6 hours
   - **Tradeoff**: File cleanup burden

2. **Session Table** (More Complex)
   - New table: `sessions(id, created_at, last_active, message_history_encrypted)`
   - Automatic timeout after 24 hours
   - **Effort**: 10 hours
   - **Tradeoff**: More infrastructure (Ultrathink concern)

**User Workflow**:
```bash
$ ponder converse
🤖 Continuing our conversation from earlier today...
   You asked about stress patterns. I found 3 more related entries since then.
```

**Decision Point**: Only if users explicitly request "remember our last conversation" feature.

---

### Advanced Context Windowing

**Current State**: RAG retrieves top-N similar chunks, assembles into context
**Enhancement**: Sophisticated context selection strategies

**Techniques** (from 2025 prompt engineering research):
1. **Semantic Clustering**: Group similar entries, pick representatives
2. **Temporal Weighting**: Recent entries weighted higher
3. **Diversity Sampling**: Ensure context covers multiple topics/time periods
4. **Relevance Re-ranking**: Two-stage retrieval (coarse + fine-grained)
5. **Context Compression**: Summarize less-relevant chunks to fit more context

**Implementation**:
- Add `context_strategy` parameter to `assemble_conversation_context()`
- Support strategies: `recent`, `diverse`, `clustered`, `compressed`
- **Effort**: 16-20 hours (research + implementation)
- **Value**: MEDIUM - Only matters for large journals (1000+ entries)

**Decision Point**: Benchmark current approach. If context quality issues emerge, prioritize specific strategy.

---

### Streaming Response Optimizations

**Current State**: Basic streaming via native Ollama API
**Potential Issues**: Backpressure, error recovery mid-stream, cancellation

**Enhancements**:
1. **Graceful Error Recovery**
   - If stream fails mid-response, retry from checkpoint
   - **Effort**: 4 hours

2. **User Cancellation** (Ctrl+C during response)
   - Clean shutdown without panic
   - Option to continue or start fresh
   - **Effort**: 3 hours

3. **Response Caching** (avoid re-asking same questions)
   - Cache question embeddings + responses
   - Check cache before generating new response
   - **Effort**: 6 hours
   - **Tradeoff**: Stale responses vs performance

**Decision Point**: Monitor user feedback for streaming issues before building.

---

### Insight Review & Curation

**Current State**: Insights stored but no management UI
**Enhancement**: Tools to review, edit, delete accumulated insights

**Features**:
```bash
$ ponder insights list
📝 Accumulated Insights (Last 30 Days)

1. Weekend Reflection Pattern (confidence: 0.85)
   Observed: 66% of entries on weekends, distinct tone shift
   Sources: 2024-10-15 to 2024-11-10 (12 entries)

2. Career Uncertainty Theme (confidence: 0.72)
   Recurring across 5 October entries
   Sources: 2024-10-05, 10-12, 10-23, 10-28, 11-02

$ ponder insights delete 2
✅ Deleted insight: Career Uncertainty Theme

$ ponder insights export insights.md
✅ Exported 5 insights to insights.md
```

**Implementation**:
- `ponder insights list/show/delete/export` subcommands
- Query insights from embeddings metadata or dedicated table
- **Effort**: 8 hours

**Decision Point**: Only relevant after persistent insights implemented and users accumulate 10+ insights.

---

### Model Selection Per Operation

**Current State**: Config specifies models globally (chat, embed, reasoning, summary)
**Enhancement**: Override model per conversation or question

**Use Cases**:
- Use faster model (gemma2:2b) for quick questions
- Use reasoning model (deepseek-r1:8b) for complex analysis
- Use large model (qwen2.5:32b) for important reflections

**Implementation**:
```bash
$ ponder converse --model gemma2:2b      # Fast, casual chat
$ ponder converse --model deepseek-r1    # Deep reasoning
$ ponder ask "summarize Oct" --model phi4:3.8b  # Efficient summaries
```

**Effort**: 2 hours (add CLI flag, pass to OllamaClient)
**Value**: LOW initially - Most users stick with one model
**Decision Point**: Build if users request model switching based on task.

---

### Conversation Export/Sharing

**Current State**: Conversations are terminal-only, ephemeral
**Enhancement**: Export conversations to markdown for sharing or archiving

**Features**:
```bash
$ ponder converse --save conversation-2024-11-10.md
🤖 [Conversation happens...]
✅ Conversation saved to conversation-2024-11-10.md

# Export format
# Conversation: 2024-11-10 15:30
## User
What patterns do you see in my entries?

## Assistant
Let me think through this step-by-step...
[Full response with citations]
```

**Implementation**:
- Add `--save <file>` flag to converse command
- Format conversation history as markdown
- Optionally encrypt with age
- **Effort**: 4 hours

**Value**: MEDIUM - Useful for journaling about journaling, sharing with therapist
**Decision Point**: User-requested feature, not core functionality.

---

### Voice Input Integration

**Current State**: Text-based conversation only
**Enhancement**: Speak questions, listen to responses

**Implementation Options**:
1. **Local Whisper** (Privacy-preserving)
   - Use whisper.cpp for speech-to-text locally
   - Pipe to converse command
   - **Effort**: 12 hours (whisper integration + audio handling)

2. **System Speech Recognition** (Platform-dependent)
   - macOS: Use `NSSpeechRecognizer`
   - Linux: Use PocketSphinx or vosk
   - **Effort**: 20+ hours (cross-platform complexity)

**User Workflow**:
```bash
$ ponder converse --voice
🎤 Listening... (Press space to start/stop)

> [User speaks: "What did I write about yesterday?"]
🤖 [AI responds in text and optionally TTS]
```

**Value**: HIGH for accessibility, LOW for typical users
**Effort**: HIGH (12-20 hours)
**Decision Point**: Only if accessibility is critical or highly requested.

---

## Technical Debt Opportunities

### Refactor Context Assembly

**Current State**: Logic will be duplicated in `ops/ask.rs` and `ops/converse.rs`
**Opportunity**: Extract to shared module if patterns emerge

**Wait for**: Both ask and converse implemented, identify exact duplication
**Effort**: 3 hours
**Value**: Code cleanliness, easier to optimize both paths

---

### Performance: Parallel Decryption

**Current State**: Entries decrypted serially in RAG pipeline
**Opportunity**: Use Rayon to decrypt multiple entries in parallel

**When**: Journal has 100+ entries and RAG feels slow
**Effort**: 4 hours
**Value**: 3-5x speedup on multi-core systems

---

### Performance: ANN Vector Search

**Current State**: Linear scan through embeddings (O(n) with cosine similarity)
**Opportunity**: Implement HNSW (Hierarchical Navigable Small World) index

**When**: Journal has 1000+ entries and vector search becomes bottleneck
**Effort**: 16 hours (HNSW implementation or integration)
**Value**: 100x speedup for large journals

---

## Product Strategy Notes

### Why Minimal First?

1. **Validate Core Value**: Does conversational interface resonate with users?
2. **Discover Real Needs**: What do users actually ask about their journals?
3. **Avoid Overengineering**: Don't build features users won't use
4. **Faster Iteration**: Ship in 1 week vs 4 weeks, learn faster

### When to Build Each Enhancement?

**Tier 1** (Build next if validated):
- Persistent insights storage (insights as entries)
- Conversation export to markdown
- Model selection per operation

**Tier 2** (Build if explicitly requested):
- Automatic insight accumulation
- Multi-session context tracking
- Insight review & curation

**Tier 3** (Build only if clear demand):
- Advanced context windowing strategies
- Voice input integration
- Response caching

### Success Metrics to Guide Backlog

Track these to decide what to build:
- **Conversations per week**: High usage → invest in enhancements
- **Average conversation length**: Long → need session persistence
- **Repeat questions**: Common → need insight accumulation
- **Large journals**: 500+ entries → need performance work
- **User requests**: "I wish it could..." → direct feature prioritization

---

## Maintenance

This backlog will be reviewed:
- After Phase 4 MVP ships (conversational interface complete)
- Every 2 weeks during active feature development
- When user feedback indicates missing capabilities
- When performance issues emerge

Items graduate from backlog → TODO.md when:
1. User demand is clear (multiple requests or high upvotes)
2. Core functionality is validated and stable
3. Enhancement builds on proven usage patterns
