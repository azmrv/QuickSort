---
name: beans
title: Beans Skill
description: "Primary Beans VS Code workspace skill. Use for Beans issue tracker workflows in this workspace: finding or creating beans, updating status/type/priority/parent/blocking relationships, maintaining bean todo checklists, choosing the right Git Flow branch name for a bean, decomposing epics, and using the VS Code extension commands or MCP tools instead of ad-hoc todo lists or generic issue workflows."
---

<!-- markdownlint-disable MD025 MD060 -->

# Beans Skill

This skill is the primary Beans VS Code skill for this workspace. It is generated into `.agents/skills/beans-vscode/SKILL.md` and should be treated as the main source of Beans workflow guidance for Copilot in this repository.

## Extremely important

- **Always use beans instead of TodoWrite to manage your work and tasks.**
- **Always use beans instead of writing ad-hoc todo lists in chat, notes, or scratch files.**
- **Before starting work, first check whether a relevant bean already exists. If not, create one and set it to `in-progress`.**
- **Keep the bean body current while you work, especially the `## Todo` checklist.**
- **When committing, include both code changes and bean changes when they changed together.**

## Interface priority

**Always use the highest available interface. Never skip levels for convenience.**

1. **Beans VS Code extension commands/UI** (`beans.*`) — preferred whenever the IDE is available
2. **Beans MCP tools** — preferred for programmatic workflows when the UI is not in focus
3. **`@beans` chat participant** — for guidance, summaries, and structured workflows
4. **CLI** — only when extension, chat, and MCP are all genuinely unavailable; see CLI constraints

## Core rules (always enforced)

- **Never start work without a bean.** Find a relevant bean or create one first. No exceptions.
- **Prefer extension commands/UI first, then MCP tools.** Do not reach for the CLI unless all other interfaces are unavailable.
- **The bean is the source of truth.** Keep status, type, priority, parent, blocking, and body current.
- **Create a branch before writing code.** Use a Git Flow topic prefix aligned to the bean type and urgency.
- **Record branch and PR in bean frontmatter.** Add `branch:` and `pr:` as soon as they exist; commit immediately.
- **Commit beans promptly.** Commit after create/edit unless building an epic hierarchy (batch commit everything together).
- **Track todos in the bean body.** Maintain a `## Todo` checklist; update and commit after every completed step.

## Git Flow branch mapping

Use standard Git Flow topic prefixes and align them to the bean type:

- **`feature/`** — default for `feature`, `task`, and most implementation work under an `epic`
- **`bugfix/`** — default for `bug` beans that are not urgent production fixes
- **`hotfix/`** — only for urgent production or release-blocking `bug` beans that must ship immediately
- **`support/`** — for `milestone`-driven maintenance, release-support work, or cross-cutting operational follow-up where `feature/` is not a fit
- **`develop`** — integration base branch, **not** a topic-branch prefix to create per bean

Recommended mapping by bean type:

- `feature` → `feature/<bean-id>-<slug>`
- `task` → `feature/<bean-id>-<slug>` unless the task is clearly a bug fix or support-line maintenance
- `bug` → `bugfix/<bean-id>-<slug>` by default, or `hotfix/<bean-id>-<slug>` for urgent production fixes
- `epic` → usually **do not work directly on the epic**; create child beans and branch from the child bean type instead
- `milestone` → usually coordination-only; if direct branch work is required, prefer `support/<bean-id>-<slug>`

## Agent constraints (must-follow)

- **Always create or switch to the bean's branch before editing code or files.**
  - If a branch exists for the bean, checkout it first. If not, create and push a new branch following the Git Flow mapping above.
  - Record the branch in the bean frontmatter immediately after creating or checking out the branch.

- **Do not keep an internal agent todo list.**
  - All task state and subtasks must live in the bean's `## Todo` checklist in the bean body.
  - The agent must update the bean `## Todo` via extension commands or MCP tools and commit the bean to persist progress.

- **Do not create or edit bean files by hand.**
  - Use the extension commands or MCP tools for bean creation and edits; never write `.md` bean files directly to the repository.
  - The agent must not add custom frontmatter keys except `branch` and `pr`.

## Starting work on a bean

1. Orient yourself first: use `beans.search`, the search view, or `@beans /next` / `@beans /search <keywords>` to find existing work.
2. If none found, create one with the extension or MCP: set title, type, priority, and body/description.
3. Set status to `in-progress`.
4. Create a Git Flow topic branch aligned to the bean type and urgency.
5. Edit the bean to add `branch: <name>` to its frontmatter.
6. Commit the bean file with message: `chore(beans): start <bean-id> — set branch`.

## During work

- Add a `## Todo` checklist to the bean body on first edit.
- After each completed step: check off the item, save the bean, and commit the bean with related code changes.
- If scope grows, create child beans and link them with parent relationships instead of overloading one bean.

## Completing or scrapping a bean

**Complete:**

1. Add `## Summary of Changes` to the bean body.
2. Set status to `completed`.
3. Commit: `chore(beans): complete <bean-id>`.

**Scrap:**

1. Add `## Reasons for Scrapping` to the bean body.
2. Set status to `scrapped`.
3. Commit: `chore(beans): scrap <bean-id>`.

## VS Code extension command reference

### Bean operations

| Command                 | Purpose                                                   |
| ----------------------- | --------------------------------------------------------- |
| `beans.create`          | Create a new bean                                         |
| `beans.view`            | Open bean in Details sidebar panel                        |
| `beans.edit`            | Edit bean body/frontmatter in editor                      |
| `beans.delete`          | Delete a bean (only draft or scrapped)                    |
| `beans.setStatus`       | Change status (todo → in-progress → completed / scrapped) |
| `beans.setType`         | Change type (task / feature / bug / epic / milestone)     |
| `beans.setPriority`     | Change priority (critical/high/normal/low/deferred)       |
| `beans.setParent`       | Link a bean to a parent epic or milestone                 |
| `beans.removeParent`    | Remove parent link                                        |
| `beans.editBlocking`    | Edit blocking/blocked-by relationships                    |
| `beans.reopenCompleted` | Reopen a completed bean                                   |
| `beans.reopenScrapped`  | Reopen a scrapped bean                                    |
| `beans.copyId`          | Copy bean ID to clipboard                                 |

### Search and navigation

| Command                   | Purpose                                 |
| ------------------------- | --------------------------------------- |
| `beans.search`            | Full-text search across all beans       |
| `beans.filter`            | Filter tree by status/type/priority/tag |
| `beans.sort`              | Change tree sort mode                   |
| `beans.refresh`           | Force reload from disk                  |
| `beans.searchView.filter` | Filter search results                   |
| `beans.searchView.clear`  | Clear search filters and query          |
| `beans.details.back`      | Navigate back in Details view history   |

### AI and workflow

| Command                  | Purpose                                         |
| ------------------------ | ----------------------------------------------- |
| `beans.copilotStartWork` | Open Copilot Chat with a bean workflow template |

### Configuration and help

| Command                        | Purpose                                        |
| ------------------------------ | ---------------------------------------------- |
| `beans.init`                   | Initialize Beans in an uninitialized workspace |
| `beans.openConfig`             | Open `.beans.yml` configuration file           |
| `beans.openExtensionSettings`  | Open VS Code extension settings                |
| `beans.showOutput`             | Show the Beans extension output/log channel    |
| `beans.openUserGuide`          | Open user guide documentation                  |
| `beans.openAiFeaturesGuide`    | Open AI features documentation                 |
| `beans.openFirstMalformedBean` | Navigate to first malformed bean file          |

## Available MCP tools

Use MCP when automating outside the VS Code UI. Prefer these tool groups in this order: discover → inspect → mutate.

| Tool                   | Purpose                                                            |
| ---------------------- | ------------------------------------------------------------------ |
| `beans_query`          | Query/list/filter/search/sort/ready/graphql/llm-context operations |
| `beans_view`           | View one or more beans with full details                           |
| `beans_create`         | Create one bean                                                    |
| `beans_bulk_create`    | Create multiple beans, optionally under a shared parent            |
| `beans_update`         | Update metadata, relationships, or body                            |
| `beans_bulk_update`    | Update multiple beans in one workflow                              |
| `beans_edit`           | Metadata-only helper                                               |
| `beans_complete_tasks` | Complete checklist items in the bean body                          |
| `beans_reopen`         | Reopen completed/scrapped beans                                    |
| `beans_archive`        | Archive completed/scrapped beans when explicitly requested         |
| `beans_delete`         | Delete draft/scrapped beans                                        |
| `beans_bean_file`      | Supported `.beans` file read/edit helpers                          |
| `beans_output`         | Read Beans extension logs                                          |

## Chat participant (`@beans`) guidance

`@beans` is the human-facing conversational interface. Use it for orientation, guidance, and structured workflows. It reads beans and returns natural-language responses; it does **not** mutate state.

Useful slash commands:

- `/summary`
- `/priority`
- `/stale`
- `/next`
- `/create`
- `/search <query>`
- `/commit`

## MCP tool guidance

Use MCP tools when automation is needed outside the VS Code UI:

1. **Default workflow** — discover with `beans_query` (`ready`, `refresh`, `filter`, `search`, `sort`, `llm_context`), inspect with `beans_view`, mutate with `beans_update`, use `beans_complete_tasks` for checklist work, then archive only when explicitly requested.
2. **Read context first** — prefer `beans_query` for `llm_context`, `refresh`, `filter`, `search`, `sort`, `ready`, and GraphQL operations; use `beans_view` when you need full bean payloads.
3. **Mutate with explicit intent** — prefer `beans_update` for most metadata/body changes; use `beans_create` for single items and `beans_bulk_create` / `beans_bulk_update` when decomposing epics or moving batches.
4. **Use file/log helpers intentionally** — `beans_bean_file` is the supported path for `.beans` file access, and `beans_output` is the supported path for log access.
5. **Destructive actions** — `beans_delete` and `beans_archive` require explicit user confirmation unless the user already requested them.
6. Prefer MCP over the CLI for any scripted or agent-driven operation.

### MCP gotchas and field guidance

- `beans_update` supports `status`, `type`, `priority`, `parent`, `clearParent`, `blocking`, `blockedBy`, `body`, `bodyAppend`, `bodyReplace`, and optional optimistic concurrency via `ifMatch`.
- `beans_update` cannot combine `body` with `bodyAppend` or `bodyReplace` in the same request.
- `beans_create` prefers `body`; `description` remains only a deprecated alias.
- Batch tools are **best-effort**, so callers must inspect per-item results instead of assuming atomic success.
- `beans_delete` only allows `draft` or `scrapped` beans unless force-delete is explicitly intended.
- `beans_reopen` requires the current status to match the reopen target (`completed` or `scrapped`).
- Prefer `beanId` (not `id`) for MCP updates.
- Use `ifMatch` with the bean `etag` from `beans_view` when concurrent edits are possible.

### Relationship semantics

- **Parent** — hierarchy (`milestone → epic → feature → task/bug`)
- **Blocking** — this bean blocks another bean
- **BlockedBy** — this bean depends on another bean; prefer this when modeling new dependencies

### Query and GraphQL examples

- Actionable work: `beans_query` with `operation: "ready"`
- Full refresh: `beans_query` with `operation: "refresh"`
- Search by text: `beans_query` with `operation: "search"`
- Advanced queries/mutations: `beans_query` with `operation: "graphql"`

## Planning mode: epic decomposition

When the user asks to plan an epic or decompose a feature:

### Step 1 — Clarify

Ask (or confirm from context):

- What is the end goal?
- Are there known constraints?
- What is the definition of done?
- Does an epic/milestone bean already exist?

### Step 2 — Create the epic bean (if needed)

Use the extension or MCP to create the parent bean first.

### Step 3 — Propose the issue map

Present a compact checklist for approval before creating anything:

```text
- [ ] <title> — type=<task|feature|bug>, priority=<①–⑤>, depends_on=<bean-ids or none>
```

Do not create any beans until the user approves the plan.

### Step 4 — Create child beans

Create approved child beans with the extension or MCP and link them to the parent.

### Step 5 — Return a summary

Reply with the created IDs and recommend the best bean to start first.

Example summary format:

```text
Epic:    <epic-id> <epic-title>
Created: <id> <title> [depends_on: <ids>]
         <id> <title>
Start with: <id> <title> — <one-line reason>
```

## CLI fallback (last resort only)

Use CLI only when extension, `@beans`, and MCP tools are all unavailable.

**Hard constraints:**

- Always use `--json`; never parse plain-text output.
- Never pass large body text as a shell argument; use GraphQL variables or pipe the query into `beans graphql`.
- One operation at a time; no chained destructive commands.
- Before using the CLI at all, verify the extension, chat participant, and MCP tooling are genuinely unavailable.

## Beans baseline

The following baseline is derived from `beans graphql --schema` and provides comprehensive guidance for working with beans in this project.

"""
A bean represents an issue/task in the beans tracker
"""
type Bean {
  """
  Unique identifier (NanoID)
  """
  id: ID!
  """
  Human-readable slug from filename
  """
  slug: String
  """
  Relative path from .beans/ directory
  """
  path: String!
  """
  Bean title
  """
  title: String!
  """
  Current status (draft, todo, in-progress, completed, scrapped)
  """
  status: String!
  """
  Bean type (milestone, epic, bug, feature, task)
  """
  type: String!
  """
  Priority level (critical, high, normal, low, deferred)
  """
  priority: String!
  """
  Tags for categorization
  """
  tags: [String!]!
  """
  Creation timestamp
  """
  createdAt: Time!
  """
  Last update timestamp
  """
  updatedAt: Time!
  """
  Markdown body content
  """
  body: String!
  """
  Content hash for optimistic concurrency control
  """
  etag: String!
  """
  Parent bean ID (optional, type-restricted)
  """
  parentId: String
  """
  IDs of beans this bean is blocking
  """
  blockingIds: [String!]!
  """
  IDs of beans that are blocking this bean (direct field)
  """
  blockedByIds: [String!]!
  """
  Beans that block this one (incoming blocking links)
  """
  blockedBy(filter: BeanFilter): [Bean!]!
  """
  Beans this one is blocking (resolved from blockingIds)
  """
  blocking(filter: BeanFilter): [Bean!]!
  """
  Parent bean (resolved from parentId)
  """
  parent: Bean
  """
  Child beans (beans with this as parent)
  """
  children(filter: BeanFilter): [Bean!]!
}
"""
Filter options for querying beans
"""
input BeanFilter {
  """
  Full-text search across slug, title, and body using Bleve query syntax.
  
  Examples:
  - "login" - exact term match
  - "login~" - fuzzy match (1 edit distance)
  - "login~2" - fuzzy match (2 edit distance)
  - "log*" - wildcard prefix
  - "\"user login\"" - exact phrase
  - "user AND login" - both terms required
  - "user OR login" - either term
  - "slug:auth" - search only slug field
  - "title:login" - search only title field
  - "body:auth" - search only body field
  """
  search: String
  """
  Include only beans with these statuses (OR logic)
  """
  status: [String!]
  """
  Exclude beans with these statuses
  """
  excludeStatus: [String!]
  """
  Include only beans with these types (OR logic)
  """
  type: [String!]
  """
  Exclude beans with these types
  """
  excludeType: [String!]
  """
  Include only beans with these priorities (OR logic)
  """
  priority: [String!]
  """
  Exclude beans with these priorities
  """
  excludePriority: [String!]
  """
  Include only beans with any of these tags (OR logic)
  """
  tags: [String!]
  """
  Exclude beans with any of these tags
  """
  excludeTags: [String!]
  """
  Include only beans with a parent
  """
  hasParent: Boolean
  """
  Include only beans with this specific parent ID
  """
  parentId: String
  """
  Include only beans that are blocking other beans
  """
  hasBlocking: Boolean
  """
  Include only beans that are blocking this specific bean ID
  """
  blockingId: String
  """
  Include only beans that are blocked by others (via incoming blocking links or blocked_by field)
  """
  isBlocked: Boolean
  """
  Include only beans that have explicit blocked-by entries
  """
  hasBlockedBy: Boolean
  """
  Include only beans blocked by this specific bean ID (via blocked_by field)
  """
  blockedById: String
  """
  Exclude beans that have a parent
  """
  noParent: Boolean
  """
  Exclude beans that are blocking other beans
  """
  noBlocking: Boolean
  """
  Exclude beans that have explicit blocked-by entries
  """
  noBlockedBy: Boolean
}
"""
Structured body modifications applied atomically.
Operations are applied in order: all replacements sequentially, then append.
If any operation fails, the entire mutation fails (transactional).
"""
input BodyModification {
  """
  Text replacements applied sequentially in array order.
  Each old text must match exactly once at the time it's applied.
  """
  replace: [ReplaceOperation!]
  """
  Text to append after all replacements.
  Appended with blank line separator.
  """
  append: String
}
"""
Input for creating a new bean
"""
input CreateBeanInput {
  """
  Bean title (required)
  """
  title: String!
  """
  Bean type (defaults to 'task')
  """
  type: String
  """
  Status (defaults to 'todo')
  """
  status: String
  """
  Priority level (defaults to 'normal')
  """
  priority: String
  """
  Tags for categorization
  """
  tags: [String!]
  """
  Markdown body content
  """
  body: String
  """
  Parent bean ID (validated against type hierarchy)
  """
  parent: String
  """
  Bean IDs this bean is blocking
  """
  blocking: [String!]
  """
  Bean IDs that are blocking this bean
  """
  blockedBy: [String!]
  """
  Custom ID prefix (overrides config prefix for this bean)
  """
  prefix: String
}
type Mutation {
  """
  Create a new bean
  """
  createBean(input: CreateBeanInput!): Bean!
  """
  Update an existing bean
  """
  updateBean(id: ID!, input: UpdateBeanInput!): Bean!
  """
  Delete a bean by ID (automatically removes incoming links)
  """
  deleteBean(id: ID!): Boolean!
  """
  Set or clear the parent of a bean (validates type hierarchy)
  """
  setParent(id: ID!, parentId: String, ifMatch: String): Bean!
  """
  Add a bean to the blocking list
  """
  addBlocking(id: ID!, targetId: ID!, ifMatch: String): Bean!
  """
  Remove a bean from the blocking list
  """
  removeBlocking(id: ID!, targetId: ID!, ifMatch: String): Bean!
  """
  Add a bean to the blocked-by list (this bean is blocked by targetId)
  """
  addBlockedBy(id: ID!, targetId: ID!, ifMatch: String): Bean!
  """
  Remove a bean from the blocked-by list
  """
  removeBlockedBy(id: ID!, targetId: ID!, ifMatch: String): Bean!
}
type Query {
  """
  Get a single bean by ID. Accepts either the full ID (e.g., "beans-abc1") or the short ID without prefix (e.g., "abc1").
  """
  bean(id: ID!): Bean
  """
  List beans with optional filtering
  """
  beans(filter: BeanFilter): [Bean!]!
}
"""
A single text replacement operation.
"""
input ReplaceOperation {
  """
  Text to find (must occur exactly once, cannot be empty)
  """
  old: String!
  """
  Replacement text (can be empty to delete the matched text)
  """
  new: String!
}
scalar Time
"""
Input for updating an existing bean
"""
input UpdateBeanInput {
  """
  New title
  """
  title: String
  """
  New status
  """
  status: String
  """
  New type
  """
  type: String
  """
  New priority
  """
  priority: String
  """
  Replace all tags (nil preserves existing, mutually exclusive with addTags/removeTags)
  """
  tags: [String!]
  """
  Add tags to existing list
  """
  addTags: [String!]
  """
  Remove tags from existing list
  """
  removeTags: [String!]
  """
  New body content (full replacement, mutually exclusive with bodyMod)
  """
  body: String
  """
  Structured body modifications (mutually exclusive with body)
  """
  bodyMod: BodyModification
  """
  Set parent bean ID (null/empty to clear, validates type hierarchy)
  """
  parent: String
  """
  Add beans to blocking list (validates cycles and existence)
  """
  addBlocking: [String!]
  """
  Remove beans from blocking list
  """
  removeBlocking: [String!]
  """
  Add beans to blocked-by list (validates cycles and existence)
  """
  addBlockedBy: [String!]
  """
  Remove beans from blocked-by list
  """
  removeBlockedBy: [String!]
  """
  ETag for optimistic concurrency control (optional)
  """
  ifMatch: String
}
