# Phase 6.3: Session Management

## Overview
Implement comprehensive session management for the fae AI agent, including tree-structured storage, auto-save, session continuation, branching/forking, bookmarks, and export capabilities. This enables users to maintain conversation history, resume work across sessions, and share their interactions.

## Tasks

### Task 1: Session Types and Core Structures
**Files:**
- crates/fae-agent/src/session/mod.rs (new)
- crates/fae-agent/src/session/types.rs (new)
- crates/fae-agent/src/lib.rs (export session module)

**Description:**
Define the core session data structures including SessionId (UUID-based), SessionMetadata (created, modified, title, tags), SessionNode (tree structure with parent/children relationships), and Message types (user, assistant, tool call, tool result). Implement basic constructors and utility methods.

**Requirements:**
- SessionId type wrapping UUID v4
- SessionMetadata with timestamps, title, description, tags
- SessionNode with tree relationships (parent_id, child_ids)
- Message enum covering all conversation types
- All types derive Debug, Clone, Serialize, Deserialize
- Use chrono for timestamps

**Tests:**
- Session ID generation uniqueness
- SessionNode parent-child relationship operations
- Message type serialization round-trips
- Metadata clone and equality

---

### Task 2: Filesystem Layout and Serialization
**Files:**
- crates/fae-agent/src/session/storage.rs (new)
- crates/fae-agent/src/session/path.rs (new)

**Description:**
Define the filesystem layout for session storage (~/.fae/sessions/{session_id}/) with manifest.json (metadata), messages/ (individual message files), tree.json (relationship graph), and bookmarks.json. Implement serialization/deserialization with serde_json. Handle directory creation, file I/O, and basic error cases.

**Requirements:**
- SessionStorage struct managing base path
- manifest.json with all metadata fields
- messages/ directory with {index}-{type}.json naming
- tree.json for parent/child/sibling relationships
- Atomic writes (write to temp, rename)
- XDG base directory support (~/.fae or $XDG_DATA_HOME/fae)

**Tests:**
- Directory creation on first use
- Message serialization to individual files
- Manifest read/write round-trip
- Tree structure persistence
- Path construction for various session IDs

---

### Task 3: Auto-Save Implementation
**Files:**
- crates/fae-agent/src/session/autosave.rs (new)
- crates/fae-agent/src/agent.rs (integrate auto-save)

**Description:**
Implement automatic session saving after each message exchange. Add AutoSaveConfig with interval/batch settings. Implement debounced saves (coalesce rapid changes) and incremental saves (append-only message log). Handle save failures gracefully with retry logic.

**Requirements:**
- AutoSaveConfig with save_interval, max_batch_size
- Debounce rapid message additions (500ms default)
- Incremental message append (don't rewrite entire session)
- Background save task (tokio::spawn)
- Retry on transient failures (3 attempts)
- Session dirty flag tracking

**Tests:**
- Debouncing coalesces rapid saves
- Incremental save appends only new messages
- Retry logic on simulated I/O error
- Session state persists after auto-save
- No data loss on rapid message additions

---

### Task 4: Continue and Resume Functionality
**Files:**
- crates/fae-agent/src/session/resume.rs (new)
- crates/fae-cli/src/args.rs (add -c/--continue and -r/--resume flags)
- crates/fae-app/src/main.rs (handle continue/resume on startup)

**Description:**
Implement session continuation (continue last active session) and resume (resume specific session by ID prefix). Add CLI flags -c/--continue and -r <id>/--resume <id>. Implement last-active tracking (update on save), session ID prefix matching (shortest unique), and session restoration (load messages, rebuild state).

**Requirements:**
- -c/--continue flag continues most recent session
- -r <prefix>/--resume <prefix> resumes by session ID prefix
- last_active timestamp in manifest
- Prefix matching finds shortest unique match (error on ambiguous)
- Load all messages and rebuild agent state
- Ephemeral mode (--ephemeral, no persistence)

**Tests:**
- Continue loads most recent session
- Resume with full ID works
- Resume with prefix works (shortest unique)
- Resume errors on ambiguous prefix
- Ephemeral mode doesn't create session files
- Restored session has all messages

---

### Task 5: Tree Command and Navigation
**Files:**
- crates/fae-agent/src/session/tree.rs (new)
- crates/fae-app/src/commands/tree.rs (new)
- crates/fae-app/src/commands/mod.rs (add tree command)

**Description:**
Implement /tree command showing session hierarchy with ASCII art tree rendering. Support navigation (select node, show messages), filtering (by date, tag), and statistics (message count, token estimate). Use tree-drawing characters (├──, └──, │) for visual hierarchy.

**Requirements:**
- /tree command with no args shows full hierarchy
- /tree <id> shows specific session subtree
- ASCII tree rendering with proper indentation
- Show: session ID (prefix), title, message count, last active
- Highlight current session
- Interactive mode: arrow keys to navigate, Enter to switch

**Tests:**
- Single session renders correctly
- Multi-level tree renders with correct lines
- Current session highlighted
- Empty tree shows helpful message
- Filtering by date range works

---

### Task 6: Branching and Forking
**Files:**
- crates/fae-agent/src/session/branch.rs (new)
- crates/fae-app/src/commands/fork.rs (new)
- crates/fae-app/src/commands/mod.rs (add /fork command)

**Description:**
Implement session forking (/fork [title]) to create divergent conversation branches. Copy current session messages up to present point, create new session with new ID, establish parent-child relationship in tree. Support automatic forking on re-editing past messages.

**Requirements:**
- /fork [title] creates branch from current point
- New session has unique ID
- Parent/child relationship recorded in tree.json
- Messages copied up to fork point
- Optional title for forked session
- Auto-fork on message edit (re-generation from middle)

**Tests:**
- Fork creates new session with correct parent
- Forked session has messages up to fork point
- Tree structure reflects fork relationship
- Multiple forks from same parent work
- Auto-fork on message edit creates branch

---

### Task 7: Bookmarks System
**Files:**
- crates/fae-agent/src/session/bookmark.rs (new)
- crates/fae-app/src/commands/bookmark.rs (new)
- crates/fae-app/src/commands/mod.rs (add bookmark commands)

**Description:**
Implement session bookmarking for quick access to important sessions. Add /bookmark [name], /bookmarks list, /jump <name> commands. Store bookmarks in ~/.fae/bookmarks.json with name → session_id mapping. Support bookmark rename and deletion.

**Requirements:**
- /bookmark [name] bookmarks current session (auto-generate if no name)
- /bookmarks shows all bookmarks with titles
- /jump <name> switches to bookmarked session
- Bookmark persistence in bookmarks.json
- /bookmark --delete <name> removes bookmark
- /bookmark --rename <old> <new> renames bookmark

**Tests:**
- Bookmark creation with custom name
- Auto-generated bookmark names are unique
- Jump loads correct session
- Bookmark deletion works
- Rename preserves session reference
- List shows all bookmarks sorted

---

### Task 8: HTML Export and Gist Sharing
**Files:**
- crates/fae-agent/src/session/export.rs (new)
- crates/fae-app/src/commands/export.rs (new)
- crates/fae-app/src/commands/share.rs (new)
- crates/fae-app/src/commands/mod.rs (add export and share commands)

**Description:**
Implement session export to HTML with syntax highlighting and styled formatting. Add /export [file] command generating standalone HTML (embedded CSS). Implement /share command uploading to GitHub gist (anonymous or authenticated). Include metadata header, message timestamps, and code block formatting.

**Requirements:**
- /export [file] generates standalone HTML
- HTML includes: metadata header, timestamps, message formatting
- Code blocks with syntax highlighting (syntect)
- Embedded CSS (no external dependencies)
- /share uploads to GitHub gist (octocrab)
- Share returns gist URL
- Support --public and --private flags for gist

**Tests:**
- HTML export creates valid file
- Exported HTML opens in browser
- Code blocks have syntax classes
- Timestamps formatted correctly
- Share creates gist (mock GitHub API)
- Private/public flags respected

---

## Integration Notes

- Add `session` feature flag to fae-agent Cargo.toml (default enabled)
- All session storage is in `~/.fae/sessions/` or `$XDG_DATA_HOME/fae/sessions/`
- Session IDs are UUID v4 displayed as 8-char prefix for user convenience
- Auto-save runs on background tokio task with debouncing
- Commands are integrated into main app command dispatch

## Dependencies

New dependencies needed:
- `uuid` (with v4 feature)
- `chrono` (already in use)
- `serde` and `serde_json` (already in use)
- `tokio` (already in use)
- `directories` (XDG base directory support)
- `syntect` (syntax highlighting for HTML export)
- `octocrab` (GitHub API client for gist sharing)

## Success Criteria

- Users can continue previous sessions with -c
- Users can resume specific sessions with -r <id>
- /tree shows visual session hierarchy
- /fork creates conversation branches
- /bookmark enables quick session access
- /export creates shareable HTML
- /share uploads to GitHub gist
- All session data persists correctly
- Zero compilation warnings
- All tests pass
