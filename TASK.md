# Ponder AI Enhancement Plan

## Vision
Transform Ponder from "encrypted journal with RAG" to "AI-powered conversation partner for self-knowledge" - all 100% private and local.

## Target Users (All Personas)
1. **Reflective Professional** - Career-focused, pattern recognition in productivity/decisions
2. **Mental Health Focus** - Privacy-conscious therapy/personal growth, emotional patterns
3. **Creative/Writer** - Idea capture and rediscovery, connecting disparate thoughts
4. **Quantified Self** - Data-driven self-improvement, correlation discovery
5. **Deep Thinker** - Second brain, knowledge graph building

## Architecture Changes

### 1. Configurable Model System (Foundation)
**File:** `src/config/mod.rs`

Add model configuration per AI operation type:
```rust
pub struct AIModels {
    embed_model: String,      // default: "nomic-embed-text"
    chat_model: String,        // default: "llama3.3:8b"
    reasoning_model: String,   // default: "deepseek-r1:8b"
    summary_model: String,     // default: "phi4:3.8b"
}
```

Environment variables:
- `PONDER_EMBED_MODEL`
- `PONDER_CHAT_MODEL`
- `PONDER_REASONING_MODEL`
- `PONDER_SUMMARY_MODEL`

Graceful fallbacks if models unavailable.

### 2. Enhanced Database Schema
**File:** `src/db/mod.rs`

#### New `summaries` Table
```sql
CREATE TABLE summaries (
    id INTEGER PRIMARY KEY,
    date TEXT NOT NULL,
    level TEXT NOT NULL,  -- 'daily', 'weekly', 'monthly'
    summary_encrypted BLOB NOT NULL,  -- age-encrypted summary
    topics TEXT,  -- JSON array of detected topics
    sentiment REAL,  -- -1.0 to 1.0
    word_count INTEGER,
    created_at TEXT
);
```

#### New `patterns` Table
```sql
CREATE TABLE patterns (
    id INTEGER PRIMARY KEY,
    pattern_type TEXT,  -- 'temporal', 'topic', 'sentiment'
    description TEXT,
    metadata TEXT,  -- JSON with pattern details
    first_seen TEXT,
    last_seen TEXT
);
```

## Three MVP Feature Tracks

### Track 1: Progressive Summaries

**New files:**
- `src/ops/summarize.rs` - Summary generation logic
- `src/ai/summarizer.rs` - Prompt engineering for summaries
- `src/db/summaries.rs` - Database operations for summaries

**Flow:**
```
ponder summarize --period weekly
  ↓
1. Fetch last 7 days of entries from DB
2. Decrypt each entry
3. Generate daily summaries (Phi-4 model)
4. Combine into weekly summary
5. Extract topics + sentiment
6. Encrypt and store in summaries table
7. Display to user
```

**Commands:**
- `ponder summarize --period [daily|weekly|monthly]`
- `ponder summarize --date 2025-01-15` (specific day)
- `ponder summaries list` (browse past summaries)
- `ponder summaries show --date 2025-01-15` (view specific summary)

**Features:**
- Progressive layers: Daily → Weekly → Monthly
- Topic extraction from each summary
- Sentiment scoring (-1.0 to 1.0)
- All summaries encrypted at rest
- Optional auto-generation on schedule

### Track 2: Pattern Detection

**New files:**
- `src/ops/patterns.rs` - Pattern detection logic
- `src/ai/analyzer.rs` - AI-powered pattern recognition
- `src/db/patterns.rs` - Database operations for patterns

**Pattern Types:**
1. **Temporal Patterns**
   - "You write most on Sunday evenings"
   - "Entries peak between 9-11pm"
   - "You journal 3x more on weekdays"

2. **Topic Clustering**
   - "Top 5 topics this month: work (12), health (7), relationships (5)..."
   - "New topic emerged this week: 'career change'"
   - "Haven't written about X in 3 weeks"

3. **Sentiment Trends**
   - Graph mood over time
   - "Your mood improved 15% this month"
   - "Sentiment dips on Mondays"

4. **Correlation Discovery**
   - "Stress entries correlate with weekdays"
   - "Exercise mentions precede positive entries by 1 day"
   - "Topic X and Y often appear together"

**Flow:**
```
ponder analyze --patterns
  ↓
1. Load all entry metadata + embeddings
2. Cluster embeddings by topic (cosine similarity)
3. Analyze temporal patterns (time-of-day, day-of-week)
4. Run sentiment analysis on recent entries
5. Generate natural language pattern descriptions
6. Store in patterns table
7. Display insights
```

**Commands:**
- `ponder analyze --patterns` (detect new patterns)
- `ponder patterns list` (show discovered patterns)
- `ponder patterns --type temporal` (filter by type)
- `ponder insights` (AI narrative of current patterns)

**Privacy Considerations:**
- Pattern descriptions stored encrypted
- All analysis happens locally
- No raw text in pattern metadata

### Track 3: Conversational Mode

**New files:**
- `src/ops/converse.rs` - Conversational journaling flow
- `src/ai/conversation.rs` - Multi-turn conversation management
- `src/cli/converse.rs` - CLI subcommand for conversation

**Flow:**
```
ponder converse
  ↓
1. Enter chat loop with AI
2. AI asks guided questions based on:
   - Time of day ("Good evening! How was your day?")
   - Day of week ("How was your weekend?")
   - Recent patterns ("You seemed stressed last week...")
   - Topics not written about recently
3. User responds naturally
4. AI follows up, explores deeper
5. When user says "done" or "finish", AI generates structured entry
6. User reviews/edits generated entry
7. Save as normal encrypted entry with auto-embeddings
```

**Features:**
- **Context-Aware Prompts**
  - "Last time you wrote about work, you seemed stressed. How's that going?"
  - "You mentioned starting a new project. How's it progressing?"

- **Follow-Up Questions**
  - "Tell me more about that feeling"
  - "What do you think caused that?"
  - "How does that compare to last time?"

- **Pattern Integration**
  - "You haven't mentioned [topic] in 2 weeks. Anything new there?"
  - "This reminds me of what you wrote on [date]"

- **Streaming Responses**
  - Real-time AI responses for natural conversation feel
  - Progressive display of questions

**Commands:**
- `ponder converse` (start open conversation)
- `ponder converse --quick` (shorter version, 3-5 questions)
- `ponder converse --topic "work stress"` (focused conversation)
- `ponder converse --continue` (continue yesterday's conversation)

**Conversation Memory:**
- Stores conversation context during session
- References recent entries for context
- Clears after entry created (privacy)

## Technical Implementation Details

### AI Client Enhancements
**File:** `src/ai/client.rs`

```rust
impl OllamaClient {
    // Add model selection per task
    pub async fn chat_with_model(
        &self,
        model: &str,
        messages: Vec<Message>
    ) -> Result<String>;

    // Add streaming support for conversational mode
    pub async fn chat_stream(
        &self,
        messages: Vec<Message>
    ) -> Result<impl Stream<Item = String>>;

    // Add sentiment analysis
    pub async fn analyze_sentiment(&self, text: &str) -> Result<f32>;  // -1.0 to 1.0

    // Add topic extraction
    pub async fn extract_topics(&self, text: &str) -> Result<Vec<String>>;
}
```

### New Prompt Templates
**File:** `src/ai/prompts.rs`

```rust
// Summary prompts
pub const SUMMARY_DAILY: &str = "Summarize this journal entry in 2-3 sentences, capturing the main themes and emotions...";

pub const SUMMARY_WEEKLY: &str = "Synthesize these daily summaries into a cohesive weekly summary. Identify recurring themes and emotional arcs...";

pub const SUMMARY_MONTHLY: &str = "Create a comprehensive monthly summary from these weekly summaries. Highlight major developments and patterns...";

// Pattern detection prompts
pub const PATTERN_DETECTION: &str = "Analyze this journal metadata and identify patterns in writing times, topics, and sentiment...";

pub const TOPIC_CLUSTERING: &str = "Group these entry excerpts by topic. Identify the main themes and how they relate...";

// Conversation prompts
pub const CONVERSATION_START: &str = "You are a thoughtful journaling companion. Ask open-ended questions to help the user reflect on their day. Be warm, curious, and non-judgmental...";

pub const CONVERSATION_FOLLOWUP: &str = "Based on what they just said, ask a deeper follow-up question that helps them explore their feelings or thoughts more fully...";

pub const CONVERSATION_SYNTHESIZE: &str = "Based on this conversation, create a well-structured journal entry that captures the key points, emotions, and insights...";

// Analysis prompts
pub const SENTIMENT_ANALYSIS: &str = "Analyze the overall sentiment of this text on a scale from -1.0 (very negative) to 1.0 (very positive). Return only a number...";

pub const TOPIC_EXTRACTION: &str = "Extract the 3-5 main topics from this text as a JSON array of strings...";
```

### Configuration System
**File:** `src/config/mod.rs`

Add new configuration section:

```rust
#[derive(Debug, Clone)]
pub struct AIConfig {
    pub embed_model: String,
    pub chat_model: String,
    pub reasoning_model: String,
    pub summary_model: String,
    pub auto_summarize_daily: bool,
    pub auto_summarize_weekly: bool,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            embed_model: "nomic-embed-text".to_string(),
            chat_model: "llama3.3:8b".to_string(),
            reasoning_model: "deepseek-r1:8b".to_string(),
            summary_model: "phi4:3.8b".to_string(),
            auto_summarize_daily: false,
            auto_summarize_weekly: false,
        }
    }
}
```

Load from environment:
```rust
pub fn load_ai_config() -> Result<AIConfig> {
    Ok(AIConfig {
        embed_model: env::var("PONDER_EMBED_MODEL")
            .unwrap_or_else(|_| "nomic-embed-text".to_string()),
        chat_model: env::var("PONDER_CHAT_MODEL")
            .unwrap_or_else(|_| "llama3.3:8b".to_string()),
        reasoning_model: env::var("PONDER_REASONING_MODEL")
            .unwrap_or_else(|_| "deepseek-r1:8b".to_string()),
        summary_model: env::var("PONDER_SUMMARY_MODEL")
            .unwrap_or_else(|_| "phi4:3.8b".to_string()),
        // ... load other settings
    })
}
```

### Example User Configuration
**File:** `~/.config/ponder/config.toml` (optional)

```toml
[ai]
# Embedding model for semantic search
embed_model = "nomic-embed-text"

# Chat model for ask/reflect/converse
chat_model = "llama3.3:8b"

# Reasoning model for deep analysis
reasoning_model = "deepseek-r1:8b"

# Specialized model for summarization
summary_model = "phi4:3.8b"

[summaries]
# Auto-generate daily summary at end of day
auto_generate_daily = true

# Auto-generate weekly summary on Sundays
auto_generate_weekly = true

# Summary verbosity (1-5, higher = more detailed)
verbosity = 3

[conversation]
# Default number of questions in conversational mode
default_questions = 5

# How many follow-up questions to ask
follow_up_depth = 2

# Include pattern insights in conversation
use_patterns = true

[patterns]
# Minimum entries needed to detect patterns
min_entries = 10

# How far back to analyze (days)
analysis_window = 90
```

## CLI Command Structure

### New Subcommands

```rust
#[derive(Debug, Subcommand)]
pub enum PonderCommand {
    // ... existing commands (Edit, Ask, Reflect, Search, Lock)

    /// Generate summaries of journal entries
    Summarize(SummarizeArgs),

    /// List or view past summaries
    Summaries(SummariesArgs),

    /// Analyze patterns in journal entries
    Analyze(AnalyzeArgs),

    /// List discovered patterns
    Patterns(PatternsArgs),

    /// Get AI insights about your journal
    Insights(InsightsArgs),

    /// Conversational journaling mode
    Converse(ConverseArgs),
}

#[derive(Debug, Args)]
pub struct SummarizeArgs {
    /// Period to summarize (daily, weekly, monthly)
    #[arg(long, value_enum)]
    pub period: Option<SummaryPeriod>,

    /// Specific date to summarize (YYYY-MM-DD)
    #[arg(long)]
    pub date: Option<String>,

    /// Force regeneration even if summary exists
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct AnalyzeArgs {
    /// Detect patterns in journal entries
    #[arg(long)]
    pub patterns: bool,

    /// Analyze sentiment trends
    #[arg(long)]
    pub sentiment: bool,

    /// Cluster topics
    #[arg(long)]
    pub topics: bool,
}

#[derive(Debug, Args)]
pub struct ConverseArgs {
    /// Quick mode (fewer questions)
    #[arg(long)]
    pub quick: bool,

    /// Focus on specific topic
    #[arg(long)]
    pub topic: Option<String>,

    /// Continue previous conversation
    #[arg(long)]
    pub continue_conv: bool,
}
```

## Database Migrations

### Migration 1: Add Summaries Table
**File:** `src/db/migrations/001_add_summaries.rs`

```sql
CREATE TABLE IF NOT EXISTS summaries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    level TEXT NOT NULL CHECK(level IN ('daily', 'weekly', 'monthly')),
    summary_encrypted BLOB NOT NULL,
    topics TEXT,  -- JSON array
    sentiment REAL CHECK(sentiment >= -1.0 AND sentiment <= 1.0),
    word_count INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(date, level)
);

CREATE INDEX idx_summaries_date ON summaries(date);
CREATE INDEX idx_summaries_level ON summaries(level);
```

### Migration 2: Add Patterns Table
**File:** `src/db/migrations/002_add_patterns.rs`

```sql
CREATE TABLE IF NOT EXISTS patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_type TEXT NOT NULL CHECK(pattern_type IN ('temporal', 'topic', 'sentiment', 'correlation')),
    description TEXT NOT NULL,
    metadata TEXT,  -- JSON with pattern details
    confidence REAL CHECK(confidence >= 0.0 AND confidence <= 1.0),
    first_seen TEXT NOT NULL,
    last_seen TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_patterns_type ON patterns(pattern_type);
CREATE INDEX idx_patterns_last_seen ON patterns(last_seen);
```

## Phased Implementation

### Phase 1: Foundation (Week 1-2)
**Goal:** Set up infrastructure for new features

**Tasks:**
1. [ ] Implement configurable model system in `src/config/mod.rs`
2. [ ] Add AI model config loading from environment
3. [ ] Create database migrations for summaries and patterns tables
4. [ ] Enhance `OllamaClient` with streaming support
5. [ ] Add sentiment analysis method to AI client
6. [ ] Add topic extraction method to AI client
7. [ ] Create new prompt templates in `src/ai/prompts.rs`
8. [ ] Write tests for configuration system
9. [ ] Write tests for new AI client methods

**Deliverables:**
- Models configurable via env vars
- Database schema supports summaries and patterns
- AI client can stream responses
- Sentiment and topic extraction working

### Phase 2: Track 1 - Progressive Summaries (Week 3-4)
**Goal:** Implement summary generation at all levels

**Tasks:**
1. [ ] Create `src/ops/summarize.rs`
2. [ ] Create `src/ai/summarizer.rs` with prompts
3. [ ] Create `src/db/summaries.rs` for DB operations
4. [ ] Implement daily summary generation
5. [ ] Implement weekly summary aggregation
6. [ ] Implement monthly summary aggregation
7. [ ] Add `summarize` CLI subcommand
8. [ ] Add `summaries list` command
9. [ ] Add `summaries show` command
10. [ ] Implement auto-summarization option
11. [ ] Add tests for summary generation
12. [ ] Add integration tests for CLI commands

**Deliverables:**
- `ponder summarize` working for all periods
- Summaries encrypted and stored
- Topic and sentiment extraction working

### Phase 3: Track 2 - Pattern Detection (Week 5-6)
**Goal:** Discover and surface patterns in journal

**Tasks:**
1. [ ] Create `src/ops/patterns.rs`
2. [ ] Create `src/ai/analyzer.rs`
3. [ ] Create `src/db/patterns.rs`
4. [ ] Implement temporal pattern detection
5. [ ] Implement topic clustering algorithm
6. [ ] Implement sentiment trend analysis
7. [ ] Implement correlation discovery
8. [ ] Add `analyze` CLI subcommand
9. [ ] Add `patterns list` command
10. [ ] Add `insights` command for narrative view
11. [ ] Add tests for pattern detection algorithms
12. [ ] Add integration tests

**Deliverables:**
- `ponder analyze --patterns` working
- All 4 pattern types detected
- Patterns stored encrypted
- Natural language descriptions

### Phase 4: Track 3 - Conversational Mode (Week 7-8)
**Goal:** Chat-based journaling interface

**Tasks:**
1. [ ] Create `src/ops/converse.rs`
2. [ ] Create `src/ai/conversation.rs`
3. [ ] Add `converse` CLI subcommand
4. [ ] Implement conversation loop with streaming
5. [ ] Implement context-aware question generation
6. [ ] Implement follow-up question logic
7. [ ] Integrate pattern insights into conversation
8. [ ] Implement conversation → entry synthesis
9. [ ] Add review/edit flow before saving
10. [ ] Add `--quick` mode
11. [ ] Add `--topic` focused mode
12. [ ] Add tests for conversation flow
13. [ ] Add integration tests

**Deliverables:**
- `ponder converse` fully functional
- Streaming conversation feels natural
- Pattern integration works
- Generated entries are high quality

### Phase 5: Polish & Integration (Week 9-10)
**Goal:** Make everything work together seamlessly

**Tasks:**
1. [ ] Create unified `insights` dashboard command
2. [ ] Optimize performance (embeddings, queries)
3. [ ] Add comprehensive error handling
4. [ ] Improve prompt engineering based on testing
5. [ ] Add graceful degradation for missing models
6. [ ] Write user documentation
7. [ ] Create example configs
8. [ ] Performance benchmarking
9. [ ] Security audit (encryption, session management)
10. [ ] End-to-end testing with real journal data

**Deliverables:**
- All features work together
- Performance is acceptable
- Documentation complete
- Ready for real-world use

## Success Metrics

### Functional Requirements
- ✅ All 3 MVP tracks fully functional
- ✅ Models configurable via env vars and config file
- ✅ All summaries and patterns encrypted at rest
- ✅ No degradation to existing features (edit, ask, reflect, search, lock)
- ✅ Comprehensive test coverage (>80%)
- ✅ Documentation for each new command

### Quality Requirements
- ✅ Summaries are accurate and useful
- ✅ Patterns detected are meaningful (not noise)
- ✅ Conversational mode feels natural
- ✅ AI responses are relevant and contextual
- ✅ Performance: Summary generation <5s, pattern detection <10s
- ✅ Streaming responses feel real-time (<100ms chunks)

### Security Requirements
- ✅ All AI processing local (no cloud)
- ✅ All summaries/patterns encrypted
- ✅ Session timeout applies to new features
- ✅ No plaintext leakage
- ✅ Passphrase never logged or exposed

## Research Findings Summary

### Best Models for Ponder (October 2024/2025)

**Embeddings:**
- **Keep:** `nomic-embed-text` (274MB, 768 dims, 8192 ctx) - excellent balance
- **Alternative:** `mxbai-embed-large` - stronger, multilingual

**Chat Models:**
- **Recommended:** `llama3.3:8b` - state-of-the-art from Meta, excellent reasoning
- **Reasoning:** `deepseek-r1:8b` - specialized for deep analysis
- **Summarization:** `phi4:3.8b` - Microsoft's latest, 128K context

**Trends from Power Users:**
- Conversational journaling (chat → structured entry)
- Progressive summarization (daily → weekly → monthly)
- Pattern detection dashboards
- Knowledge graph building
- Local-first, privacy-focused AI

## Next Steps After Plan Approval

1. Review and approve this plan
2. Set up development branch: `git checkout -b feature/ai-enhancements`
3. Start Phase 1: Foundation
4. Incremental commits with tests
5. Test with real journal data throughout
6. Iterate based on actual usage

## Questions to Resolve

1. Should summaries be viewable without decryption (metadata only)?
2. How to handle Ollama model not available errors?
3. Should patterns auto-update or manual refresh?
4. Conversation history: store or ephemeral?
5. Auto-summarization: cron job or on-demand only?

## Future Considerations (Post-MVP)

- Multi-modal support (LLaVA for images)
- Knowledge graph visualization
- Export summaries/patterns to markdown
- Plugin system for custom analyzers
- Web UI for pattern visualization
- Mobile app with conversational mode
- Voice input for conversational journaling
