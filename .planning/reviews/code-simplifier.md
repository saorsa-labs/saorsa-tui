# Code Simplification Review
**Date**: 2026-02-08
**Mode**: gsd (phase review)
**Scope**: Last commit (HEAD~1..HEAD)

## Summary

Reviewed 28 modified Rust files from the last commit, focusing on:
- `saorsa-agent/src/config/import.rs` (946 new lines) - Configuration import from ~/.pi and ~/.claude
- `saorsa-agent/src/tools/web_search.rs` (534 new lines) - DuckDuckGo web search tool
- `saorsa-agent/src/cost.rs` (192 new lines) - LLM cost tracking
- `saorsa/src/main.rs` (242 lines modified) - Main application with session management
- Various config modules (auth, models, settings, paths)

## Findings

### MEDIUM Priority

1. **File: `saorsa-agent/src/config/import.rs:269-341`**
   - **Pattern**: Duplicate logic in `import_skills()` function
   - **Issue**: The function handles two different file patterns (`.md` files and `SKILL.md` subdirs) within a single loop with repeated `copy_skill_file()` calls
   - **Suggestion**: Extract file pattern detection into a helper function that returns `Option<(source_path, target_name)>`, simplifying the main loop

2. **File: `saorsa-agent/src/config/import.rs:351-397`**
   - **Pattern**: Similar structure to `import_skills()` but slightly different
   - **Issue**: `import_agents()` and `import_skills()` share substantial structural similarity but are separate functions
   - **Suggestion**: Consider extracting a generic `import_files()` function with a predicate/transformer closure to reduce duplication

3. **File: `saorsa/src/main.rs:319-380`**
   - **Pattern**: Duplicated code in `CycleModel` and `CycleModelBackward` handlers
   - **Issue**: Both branches contain nearly identical logic (80+ lines each) for API key resolution and model switching
   - **Suggestion**: Extract a helper function `switch_to_model(state, new_model, &mut api_key, &mut provider_kind, &mut model)` to eliminate ~160 lines of duplication

4. **File: `saorsa-agent/src/tools/web_search.rs:80-137`**
   - **Pattern**: Complex string parsing with nested conditionals
   - **Issue**: The `parse_ddg_html()` function has deeply nested logic for extracting URLs, titles, and snippets
   - **Suggestion**: Extract URL extraction into a dedicated function `extract_result_url()` that handles both backward and forward href searches, improving readability

5. **File: `saorsa/src/main.rs:211-258`**
   - **Pattern**: Nested if-let-else chain for session loading
   - **Issue**: Three-level conditional with repeated `state.add_system_message()` patterns
   - **Suggestion**: Use early returns or match expressions instead of nested if-else to reduce indentation

### LOW Priority

6. **File: `saorsa-agent/src/cost.rs:40-64`**
   - **Pattern**: Chained `and_then` with inline calculations
   - **Issue**: The `track()` method has complex inline cost calculation logic
   - **Suggestion**: Extract cost calculation into a separate `calculate_cost(model_info, usage) -> f64` function for clarity

7. **File: `saorsa-agent/src/config/import.rs:108-152`**
   - **Pattern**: Repeated error handling pattern
   - **Issue**: `import_pi_auth()`, `import_pi_models()`, and `import_pi_settings()` all use identical early-return patterns
   - **Observation**: The pattern is reasonable, but could benefit from a macro or helper if this pattern expands

8. **File: `saorsa-agent/src/tools/web_search.rs:259-292`**
   - **Pattern**: Manual whitespace normalization
   - **Issue**: `clean_text()` manually iterates with state tracking for whitespace
   - **Suggestion**: Could use `text.split_whitespace().collect::<Vec<_>>().join(" ")` for simpler logic (though current approach is more efficient)

9. **File: `saorsa/src/main.rs:533-556`**
   - **Pattern**: Match expression with repetitive string truncation
   - **Issue**: `add_message_to_state()` has duplicate truncation logic in two branches
   - **Suggestion**: Extract `truncate_display(text, max_len)` helper

## Simplification Opportunities

### High Impact (Recommended)

1. **Consolidate model switching logic** (`saorsa/src/main.rs`)
   - Extract common code from `CycleModel` and `CycleModelBackward` handlers
   - **Savings**: ~160 lines → ~50 lines (110 lines eliminated)
   - **Complexity**: Reduces maintenance burden significantly

2. **Simplify config import functions** (`saorsa-agent/src/config/import.rs`)
   - Create generic file import helper to reduce duplication between `import_skills()` and `import_agents()`
   - **Savings**: ~90 lines → ~60 lines (30 lines eliminated)
   - **Complexity**: Improves testability and reduces error-prone duplication

3. **Refactor HTML parsing** (`saorsa-agent/src/tools/web_search.rs`)
   - Extract URL/title/snippet extraction into separate focused functions
   - **Savings**: Minimal line reduction, but significant readability improvement
   - **Complexity**: Makes the parsing logic much easier to test and debug

### Medium Impact (Consider)

4. **Extract cost calculation logic** (`saorsa-agent/src/cost.rs`)
   - Separate calculation from tracking
   - **Benefit**: Easier to test cost formulas independently

5. **Simplify session loading flow** (`saorsa/src/main.rs`)
   - Use match or early returns instead of nested if-let-else
   - **Benefit**: Reduces indentation depth from 4 to 2 levels

## Code Quality Assessment

**Strengths:**
- Excellent error handling with proper `Result` types throughout
- Comprehensive test coverage (all new modules have substantial test suites)
- Clear separation of concerns (auth, models, settings as separate modules)
- Consistent use of `#[allow(clippy::unwrap_used)]` in test modules only
- Good documentation with module-level and function-level doc comments

**Areas for Improvement:**
- Some functions exceed comfortable length (100+ lines)
- Duplication in model cycling logic is significant
- Complex parsing logic could benefit from better decomposition

**Patterns Observed:**
- Heavy use of early returns for error cases (good)
- Consistent error type conversion with descriptive messages (good)
- Preference for explicit code over compact/clever solutions (good)
- Some functions accumulate multiple responsibilities

## Grade: B+

**Rationale:**

The code demonstrates strong fundamentals with excellent error handling, comprehensive testing, and clear documentation. The primary issues are:

1. **Duplication** in the model switching logic (major)
2. **Complexity** in some parsing and import functions (moderate)
3. **Function length** in a few cases (minor)

These issues are addressable through targeted refactoring without requiring architectural changes. The code follows project standards well (zero unwrap/expect in production, proper error types, good test coverage).

**Deductions:**
- -0.5: Significant duplication in model cycling handlers
- -0.5: Complex nested logic in HTML parsing and session loading
- -0.5: Some functions exceed 100 lines (import_skills, parse_ddg_html)

**What would make this an A:**
- Extract model switching logic to eliminate ~110 lines of duplication
- Decompose `parse_ddg_html()` into smaller, focused functions
- Simplify session loading flow with match expressions or early returns

## Recommendations

1. **Immediate**: Extract model switching helper in `main.rs` (high impact, low risk)
2. **Short-term**: Refactor HTML parsing in `web_search.rs` (improves testability)
3. **Long-term**: Consider generic import helper if more import sources are added

All findings are refinement opportunities, not critical issues. The code is production-ready as-is.
