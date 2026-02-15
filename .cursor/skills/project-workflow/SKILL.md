---
name: project-workflow
description: Documents the sprint-based project workflow for the sifr GitHub project. Covers the full ticket lifecycle from planning through review, ticket types (Epics and Tasks), PRDS documents, board columns, and which commands to use at each phase. Use when the user asks about the workflow, sprint process, ticket management, or needs guidance on which command to run next.
---

# Project Workflow

Sprint-based workflow using GitHub Projects to manage tickets from creation through completion.

## Sprint Lifecycle

Create Tickets --> Refine --> Work --> Review --> Milestone Demo --> Done

## Ticket Types

**Epics** -- High-level features. Require a PRDS (PRD + Solution Design) drafted via `/create-prds` before being added as a ticket. Epics are later broken down into smaller Tasks.

**Tasks** -- Scoped work items. Drafted via `/create-task`. These are what developers actually work on.

Both types are added to the board via `/add-ticket`, which handles GitHub issue creation and project field setup.

## PRDS (PRD + Solution Design)

A combined document covering product requirements (problem, goals, scope, acceptance criteria) and the technical solution design (architecture, data model, API, testing). Created with `/create-prds` using the template at `.cursor/references/prd-solution-design-template.md`. Every Epic should have a PRDS.

## Board Columns

| Column      | Description                                    |
| ----------- | ---------------------------------------------- |
| Backlog     | Newly created tickets land here                |
| Ready       | Refined and prioritized, ready for development |
| In Progress | Currently being worked on                      |
| Review      | PR created, awaiting code review               |
| Done        | Merged and complete                            |

## Sprint Phases

### 1. Planning

Draft PRDS for epics (`/create-prds`) and task descriptions (`/create-task`), then add them to the board with `/add-ticket`. New tickets land in Backlog.

### 2. Refinement

Prioritize backlog tickets and move the highest priority ones to Ready with `/refinement`.

### 3. Epic Breakdown

When an Epic is refined and ready, break it down into smaller Task tickets (each added via `/add-ticket`) before development begins.

### 4. Development

Pick up the highest priority Ready ticket and create a PR with `/work-on-ticket`.

### 5. Review

Review PRs in the Review column with `/review-pr`.

### 6. Milestone Demo

Before marking a milestone (Epic) as Done, create a demo in `./demos` named `<milestone>_demo` (e.g., `m3_demo`). The demo should:

- Showcase all major features delivered in that milestone
- Include comments explaining each section
- Works as expected

### 7. Done

Merged PRs move to Done.

## Available Commands

| Command           | When to Use                                                |
| ----------------- | ---------------------------------------------------------- |
| `/create-prds`    | Draft a PRDS document for an epic                          |
| `/create-task`    | Draft a task description                                   |
| `/add-ticket`     | Add a drafted ticket to the GitHub project board           |
| `/refinement`     | Prioritize backlog tickets and move them to Ready          |
| `/work-on-ticket` | Pick up a Ready ticket, implement changes, and create a PR |
| `/review-pr`      | Review a PR in the Review column                           |

## GitHub Project Constants

- **Owner**: `yaseralnajjar`
- **Project number**: `2`
- **Project ID**: `PVT_kwHOAKAfcc4BPKkL`
- **Repository**: `yaseralnajjar/sifr`
