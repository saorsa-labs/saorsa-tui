# NEXT TASK: Phase 6.1 Task 6

## Status
- **Current Task**: 6 of 8
- **Phase**: 6.1 (Additional Providers)
- **Milestone**: 6 (Full Agent Features)
- **Previous Task**: Task 5 (Ollama Provider) — COMPLETE and REVIEWED (PASS)

## Task 6: OpenAI-Compatible Provider

**Objective**: Implement a generic OpenAI-compatible provider that works with Azure OpenAI, Groq, Cerebras, xAI, OpenRouter, Mistral, and other OpenAI-compatible APIs.

### Files to Create/Modify
- **CREATE**: `crates/fae-ai/src/openai_compat.rs`
- **MODIFY**: `crates/fae-ai/src/openai.rs` (make helpers `pub(crate)`)
- **MODIFY**: `crates/fae-ai/src/lib.rs` (export new module)

### Implementation Requirements

See `.planning/PLAN-phase-6.1.md` Task 6 for full specification.

Key points:
1. Reuse OpenAI request/response logic from `openai.rs`
2. Support custom base URLs, auth headers, extra headers
3. Factory functions for common providers (Azure, Groq, OpenRouter, Mistral, Cerebras)
4. Builder pattern for configuration
5. Comprehensive tests (10+)

### Next Steps for Autonomous Execution
1. Implement Task 6 (this task)
2. Run /gsd-review (11-agent parallel review)
3. Fix any findings with code-fixer
4. Commit task completion
5. Continue to Task 7

**DO NOT STOP** — Phase 6.1 has 8 tasks, only 5 are complete.
