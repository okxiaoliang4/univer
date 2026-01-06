Summarize the chat messages, and then commit related files. Can be split into multiple commits.

**Critical Requirements:**
1. Each commit must be an independent, working version - the code must compile, pass linting, and be functional without errors
2. No commit should contain errors, broken code, or incomplete implementations
3. When splitting into multiple commits, each commit must still be independently usable:
   - Each commit should be a complete, working unit
   - Dependencies between commits should be minimal and clearly documented
   - Test that each commit can be checked out and run successfully
4. Commit messages must be concise, clear, and follow conventional commit format
5. Before committing, verify:
   - Code compiles without errors
   - ESLint passes without errors
   - No TypeScript errors
   - Related tests pass (if applicable)

**Commit Message Format:**
- Use conventional commit format: `type(scope): description`
- Types: feat, fix, refactor, docs, style, test, chore, etc.
- Be specific about what changed and why
