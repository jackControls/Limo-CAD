# ADR 0007 — Limo product name and localized presentation

- Status: Proposed
- Date: 2026-10-01
- Updated: 2026-10-02
- Detail: [Limo naming proposal](../limo-naming-proposal.md)

## Context

The project needs a name that is welcoming across ages and experience levels,
easy to introduce and find, and suited to mechanical design with learning,
engineering resources, and optional AI assistance.

Limo connects model refinement with honing the designer's judgment. The
proposal explains its Latin root, Chinese name, and learning opportunities.
The app offers English, Simplified Chinese, Spanish, and German; public
naming should follow the user's selected language.

## Proposed decision

Rename noBS CAD to **Limo**, with **砺模** as its Chinese name. Introduce the
application as **Limo CAD** on English, Spanish, and German pages and as
**砺模 CAD** on Simplified Chinese pages. Use the shorter name for the selected
language once the context is established.

Use **"Design with understanding."** as the English main tagline, with
naturally localized wording reviewed for the other languages. Each localized
page uses its own product heading, copy, and banner. Put the fuller Latin
and Chinese origin story on an optional About/name page.

Propose a public site with entrances for the four existing app languages,
a visible language selector, and localized routes into learning, help, and
connector setup. Website and knowledge translations are follow-up work;
existing community UI translations may be incomplete.

Review and record acceptance through the proposal PR before implementing the
rename. After approval, update this record's status to Accepted and record
the accepting PR. Implementation follows in separately reviewed changes.

## Consequences

- This proposal PR changes documentation and includes a concept image only.
  Application names, packages, identifiers, project formats, MCP configuration,
  repository ownership, and public site identity are not changed here.
- Follow-up rename work must preserve existing projects and agent connections
  and saved language preferences, and provide a clear path from the previous
  name to the new one in each locale.
- Learning and integrated AI feature proposals retain their own scope and
  review. This decision does not change the engineering priorities in
  [ADR 0005](0005-main-goals.md) or declare future features shipped.
- A dedicated organization and localized project site could provide a shared
  home for the app, learning, help, and connectors. Organization creation,
  ownership, and any repository transfer require separate review.
- The concept image illustrates a possible presentation; it is not a final
  identity asset or evidence of CAD functionality.
