---
name: user-documentation
description: Write and maintain the end-user documentation published on the Woodstock Backup website (docs/website/doc/, VitePress). Use this whenever the user asks to write, update, review, or check a user-facing doc page, add a new feature to the docs site, register a page in the VitePress sidebar (docs/website/.vitepress/config.mts), or verify that docs/website is up to date with a recent code change — even if they just say "update the docs" or "document this feature" without naming the file. This is strictly for end-user content (what to install, configure, click, expect); for architecture/API/ADR/developer-facing docs (docs/developer_guide/, doc/internal/), use the technical-documentation skill instead.
---

# User Documentation (Woodstock Backup website)

The goal here is narrower than "write good docs" in the abstract: every page in
`docs/website/doc/` is read by someone deciding whether to trust this backup
software with their data. A step that doesn't match reality, a config field
that doesn't exist, a missing prerequisite — any of these breaks that trust
and sends the reader to file an issue or give up. Optimize for "a reader can
follow this without guessing and without it being wrong," not for prose
polish.

## Scope: end-user only

`docs/website/doc/` serves two audiences that must not mix:

- **End-user pages** (`index.md`, `installation*.md`, `agent.md`,
  `configuration.md`, `authentication.md`, `scheduler.md`, `faq.md`,
  `roadmap.md`, `migration/*`) — what a person running Woodstock Backup needs
  to install, configure, and operate it. This skill covers these.
- **`doc/internal/*`** and **`docs/developer_guide/*`** — implementation
  notes, protocol internals, architecture decisions. Not this skill's job —
  point to the `technical-documentation` skill instead. If a task touches
  both (e.g. a new feature needs a user-facing page *and* an internal design
  note), handle the user-facing half here and say so.

Never let internal naming leak into user pages. The clearest example in this
codebase: pool/refcount code calls the hash field `sha256` for legacy reasons,
but it's actually Blake3 (see root `CLAUDE.md`). A user page should never
surface that kind of internal quirk — describe what the user sees and
configures, not how the code names it internally.

## Before writing: verify against the actual code

A doc that was true when written but silently drifted is worse than no doc —
it actively misleads. Before documenting a config field, CLI flag, or
behavior, check it still exists and still works as described:

- Config fields/defaults: grep the real struct, don't recall or guess
  (`woodstock-rs/src/config/model.rs`, `hosts.rs`). Copy the actual default
  value, not an approximation.
- CLI subcommands/flags: check the actual `clap` definitions in the relevant
  `src/bin/` or `cli-rs` crate.
- Behavior claims ("X happens when Y"): trace it in the code or ask, don't
  assume from the field name.
- If a field exists in the schema but isn't wired end-to-end yet, don't
  document it as usable — see "Partial features" below.

When revising an existing page, treat every concrete claim in it (field name,
default, command, path, screenshot) as a claim to re-verify, not as ground
truth — code moves faster than docs.

## Structure for a page

Not every page needs every section, but this is the default shape — it
matches what `configuration.md`, `agent.md`, and `authentication.md` already
do:

```markdown
# [Feature/Topic]

One-sentence statement of what this page covers and who needs it.

## Prerequisites
What must already be true/installed/configured before starting.

## [Task section(s)]
Numbered steps for anything sequential. State the expected result after
steps that aren't obviously reversible or obviously successful
("you should now see the host listed as online").

## Configuration reference (if applicable)
| Field | Default value | Description |
|-------|---------------|-------------|

## Limitations
What doesn't work yet, or only works under specific conditions. Say it
plainly rather than omitting it — a documented limitation builds more trust
than a silent gap the user discovers the hard way.

## See also
Links to related pages.
```

Match existing conventions exactly: YAML/bash code blocks with the language
tag set, real paths from this repo (`/var/lib/woodstock/...`), tables for
config fields in the `| Field | Default value | Description |` shape already
used throughout. Write in **English** — the entire site is English regardless
of the language the request came in.

## Diagrams

If a page needs a schema (flow, sequence, architecture), use a fenced
`mermaid` code block — the site renders it as a real diagram
(`vitepress-plugin-mermaid`) and it also renders natively on GitHub/Gitea, so
the same source works everywhere. Never fall back to a hand-typed ASCII
diagram or a prose description of "boxes and arrows" — those don't render as
a picture anywhere and are the first thing to go stale. Don't use PlantUML
either: it was removed from this site because its rendering path sends the
diagram source to a public third-party server (`plantuml.com`) just to get
back an image, which is both a reliability dependency and an odd thing to
route architecture/auth diagrams through for a project built around pull-only
backups and mTLS.

## Adding or moving a page

A new page isn't reachable until it's wired into VitePress:

1. Create `docs/website/doc/<name>.md`.
2. Add it to the sidebar in `docs/website/.vitepress/config.mts` — find the
   right group (`Documentation`, `Migration Documentation`, or `Internal
   Documentation`) and add `{ text: "...", link: "/doc/<name>" }` (no `.md`).
3. If it's a page a new user would need to discover, link it from
   `doc/index.md` too — the index is the entry point and stale if it doesn't
   point to everything important.
4. Check for dead links both ways: the new page's own links, and whether
   anything should now link *to* it (e.g. `faq.md` referencing a feature that
   now has its own page).

## Partial or in-progress features

When a feature ships partially, resist writing the doc for the finished
version. Document, explicitly:

- what already works today
- what's a known limitation right now
- what exists in config/schema but isn't actually usable yet

This is also when `roadmap.md` needs a matching update — move the item out of
"planned" once it's real, or note the partial state if it's not fully there.
A roadmap that still lists a shipped feature as upcoming is itself a form of
untrustworthy doc.

## Detecting stale docs (no code change in hand yet)

If asked to check whether the docs are current rather than to write something
new: pick the user-facing pages most likely tied to recent work (check
`git log` on `server-rs/`, `client-rs/`, `woodstock-rs/` for what changed
recently), then for each claim in the relevant doc page, re-verify against
the code as described above. Report mismatches rather than silently
"improving" prose — the user decides what's worth a doc update.

## Before calling a page done

- Every config field/command/path mentioned was just verified against the
  current code, not recalled from memory or from the page's previous text.
- A first-time reader has everything they need in order (prerequisites
  before steps, not discovered mid-step).
- No internal jargon, internal field-naming quirks, or implementation detail
  that belongs in `doc/internal/` or `docs/developer_guide/` instead.
- New pages are reachable: sidebar entry added, `index.md` linked if it's a
  major page.
- Limitations are stated, not omitted.
- Internal links resolve.
