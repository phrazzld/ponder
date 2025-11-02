# TODO: Comprehensive Trend Analysis - Robustness Features

**Goal**: Make trend analysis robust for 1000+ entry journals with automatic session management, resumable checkpoints, and clear progress feedback.

**Success Criteria**: User can analyze 1000 entries with automatic session extension, interrupt at any time (Ctrl+C) and resume seamlessly, see clear progress bars with ETAs, and never worry about implementation details.

---

## ✅ Completed

### Phase 1: Progress Infrastructure
- ✅ Added `indicatif` dependency
- ✅ Created `PhaseProgress` wrapper with ETA and throughput
- ✅ Integrated progress bars into Discovery phase
- ✅ Integrated progress bars into Extraction phase
- ✅ Added spinner to Synthesis phase

### Phase 2: Checkpoint System
- ✅ Defined checkpoint data structures (TrendsCheckpoint, CheckpointPhase, SerializableInsight)
- ✅ Implemented checkpoint I/O (get/save/load/delete with atomic writes)
- ✅ Discovery phase: checkpoint loading, resume logic, save every 100 entries
- ✅ Extraction phase: checkpoint loading, resume logic, save every 50 entries
- ✅ Cleanup checkpoint on successful completion

**Result**: Trend analysis now has progress bars with ETA and can be interrupted/resumed at any point.

---

## 🚧 Remaining Work

### Phase 3: Session Auto-Extension (15 min)

**Goal**: Prevent session timeout during long-running analysis (30-40 min for 1000 entries).

- [ ] Create `SessionExtender` struct in `src/crypto/session.rs`
  - Background thread sleeps 5 min, calls `session.get_passphrase()` to refresh
  - Use `Arc<Mutex<SessionManager>>`, `Arc<AtomicBool>` for stop signal
  - Auto-stop on Drop (RAII cleanup)

- [ ] Integrate SessionExtender into `analyze_trends()`
  - Wrap SessionManager in Arc<Mutex>
  - Start extender after session unlock
  - Update all session access to use `.lock().unwrap()`

- [ ] Add upfront time estimation
  - Calculate: `(total_entries as f64 * 1.5) / 60.0` minutes
  - Print: "⏱️  Estimated time: X minutes" and "🔐 Session will be automatically extended"

---

### Phase 4: Graceful Interruption (15 min)

**Goal**: Handle Ctrl+C gracefully with checkpoint save and user-friendly messaging.

- [ ] Add `Interrupted` variant to `AppError` in `src/errors/mod.rs`
  - `#[error("Operation interrupted by user")] Interrupted,`

- [ ] Add `signal-hook` dependency (v0.3)

- [ ] Implement interrupt handler in `analyze_trends()`
  - Use `Arc<AtomicBool>` for interrupted flag
  - Spawn signal handler thread: `signal_hook::iterator::Signals::new(&[SIGINT])`
  - On signal: set flag, print "⚠️  Interrupt received. Saving progress..."

- [ ] Check interrupt flag in `discover_relevant_entries()` loop
  - If interrupted: save checkpoint, return `Err(AppError::Interrupted)`

- [ ] Check interrupt flag in `extract_insights()` loop
  - If interrupted: save checkpoint, return `Err(AppError::Interrupted)`

- [ ] Handle `AppError::Interrupted` in `cmd_trends()` (`src/main.rs`)
  - Print: "Progress saved. Run again to resume from entry X."
  - Return `Ok(())` (exit code 0, not a failure)

---

### Phase 5: Testing & Validation (30 min)

**Goal**: Verify checkpoint system works correctly and handles edge cases.

#### Unit Tests (`src/ops/trends.rs`)

- [ ] Checkpoint save/load roundtrip test
  - Create checkpoint → save → load → verify fields match
  - Test query mismatch → returns None
  - Test non-existent file → returns None

- [ ] SerializableInsight conversion test
  - `EntryInsight` → `SerializableInsight` → `EntryInsight` roundtrip
  - Verify date format (YYYY-MM-DD)

#### Manual Integration Tests

- [ ] Interrupt during Phase 1
  - Create 20 test entries
  - Run `ponder trends "test query"`
  - Ctrl+C after ~10 entries
  - Resume: verify skips processed entries

- [ ] Verify checkpoint JSON structure
  - Inspect checkpoint file after interrupt
  - Verify: query, started_at, phase, processed_entry_ids, relevant_entry_ids, extracted_insights

- [ ] Verify checkpoint deletion on completion
  - Run full analysis to completion
  - Check checkpoint file is deleted

- [ ] Verify checkpoint deletion on manual deletion
  - Interrupt run, delete checkpoint manually
  - Resume: verify starts fresh without error

---

### Phase 6: Documentation (15 min)

**Goal**: Document checkpoint system for developers and users.

- [ ] Add module-level doc to `src/ops/trends.rs`
  - Explain checkpoint location, save frequency, resume behavior

- [ ] Document `SessionExtender` in `src/crypto/session.rs`
  - Explain why needed, how it works, thread safety

- [ ] Update CLAUDE.md (`src/ops/` section)
  - Add: "Trend analysis: 30-40 min for 1000 entries"
  - Add: "Safe to interrupt (Ctrl+C), resumes from checkpoint"
  - Add: "Session auto-extended, no manual timeout management"

---

## Summary

**Completed**: Phases 1-2 (Progress + Checkpoints) ✅
**Remaining**: Phases 3-6 (Session Extension + Interruption + Testing + Docs)
**Estimated time**: ~75 minutes for remaining work
