# Phase 6.4: Context Engineering

## Overview
Implement comprehensive context engineering for the saorsa-tui AI agent, including AGENTS.md discovery and merging, SYSTEM.md support for custom system prompts, context compaction strategies, skills system for on-demand capabilities, and prompt templates. This enables users to customize agent behavior, manage context growth, and extend functionality through discoverable context files.

## Tasks

### Task 1: Context Discovery System
**Files:**
- crates/saorsa-agent/src/context/mod.rs (new)
- crates/saorsa-agent/src/context/discovery.rs (new)
- crates/saorsa-agent/src/lib.rs (export context module)

**Description:**
Implement the file discovery system for AGENTS.md files across multiple locations with precedence rules: global (~/.saorsa-tui/), parent directories (walk up from CWD), and current working directory. Implement path walking, file existence checking, and precedence resolution (CWD > parent > global).

**Requirements:**
- ContextDiscovery struct with methods for finding files
- discover_agents_md() returning Vec<PathBuf> ordered by precedence
- discover_system_md() with same precedence rules
- Walk parent directories up to filesystem root or home
- Global location: ~/.saorsa-tui/AGENTS.md and ~/.saorsa-tui/SYSTEM.md
- Filter non-existent paths
- Deterministic ordering (highest precedence first)

**Tests:**
- Discovery finds global file when it exists
- Discovery finds CWD file with higher precedence than global
- Parent directory discovery walks up correctly
- Empty result when no files exist
- Precedence ordering is correct (CWD, parent, parent's parent, ..., global)

---

### Task 2: AGENTS.md Loading and Merging
**Files:**
- crates/saorsa-agent/src/context/agents.rs (new)
- crates/saorsa-agent/src/context/types.rs (new)

**Description:**
Load discovered AGENTS.md files and implement merging strategies. Support two modes: "replace" (only highest precedence file) and "append" (merge all files with separators). Parse front matter for merge directives. Handle file I/O errors gracefully. Return merged content as String ready for LLM context.

**Requirements:**
- AgentsContext struct with content: String field
- load_and_merge() accepting Vec<PathBuf> from discovery
- MergeStrategy enum: Replace, Append
- Front matter parsing: "---\nmerge: replace|append\n---"
- Default strategy: Append (most common use case)
- File read with proper error handling (anyhow::Context)
- Separator between merged files: "\n\n---\n\n"

**Tests:**
- Single file loads correctly
- Append strategy merges multiple files in order
- Replace strategy uses only highest precedence
- Front matter overrides default strategy
- File read errors are propagated with context
- Empty file list returns empty content

---

### Task 3: SYSTEM.md Support
**Files:**
- crates/saorsa-agent/src/context/system.rs (new)

**Description:**
Implement SYSTEM.md discovery and loading with similar precedence rules as AGENTS.md. Support two modes: "replace" (override default system prompt entirely) and "append" (add to default system prompt). Parse front matter for mode directives. Integrate with agent config to inject custom system prompts.

**Requirements:**
- SystemContext struct with content: String field
- load_and_merge() similar to AgentsContext
- SystemMode enum: Replace, Append
- Front matter: "---\nmode: replace|append\n---"
- Default mode: Append (safer, preserves base behavior)
- Append adds custom content after default system prompt
- Replace completely overrides system prompt

**Tests:**
- Replace mode overrides default prompt
- Append mode combines default + custom
- Front matter parsing works
- Multiple SYSTEM.md files handled correctly
- Empty content handled gracefully

---

### Task 4: Context Types and Integration
**Files:**
- crates/saorsa-agent/src/context/types.rs (update)
- crates/saorsa-agent/src/config.rs (integrate context)
- crates/saorsa-agent/src/agent.rs (use context in message building)

**Description:**
Define unified context types and integrate them into agent configuration and message building. Create ContextBundle containing agents, system, and user context. Update AgentConfig to hold ContextBundle. Modify message building to inject context at appropriate points in conversation.

**Requirements:**
- ContextBundle struct with agents, system, user fields
- ContextBuilder for fluent construction
- Integration into AgentConfig
- Message building injects AGENTS.md before user messages
- System prompt building uses SYSTEM.md if present
- Serialization support for session persistence (Serialize, Deserialize)

**Tests:**
- ContextBundle construction and field access
- Integration with AgentConfig
- Message building includes context correctly
- Serialization round-trip
- Empty context bundles handled correctly

---

### Task 5: Context Compaction Strategy
**Files:**
- crates/saorsa-agent/src/context/compaction.rs (new)
- crates/saorsa-agent/src/context/token_counter.rs (new)

**Description:**
Implement context compaction strategies to manage growing conversation history. Add token counting for messages using tiktoken-rs or approximate tokenization. Implement compaction strategies: truncate oldest, summarize blocks, preserve important markers (tool calls, errors). Track compaction statistics.

**Requirements:**
- CompactionStrategy enum: TruncateOldest, SummarizeBlocks, Hybrid
- TokenCounter using tiktoken-rs (cl100k_base encoding)
- count_tokens(message) for individual messages
- CompactionConfig with max_tokens, preserve_recent_count
- compact() function taking messages and config, returning compacted Vec<Message>
- Preserve system messages, tool definitions, recent N messages
- Statistics: original_tokens, compacted_tokens, messages_removed

**Tests:**
- Token counting matches expected values
- TruncateOldest removes oldest messages first
- Recent messages always preserved
- System messages never removed
- Compaction achieves target token count
- Statistics tracked correctly

---

### Task 6: /compact Command
**Files:**
- crates/saorsa-agent/src/context/command.rs (new)
- crates/saorsa-app/src/commands.rs (integrate /compact)

**Description:**
Implement /compact slash command for manual context compaction and auto-compaction triggers. Add CompactCommand with options (--strategy, --max-tokens, --dry-run). Implement auto-compaction trigger when context exceeds threshold. Provide UI feedback showing before/after token counts and messages removed.

**Requirements:**
- /compact command with clap-based argument parsing
- --strategy flag (truncate, summarize, hybrid)
- --max-tokens flag (default from config)
- --dry-run flag (show what would be removed without doing it)
- Auto-compaction trigger at 80% of max context window
- UI feedback with CompactionResult (before/after counts)
- Update session after compaction

**Tests:**
- Command parsing with various flags
- Dry-run doesn't modify messages
- Auto-compaction triggers at threshold
- Manual compaction with different strategies
- UI feedback includes correct statistics
- Session updated after compaction

---

### Task 7: Skills System Foundation
**Files:**
- crates/saorsa-agent/src/skills/mod.rs (new)
- crates/saorsa-agent/src/skills/registry.rs (new)
- crates/saorsa-agent/src/skills/types.rs (new)

**Description:**
Implement the skills system foundation for on-demand capabilities. Define Skill struct with name, description, triggers, and content. Implement skill discovery (similar to AGENTS.md) in ~/.saorsa-tui/skills/ and project .saorsa-tui/skills/. Create SkillRegistry for loading and activating skills based on user requests or automatic triggers.

**Requirements:**
- Skill struct with name, description, triggers, content
- SkillRegistry managing loaded skills
- discover_skills() finding .md files in skills directories
- Skill file format: front matter + markdown content
- Front matter: name, description, triggers (keywords/patterns)
- load_skill(path) parsing skill files
- activate_skill(name) injecting skill content into context
- list_skills() for UI display

**Tests:**
- Skill file parsing with front matter
- Discovery finds skills in multiple locations
- SkillRegistry loads and stores skills
- Skill activation adds content to context
- List skills returns all loaded skills
- Invalid skill files handled gracefully

---

### Task 8: Prompt Templates
**Files:**
- crates/saorsa-agent/src/templates/mod.rs (new)
- crates/saorsa-agent/src/templates/engine.rs (new)
- crates/saorsa-agent/src/templates/builtins.rs (new)

**Description:**
Implement prompt template system with variable substitution and conditionals. Use simple template syntax: {{variable}} for substitution, {{#if variable}}...{{/if}} for conditionals. Create built-in templates for common tasks (code review, debugging, documentation). Support user-defined templates in ~/.saorsa-tui/templates/.

**Requirements:**
- TemplateEngine with render(template, context) method
- Variable substitution: {{variable}} → value from context
- Conditionals: {{#if var}}text{{/if}} and {{#unless var}}text{{/unless}}
- Template context as HashMap<String, String>
- Built-in templates: code_review, debug, document, test, refactor
- User template discovery in ~/.saorsa-tui/templates/*.md
- Template validation on load
- Error handling for missing variables

**Tests:**
- Variable substitution works correctly
- If/unless conditionals evaluate properly
- Nested conditionals handled
- Built-in templates render without errors
- User templates loaded and rendered
- Missing variable errors are clear
- Template file discovery works

---

## Integration

After all tasks complete, the context engineering system should:
1. Automatically discover and load AGENTS.md and SYSTEM.md from appropriate locations
2. Merge context files according to precedence and merge strategies
3. Support manual and automatic context compaction to manage token limits
4. Enable skills system for on-demand capability injection
5. Provide prompt templates for common tasks
6. All context customization persists across sessions
7. Zero compilation warnings, zero test failures

## Dependencies

New dependencies to add to saorsa-agent/Cargo.toml:
- tiktoken-rs = "0.5" (for token counting)

Existing dependencies sufficient for other features (serde, anyhow, tokio).
