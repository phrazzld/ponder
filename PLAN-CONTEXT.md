-   **[Refactor]**: Enforce Strict Module Boundaries and Feature-Based Organization
    -   **Complexity**: Medium
    -   **Rationale**: **CRITICAL** for "Modularity is Mandatory" and "Architecture Guidelines (Package/Module Structure)" (Dev Philosophy). A clear, feature-oriented module structure improves maintainability, navigability, and separation of concerns, making the codebase easier to understand and evolve.
    -   **Expected Outcome**: The `src/` directory and module structure are reviewed and refactored to strictly adhere to "Package by Feature, Not Type." Modules exhibit high cohesion, minimal public APIs, and clear responsibilities aligned with distinct application features or domains (e.g., `cli`, `config`, `journal_operations`, `error_handling`).
    -   **Dependencies**: None. Facilitates many other refactoring and feature development tasks.


