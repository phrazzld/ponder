# MANIFESTO: The Ponder Way

## 🎯 1. Ponder's Identity: What Ponder Is (and Isn't)

**Ponder IS:**

*   A **simple, opinionated, single-purpose CLI tool** for maintaining a personal journal.
*   Designed for **low-friction, reliable, daily reflection** using your preferred text editor.
*   **Local-first and filesystem-based**, storing entries as plain Markdown files (e.g., `YYYYMMDD.md`).
*   **Privacy-conscious**, with support for encrypting entries at rest.
*   **Continuous across devices**, with simple synchronization capabilities.
*   Built in Rust with a focus on **reliability, performance, and a small, understandable codebase** (target: ~500-1500 lines of core logic).
*   **Intentionally limited in scope and configuration.**

**Ponder IS NOT:**

*   A journaling **framework or library** intended for reuse in other applications.
*   A **platform** for complex note-taking, linking, or knowledge management (like Obsidian, Notion, etc.).
*   A **web application or GUI tool**.
*   Designed for complex data structures beyond simple dated entries.
*   A **highly configurable or extensible system** with plugins or deep customization points.
*   A project optimized for hypothetical flexibility or future extensibility.

**Core Purpose:** To provide a secure, straightforward, and reliable tool for the simple act of daily reflection and review, accessible across your devices, without the overhead or complexity of larger systems.

**Essential Features:**
*   Local filesystem storage with plain text/Markdown entries
*   External editor integration
*   Optional encryption for privacy (using established tools like age/gpg)
*   Optional synchronization for continuity (using established tools like git)
*   Minimal configuration (paths, editor, sync settings)

**Intentional Limitations:**
*   Single-user focus
*   Date-based entries only
*   No plugins or extensions
*   No complex metadata beyond dates
*   No internal search or tagging systems

Ponder's strength lies in its focused scope. *Simplicity is the core product feature.*

---

## 💡 2. The Ponder Philosophy: Radical Simplicity

Ponder's design is guided by one overarching principle: **Radical Simplicity**. This is not merely a preference but a fundamental design constraint. For a focused CLI tool like Ponder, complexity is the primary enemy of maintainability, readability, and correctness.

This philosophy manifests through the following tenets:

*   **Simplicity First**: We always seek the simplest possible solution that correctly meets Ponder's *defined, limited* requirements.
*   **Essential Features Only**: We include only features that directly serve the core journaling use case. Privacy (encryption) and continuity (sync) are considered essential; everything else is scrutinized.
*   **Direct Implementation**: Features are implemented directly using standard tools and libraries, not through abstraction layers.
*   **YAGNI (You Ain't Gonna Need It)**: We do not build for hypothetical future requirements that fall outside Ponder's core identity.
*   **Pragmatism Over Dogma**: "Best practices" must justify their complexity cost. If a practice adds overhead without clear benefit to Ponder's specific needs, we deviate.

---

## 🏛️ 3. Architectural Principles & Decisions

These principles define Ponder's architecture and guide all implementation decisions.

### 3.1. No Unnecessary Abstractions

**The Principle**: Ponder uses direct function calls and concrete types rather than trait abstractions for core operations.

**Justification**:
1.  **Single Implementation Path**: Ponder has exactly one way of performing each operation:
    *   **File I/O**: Direct use of `std::fs`
    *   **Process Execution**: Direct use of `std::process::Command`
    *   **Encryption**: Direct integration with external tools (age/gpg)
    *   **Sync**: Direct integration with external tools (git)
    
2.  **Clarity Over Flexibility**: Direct calls are explicit and immediately understandable.
3.  **Minimal Overhead**: No cognitive or performance cost from indirection.
4.  **No Polymorphism Needed**: We're not building a framework; we don't need swappable implementations.

### 3.2. Integration Testing Over Mock-Based Unit Tests

**The Principle**: Ponder prioritizes integration tests that exercise real behavior over unit tests with mocked dependencies.

**The Strategy**:
1.  **Integration Tests First**: Tests run the actual binary with real (temporary) filesystem operations.
2.  **Unit Tests for Pure Logic**: Date calculations, parsing, and other pure functions are unit tested.
3.  **No Internal Mocking**: We don't mock Ponder's own modules or standard library calls.

### 3.3. Security Through Simplicity

**The Principle**: Security is achieved through simple, auditable validations rather than complex frameworks.

**Implementation**:
1.  **Editor Command Validation**: Simple string validation prevents command injection.
2.  **Encryption Integration**: Use established external tools rather than implementing crypto.
3.  **Path Validation**: Direct validation of user-provided paths.

### 3.4. Modularity Through Organization

**The Principle**: Code is organized into logical modules with clear responsibilities, connected through explicit function calls.

**Structure**:
- `src/cli/`: Command-line interface
- `src/config/`: Configuration management
- `src/journal_logic.rs`: Core journaling operations
- `src/sync.rs`: Synchronization operations (if implemented)
- `src/encryption.rs`: Encryption operations (if implemented)

---

## 🛠️ 4. Implementing Essential Features

When implementing features like encryption and synchronization, we maintain our principles:

### Encryption
- Direct integration with established tools (age, gpg)
- Simple command-line interface: `ponder encrypt` / `ponder decrypt`
- No custom cryptography implementation
- No complex key management systems

### Synchronization
- Direct integration with git or similar tools
- Simple command-line interface: `ponder sync`
- No custom conflict resolution beyond what git provides
- No complex merge strategies

Both features follow the pattern: **wrap existing, proven tools with the minimum interface needed for Ponder's use case**.

---

## 🌱 5. Guiding Contributors and Reviewers

### The Ponder Way: DOs

*   ✅ **Embrace Radical Simplicity**: Always question if your change introduces *necessary* complexity.
*   ✅ **Implement Directly**: Use standard library functions and established external tools directly.
*   ✅ **Test Behavior, Not Implementation**: Write integration tests for user-visible behavior.
*   ✅ **Validate Security Simply**: Clear, auditable validation over complex frameworks.
*   ✅ **Follow Conventions**: Use rustfmt, clippy, and conventional commits.
*   ✅ **Document Intent**: Explain *why* in comments, not just *what*.

### The Ponder Way: DON'Ts

*   ❌ **Don't Add Unnecessary Abstractions**: No traits, interfaces, or plugins for single-implementation features.
*   ❌ **Don't Mock Internal Code**: Test through the public interface instead.
*   ❌ **Don't Expand Core Scope**: No tagging, search, templates, or multiple journal support.
*   ❌ **Don't Implement Complex Features**: Wrap existing tools instead.
*   ❌ **Don't Over-Engineer**: Build for today's defined needs only.

### Code Examples

**AVOID: Abstraction for single implementation**
```rust
// ❌ DON'T DO THIS
trait Storage {
    fn write(&self, path: &Path, content: &str) -> Result<()>;
}
struct FileStorage;
impl Storage for FileStorage { ... }
```

**EMBRACE: Direct implementation**
```rust
// ✅ DO THIS
use std::fs;
pub fn write_entry(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content)?;
    Ok(())
}
```

**AVOID: Complex in-house solution**
```rust
// ❌ DON'T DO THIS
mod crypto {
    pub fn encrypt_aes256(data: &[u8], key: &[u8]) -> Vec<u8> { ... }
}
```

**EMBRACE: Wrap existing tools**
```rust
// ✅ DO THIS
use std::process::Command;
pub fn encrypt_file(path: &Path) -> Result<()> {
    Command::new("age")
        .args(&["-r", recipient, path.to_str().unwrap()])
        .status()?;
    Ok(())
}
```

---

## ❓ 6. Addressing Common Questions

**"Why no traits for testability?"**
- Integration tests provide better confidence for I/O-heavy tools
- Direct code is easier to understand and debug
- Mocking internal code often tests the mocks, not the behavior

**"What if we need plugins later?"**
- We won't. Ponder's scope is intentionally fixed
- If you need plugins, you need a different tool
- Fork if you must, but mainline Ponder stays simple

**"Why wrap external tools instead of using libraries?"**
- Reduces dependencies and compilation time
- Leverages battle-tested implementations
- Keeps the codebase small and focused
- Users already trust these tools

**"How do we add new features?"**
- First ask: Does this serve the core journaling use case?
- If yes: Can it be implemented simply by wrapping existing tools?
- If no to either: It doesn't belong in Ponder

---

## 🏁 7. Final Words: Simplicity as a Value

Ponder is an exercise in **radical simplicity, clarity, and restraint**. Every line of code must justify its existence. Every feature must directly serve the core use case.

> *"Perfection is achieved not when there is nothing left to add, but when there is nothing left to take away."* — Antoine de Saint-Exupéry

Contributors and reviewers are expected to:
- **Guard the project's simplicity** as its most valuable feature
- **Resist scope creep** and architectural complexity
- **Choose boring technology** over clever solutions
- **Prefer deletion** over addition when in doubt

This manifesto is the constitution for Ponder. All code reviews, contributions, and technical decisions must align with its principles.

---

**Ponder is simple. We are here to keep it that way.**

---

*For implementation guidance, see:*
*   [CONTRIBUTING.md](./CONTRIBUTING.md)
*   [README.md](./README.md)