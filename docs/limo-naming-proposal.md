# Proposal: Limo CAD

**Status: Proposed · 2026-10-01 · Updated: 2026-10-02**

Rename **noBS CAD** to **Limo**, introduced as **Limo CAD**.

> **Design with understanding.**

Limo brings together two kinds of progress: refining a model and developing
the understanding behind it. A design becomes more useful as we work on it;
we become more capable as we understand the choices we make.

## A name to grow with

Limo is short and straightforward to introduce in a classroom, at a kitchen
table, in a workshop, or in a professional design review. Someone exploring a
first mechanism and someone refining a demanding assembly can recognize the
same idea: make the work better, and understand it more deeply. The name can
accompany a learner as their work becomes more ambitious.

Use **Limo CAD** in introductions, search listings, downloads, and links so
people can recognize and find the application. Use **Limo** once the context
is established. Write it with normal capitalization and a real space before
the secondary descriptor **CAD**.

In ordinary English, **limo** is also a familiar word for a limousine.
[Merriam-Webster](https://www.merriam-webster.com/dictionary/limo) records that
usage. **Limo CAD** gives the application a clear engineering identity;
the roots below supply its chosen name story. The suggested English
pronunciation is **"LEE-moh."**

## One product, in the user's language

The name should feel at home in the language someone has chosen. An English
page introduces **Limo CAD**, with English copy and an English banner. The
Chinese name belongs on the Chinese page. Spanish and German pages use
**Limo CAD** with their own localized copy. Each page presents a coherent
introduction, with a language selector for people who want another language.

The application already supports these four UI languages, as defined in
[the locale registry](../src/i18n/locales.ts):

- **English (`en`): Limo CAD**, shortened to **Limo** in context.
- **简体中文 — Simplified Chinese (`zh-CN`): 砺模 CAD**, shortened to **砺模**
  in context, with the intended reading **Lìmó**.
- **Español — Spanish (`es`): Limo CAD**, shortened to **Limo** in context.
- **Deutsch — German (`de`): Limo CAD**, shortened to **Limo** in context.

These are localized presentations of the same application, with the same
downloads, releases, examples, and community. The proposal's discussion of
Chinese characters explains the naming decision; it does not prescribe a
bilingual masthead. On the public site, the fuller Latin and Chinese origin
story belongs on an optional About/name page rather than in every banner.

### Carry the existing localization into the public pages

The app currently checks a saved language preference first, then the browser
language, then defaults to English. Its settings picker uses the languages'
own names, and missing translations fall back to English. Chinese, Spanish,
and German are community translations that may be incomplete. See the
[translation provider](../src/i18n/index.tsx) and
[settings language picker](../src/components/AppearanceDialog.tsx).

A dedicated project site should extend that approach to all four languages:

- Give each language a directly shareable page, for example `/en/`, `/zh-CN/`,
  `/es/`, and `/de/`. Translate the introduction, navigation, getting-started
  links, download guidance, page titles, and image descriptions. Have fluent
  contributors review the catchphrase and copy for natural wording.
- Offer **English**, **简体中文**, **Español**, and **Deutsch** in a clear language
  selector. Respect an explicit page link and the user's saved choice; browser
  language can suggest a starting point for a first visit. Keep the choice
  easy to change and preserve the topic when switching languages.
- Use the page's localized product name in its heading, banner, search
  description, and social preview. Keep the visual identity and geometry
  consistent across languages, with text prepared for each locale.
- Start localized learning and help with the introduction, first-part guide,
  common tasks, and connector setup. Link to the same editable examples and
  reviewed engineering resources. Clearly identify material still available
  only in English, and expand coverage as translations are reviewed.

Four app languages are a useful starting point; a fully translated website
and knowledge library would be follow-up work. This PR proposes that direction
without claiming that the pages or translated learning resources already exist.

## The roots

The naming story connects Latin refinement with a Chinese name built around
honing and modeling. The histories illuminate the shared idea; the public
presentation remains specific to each language.

### Latin: the file, the finished work, and the practiced mind

Latin **līmō** is a verb form: **"I file"** or **"I polish."** Its infinitive
is **līmāre**, and it comes from **līma**, a file. The name starts with a real
workshop action: making a surface or edge more exact through deliberate work.
The [verb and its inflection](https://en.wiktionary.org/wiki/limo#Latin) and
[Lewis and Short's dictionary entry](https://atlas.perseus.tufts.edu/dictionaries/entry/urn:cite2:scaife-viewer:dictionary-entries.atlas_v1:lat.ls.perseus-eng2-n26650/)
support both the literal and figurative meanings.

That figurative use already belongs to classical Latin. A writer could refine
a composition as a craftsperson files a piece of material. In **Horace's
*Ars Poetica*, line 291**, *limae labor et mora* evokes the labor and time
of filing as a picture of careful revision. A few lines later, Horace describes
his teaching role through a whetstone that sharpens iron. Refining the work
and helping its maker improve appear together in the same passage.
[Read lines 291–308](https://www.thelatinlibrary.com/horace/arspoet.shtml).

The metaphor reaches reasoning too. In **Cicero's *De Officiis*, 2.35**,
*veritas ipsa limatur in disputatione* presents truth itself as refined through
discussion. That gives the name a connection to examining an idea, questioning
it, and making our understanding more precise.
[Read the passage](https://beta.perseus.tufts.edu/urn:cts:latinLit:phi0474.phi055.perseus-lat2:2/).

The word family continues in modern language. Spanish **limar**, derived from
Latin *limāre*, means filing material and also polishing a work; **yo limo**
is its present-tense first-person form. The physical and figurative senses
still sit together in the [RAE dictionary](https://dle.rae.es/limar).

### Chinese: honing a tool, cultivating capability, shaping a model

The intended Chinese reading is **Lìmó**: **砺** (*lì*) followed by **模**
(*mó*).

**砺**, traditionally written **礪**, begins with the whetstone and the action
of sharpening. It also carries the established figurative sense of developing
oneself through practice. The [character entry](https://zdic.net/hans/%E7%A0%BA)
records both meanings and an example from **Xunzi's *Encouragement of Learning***:
**金就砺则利** — metal becomes sharp when brought to the whetstone.
The surrounding passage connects that physical image with learning and
self-examination. See [the original text](https://ctext.org/xunzi/quan-xue).

The connection is also visible in familiar compounds:

- **磨砺** (*mólì*): sharpening, and figuratively developing oneself through
  practice or experience. [Dictionary entry](https://zdic.net/hans/%E7%A3%A8%E7%A0%BA).
- **砥砺** (*dǐlì*): whetstones and sharpening, extending to training and
  encouragement. [Dictionary entry](https://zdic.net/hans/%E7%A0%A5%E7%A0%BA).

**模** supplies the model, pattern, and standard. Its history includes the
forms used in making objects; its meanings also include an example to follow.
That makes it a useful companion to 砺: a form we can work on, inspect, and
learn from. [Character entry](https://zdic.net/hans/%E6%A8%A1).

Modern compounds make that range concrete:

- **模型** (*móxíng*): a model, including a representation of an object's form
  and structure. [Dictionary entry](https://zdic.net/hans/%E6%A8%A1%E5%9E%8B).
- **模式** (*móshì*): a pattern or standard form.
  [Dictionary entry](https://zdic.net/hans/%E6%A8%A1%E5%BC%8F).
- **模范** (*mófàn*): an example or model worth learning from. The dictionary
  also records historical uses for a form used to make objects, and Yang
  Xiong's description of a teacher as a model for people.
  [Dictionary entry](https://zdic.net/hans/%E6%A8%A1%E8%8C%83).

For this name, **模 is mó**, as in 模型; the character also has a **mú**
reading in words such as 模具, a mold or die. **砺模** is a new pairing of
established characters, with the intended interpretation **"hone the model."**
The broader idea of developing the designer grows from these associations;
it is our brand interpretation, rather than a dictionary definition of 砺模.

### Where the meanings meet

Both traditions move from **working on material** to **working on capability**.
A file improves a surface. A whetstone improves an edge. Practice, revision,
and reflection improve the person doing the work.

The Latin name gives us the action of refinement; the Chinese name brings
that action into modeling and carries the association with cultivation.
Their similar sound offers a memorable connection, while their histories
remain independent. This is a deliberate meeting of meanings across languages.

In CAD, that becomes tangible: refine a fit, understand the clearance; change
a feature, understand the constraint; study a mechanism, understand its motion.
The model improves, and so does the designer's ability to reason about it.

Together, the roots give us the mission line:

> **Refine the model. Develop the designer.**

## Understanding through design

Mechanical design offers a natural way to learn. Change a dimension and see
how the part responds. Assemble two components and investigate their fit.
Move a mechanism and ask what constrains its motion. Compare two approaches
and understand why one suits the job.

Lessons, worked examples, and useful help can connect those actions to the
concepts behind them: geometry, constraints, fits, fasteners, materials,
manufacturing, and engineering judgment. The model remains something a person
can inspect, edit, and build on as their understanding develops.

Analysis belongs in that journey too: from existing fit and motion checks to
richer analysis as validated capabilities mature. Mechanical CAD stays at
the center.

There is already a foundation in the
[first-part lesson](INSTALL.md#make-your-first-part),
[recipe library](../examples/scripts/README.md),
[construction playback](native-scripts.md), and
[engineering knowledge](../knowledge/index.md). Improving access to these
resources, through clear introductions and help close to the task, makes the
project easier to discover and keep learning with.

## Working directly and with AI

People can work directly or connect an MCP-compatible AI agent. The
[existing MCP interface](../mcp-server/README.md) exposes modeling operations
and bundled engineering knowledge, and agent-built work retains editable
construction history. People can bring their preferred agent and model to
the same design tools.

Looking ahead, integrated AI guidance could help explain a modeling step,
find a relevant example, explore a design choice, or guide someone through a
mechanical concept. Human-readable resources and agent-accessible knowledge
should build on the same material, so the explanations and the work can be
examined together. See the [current help access guide](agentic/HUMAN_HELP.md).

Integrated tutoring and conversational design wizards are future direction.
They retain their own review under the project's priorities:
**reliability, performance, and ease of use**.

## Proposed English introduction

> **Limo CAD**
>
> **Design with understanding.**
>
> Open-source mechanical CAD for parts, assemblies, and drawings.
> Explore their motion. Work directly or with an agent.
>
> Learn through editable examples and engineering resources as you build.

Use **"Design with understanding."** as the main catchphrase, with
**"Refine the model. Develop the designer."** as the longer mission line.
Localize their meaning naturally for the Chinese, Spanish, and German pages;
review those versions with fluent contributors before publication.

Public copy should continue to show **pre-alpha** maturity. CAM is developing
and not production-safe; strength analysis is future work. See the
[current product direction](goals.md).

## Visual direction

![English proposal concept: Limo CAD with the line Design with understanding, beside a simple bracket transitioning from construction lines to a solid.](assets/branding/limo-proposal-concept-en.png)

*AI-generated concept for discussion; not an application screenshot, a
validated part, or a final logo. [Generation prompt and provenance](assets/branding/limo-proposal-concept.txt).*

Keep the name prominent and the geometry simple. For the public banner, a
real editable model or a brief dimension-change demonstration can make the
promise tangible. This English concept uses **Limo CAD** and English copy;
the Chinese page would use **砺模 CAD** and Chinese copy, and the Spanish and
German pages would use **Limo CAD** and copy in their respective languages.
Review final identity assets in the implementation PR.

## A dedicated home for the project

A dedicated Limo organization and project site could give the application,
learning resources, help, and AI connectors a stable, recognizable home.
Someone arriving from a classroom, community workshop, search result, or
shared example should be able to find the application and a useful next step
in their chosen language.

The organization profile can introduce **Limo CAD** in English and link to
the four localized site entrances. The site can then give each audience its
own introduction and route into downloads, mechanical design lessons,
engineering knowledge, worked examples, help, and connector setup. About and
contributor pages should clearly describe the project and its maintainers.

This would make localization part of how people discover and use the project,
with an identifiable home that contributors can maintain together. Creating
the organization, choosing its account and site names, assigning ownership,
and transferring a repository remain separate decisions after name acceptance.

## Review and next steps

1. **Review this proposal.** Agree on Limo, its Chinese name, the localized
   naming conventions, and the main catchphrase through the normal PR process.
   Record the decision in
   [ADR 0007](adr/0007-limo-name.md). This PR adds proposal documentation and
   a concept visual; it does not perform the application rename.
2. **After acceptance, submit the rename implementation.** Update public
   copy, localized application display names, and release presentation. Inventory
   packaging and persistent identifiers; preserve existing projects and agent
   setups and saved language preferences. Use "Limo CAD (formerly noBS CAD)"
   in English during the transition, with equivalent guidance on localized
   pages, so existing users can find the project. Check the
   chosen repository, site, and account names before the public cutover.
3. **Review the public site and organization plan.** Prepare the four localized
   entrances, translation review, and learning/help routes in follow-up work.
   Review a dedicated organization's ownership, maintenance responsibilities,
   and any repository transfer separately after the name is accepted.
