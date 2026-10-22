# Bugs — `bugs/` directory ledger

**Purpose.** Active records are one markdown file per bug directly under `bugs/`,
with filename == bug ID. Sealed Closed records live under
`bugs/archive/YYYY-MM/<ID>.md`, indexed by the top-level
`bugs/archive-YYYY-MM.md` manifest. There is **no `BUGS.md` — never create one.**

**Monthly archive.** A well-formed Closed record is sealed seven days after its
final Closed transition, into the month that transition names. Sealing publishes
the full original under `bugs/archive/YYYY-MM/` and a manifest row before deleting
the hot copy, so an interrupted run is safe to repeat. Closing a bug seals in
batches — the automatic pass waits for a batch of eligible records, or for the
oldest to have waited a month — so the current month's manifest is not rewritten on
every close; `deltic bugs archive` forces the pass.

**A manifest row is a lean index, not a copy.** It carries the title, the raised
date, every state transition at day precision, severity, priority, area,
resolution, discovery source, and the original's byte length and SHA-256. That is
what every table, filter, chart and lookup reads. Anything else about an archived
bug — the text attached to a transition, ownership, attempts, the requirement
backlink — is read from the original, which the row's digest proves unchanged.

`deltic bugs archive --check` re-reads every original and validates it against its
row, changing nothing. `deltic bugs archive --rebuild` re-rows every manifest from
its originals: run it after an operational pass has edited archived originals, and
it changes nothing when row and original already agree. Do not hand-edit archive
manifests or move records yourself. A later recurrence after closure is a new
regression record; Closed records are not reopened in place.

**ID grammar.** `PREFIX-TYPE-HOST-NNNNN`.
- **PREFIX** — this repo's acronym: `MDH`.
- **TYPE** — `BUG` in this ledger; the sibling ledgers mint `REQ` (`reqs/`, see
  `reqs/README.md`) and `ISS` (`issues/`). The ledgers share one ID parser, but
  the token **is** honoured: `deltic show` and every id-taking verb use it as a
  filter, so a `REQ`-spelled id typed at a `bugs` verb resolves to nothing rather
  than reaching the bug that happens to share its number.
- **HOST** — the machine, spelled the way deltic's host registry mints it. A
  **registered** machine mints its curated **3-letter code** (`CRU`, `ANV`,
  `ARC`); an unregistered one mints its full canonical token: the **first DNS
  label** of its hostname (`<name>.local`, `<name>.home.arpa`, and a bare
  `<name>` all collapse to `<NAME>`), uppercased and reduced to `[A-Z0-9]`. It
  is derived **in code** by `deltic bugs new` — not by a hand-run `hostname`,
  which would bake the DNS domain into the token and split one machine into two.
  An unregistered host is never *truncated* to three letters: a derived code
  could not be inverted back to the machine, so `deltic show` of a displayed id
  would miss.
- **Mixed spellings are normal, not debt.** One box legitimately carries several
  HOST spellings across a ledger's history — the long token it minted under
  before, its code now, and any legacy alias. Every reader folds them onto one
  machine, and the number space is unioned across all of them, so nothing
  renumbers and no id is ever reissued. Do not "tidy" old ids to the new
  spelling.
- **NNNNN** — a per-host sequence, zero-padded to at least 5 digits, **unified
  across the sibling ledgers**: the next number is `max(NNNNN)+1` over `bugs/`
  **and** `reqs/` **and** `issues/` for this host. **Unification holds only for
  items minted since that rule came in.** A repo can still carry older numbers
  claimed by *both* a bug and a req, so cite a bare host+number only together
  with its TYPE. Do **not** renumber those old ids to remove the overlap — the
  mixed-spellings rule above applies here for the same reason.
- Legacy `PREFIX-NNNNN` ids (pre-conversion) remain valid, verbatim, forever.

**Per-host allocation.** Raise a bug with
**`deltic bugs new --title "<summary>" --discovery-source <SOURCE>`** (`--severity`/`--priority`/`--area`/
`--body` optional; `--json` for tooling). It derives this machine's HOST token,
mints the next number across **all** sibling ledgers for that host, and writes
the record from this binary's own template. A clone-wide OS file reservation
serializes the local counter, worktree floor, and publish, so no number minted
from here on is shared with a sibling ledger. Add **`--land`** to mint-or-find the
record at origin and land it on the docs lane from wherever you stand (dedup key:
`--trunk-red` for the repo's trunk-red record **of this failure** — keyed on the
title, so a repeat of the same red dedups while a second, unrelated red raised
while the first is still open gets its own record — `--fingerprint <text>` for any
other stable key, else the title). It prints the id **and the title of the record
that id names**, minted or found: read the title, because it is the only thing
that tells a dedup which matched YOUR failure from one that handed you somebody
else's defect (MDK-BUG-CRU-01331).

**Always mint with the command — never by hand.** A hand-run `max(NNNNN)+1` over
one ledger's glob collides with the sibling ledgers by construction, and a
hand-run `hostname` bakes the DNS domain into the token, splitting one machine
into two (MDK-BUG-KILN-00152). The command derives HOST in code and reads the
whole number space.

**How concurrent minting is made safe (R-SAMEHOST).** The unit of concurrency on
one box is the **worktree**, not the machine — a host runs many task worktrees at
once, each a fork of trunk with its own `bugs/` directory. The number space is per
HOST and every worktree for a host lives on that host's filesystem, so the
allocator keeps a durable monotonic counter per clone and canonical HOST. Before
each batch, it floors that counter from this repository's local worktrees. It
does not reconstruct the number from Git history. A peer agent's uncommitted id
therefore still raises the floor, while a record removed after allocation leaves
only a harmless gap. An error reading a local worktree aborts the mint rather
than silently narrowing the floor — under-counting mints a duplicate, while
over-counting only skips a number (MDK-BUG-KILN-00150). The allocator holds an
exclusive OS file lock under `~/.deltic/locks`, keyed by the clone's canonical
common Git directory, across the floor, counter reservation, and publish. Process
exit releases the lock; there is no stale lease to recover (MDK-BUG-KILN-00176,
MDK-BUG-ANVIL-00334).

Two further backstops sit behind the reservation:
- A same-directory race is caught by the write itself: publication is atomic and
  no-clobbering, so the loser gets `AlreadyExists` and retries at the next number
  rather than overwriting the winner. This is a no-replace rename, **not**
  `O_EXCL`, and it compares filenames within **one directory** — it cannot see a
  peer worktree, and it cannot see a bug and a req holding the same number.
- Same number in the same ledger is the same path, so git raises an **add/add
  conflict** at integration — loud, and never a silent overwrite. The resolver
  renumbers.

Same number in **different** sibling ledgers is *not* the same path, so git
merges it silently. The reserved local counter and worktree floor prevent that
class (MDK-BUG-KILN-00151 and MDK-BUG-KILN-00176); a structural oracle in
`crates/deltic-repos/src/mint.rs` also fails the gate on a new occurrence.

**Timestamps.** Every timestamp deltic writes — `Raised` and each `State
history` transition — is **RFC 3339 UTC to whole seconds**: `YYYY-MM-DDTHH:MM:SSZ`.
Readers also accept a bare `YYYY-MM-DD` — the legacy shape — and those records
keep their day and simply have no recorded time. Ordering is by **day**, so
same-day transitions are read in the order they appear on the line. For an
automated transition the instant is when the *run* began, not the moment the file
was written — precise to the second, but not a measure of write latency.

**States & transitions.** `Open → Blocked → Fixed → Closed`, each dated and
attributed in the append-only `State history:` line. State is a **field inside
the file** (`- **State:** …`) — never rename a file to change its state.

**Two-eyes rule.** A bug moves to `Closed` **after a fix** only once a second
pair of eyes verifies it (regression test green, root cause understood).

**Fix provenance — `code=<sha>`.** A `Fixed` transition must name the commit that
fixed it, as a `code=<sha>` token in that transition's detail (`code <sha>` and
`` code `<sha>` `` are read too; the value is a 7–64 character lowercase git object
name). That marker is the only shape the reader accepts — a bare backticked sha names
nothing — and a `Fixed` record without it is parked as `MissingFixProvenance`, which
the verify lane then cannot pick up. `deltic bugs land <ID> --code <sha>` writes the
transition in the accepted shape and clears the fix claim in the same push; prefer it
over hand-editing the line.

**Closing without a fix — `Resolution`.** Not every bug ends in a fix. Some are
working as designed, some are not worth the spend, some stop applying, some were
never defects, and some cannot be reproduced after an initial observation. `deltic bugs close <ID> --resolution <VALUE> --reason "<why>"`
records that in one edit, and writes a `- **Resolution:**` field saying which:

| Value | Meaning |
| --- | --- |
| `by-design` | The behaviour is real and intentional. The report is accurate; this is what we want. |
| `wont-fix` | A real defect we agree exists and have decided not to spend on. |
| `obsolete` | The defect no longer applies — the surface was removed, rewritten, or retired. |
| `cannot-reproduce` | The defect was observed at least once but has not recurred and cannot be reproduced. If it recurs, raise a new record citing this one. |
| `not-a-bug` | The report is mistaken: what it describes does not actually happen. |
| `duplicate` | Already tracked by another record, named in the reason. |

Two pairs are worth getting right. If the reporter's description of current
behaviour is **accurate**, it is `by-design` or `wont-fix`; if what they
describe **does not happen**, it is `not-a-bug`. And `wont-fix` is a judgment
about priority while `obsolete` is a statement about the world — an `obsolete`
close must name *what* made it moot (the commit, refactor, or retirement), or it
becomes the lazy default for anything old, and the difference between "we are
under-resourcing quality" and "the code churned" is lost.

`Resolution` is **absent** on a bug closed the ordinary way after a verified fix,
and on records predating the field. There is no `fixed` value: a fixed bug closes
through verification, which is what enforces the two-eyes rule above. The verb
refuses a `Fixed` record for that reason, and refuses one that is still claimed —
release it first.

**Verifying a `Fixed` bug by hand.** Claim it with `deltic bugs claim <ID> --role
verify`, check the fix, then record the verdict with `deltic bugs verify <ID> --run
<RUN> --close|--reopen --note "<evidence>"`, where `<RUN>` is the `Owner run` the claim
printed. It writes the verdict and clears the claim in one ledger commit. `--reopen`
sends the bug back to `Open`, or parks it `Blocked` at the doubt threshold. The verb
refuses a record that changed since the claim, and a close while a known-failure
citation names the bug. Never verify a fix you made.

**Priority vs severity.** `Priority` is fix urgency (`Must` / `Should` /
`Could`). `Severity` is user impact (`Critical` / `High` / `Medium` / `Low`).
Malformed or missing priority is automation-ineligible until corrected.
`Severity` does **not** order the queue: it is an input to choosing the
priority, so ordering by it as well would weight user impact twice. It stays on
the record to inform whoever sets the priority, and for the dashboard and
release notes.

**Pick order.** Automation picks Open bugs by `Priority`, then `Discovery
source`, then oldest `Raised` first. So a customer- or human-reported bug leads
its priority band — responsiveness to the people who reported a problem does not
queue behind agent-generated backlog — but source is a tie-break *within* a
band, never above it. `Priority` is the raiser's own declaration of urgency, so
a human-raised `Could` (an explicit "this can wait") does not displace an
agent-raised `Must`. The verify-and-close queue leads with the attempt counts
instead (fewest `doubt`, then fewest `indeterminate`) so uncontested closures
clear first, and applies the same source order beneath them.

**Discovery source.** `Discovery source` records the earliest evidenced route
into engineering: `Customer`, `Human`, `Agent`, `Automation`, or `Unknown`.
Choose `Customer` only for customer issue intake and record the originating
issue backlink; a later issue backlink alone does not prove customer origin.
Use `Human` for non-customer manual/exploratory reports, `Agent` for model
judgment, and `Automation` for deterministic assertions. Use `Unknown` when the
available evidence does not establish the route. Historical records may omit
the field; records raised from 2026-08-02 must carry it. It ranks the queue in
that same listed order, with `Unknown` and omitted records last (a malformed value
ranks there too). In the Open queue that cannot bury the pre-field backlog, because
`Priority` is the outer key there: a legacy `Must` still outranks every `Should`.
That argument does not carry to the verify-and-close queue, which has no priority
key — there the source rank only orders bugs already equal on attempt counts.
`Agent` ranks above `Automation`: agent-raised defects come from model judgment
during review or testing, while automation-raised ones are deterministic
assertions that arrive with a failing test and stay cheap to pick up whenever
they are reached.

**Ownership.** The current owner lives in the bug file. `Owner role: human`
parks automation indefinitely for a named human owner. `Owner role: fix` and
`Owner role: verify` are leased automation claims; every automation owner must
carry the matching `Owner run`, `Owner host`, `Owner branch`, `Owner base`,
timestamps, and, for verify owners, `Owner fingerprint`. The unowned marker is
the ASCII hyphen `-` in every owner field. Partial, blank, or inconsistent owner
data is treated as malformed and skipped by automation.

**Renewing a manual claim.** A `deltic bugs claim` is a 4 h lease with no
supervisor. Extend it with `deltic bugs renew <ID> --run <RUN>`, where `<RUN>` is
the `Owner run` the claim printed. To renew several claims, pass `<ID>:<RUN>`
pairs instead, without `--run`: every valid pair renews in ONE ledger commit. A
pair whose run does not hold the lease, or whose lease has lapsed, is reported
(the stored run is never printed) and skipped; the rest still renew, and the
exit is non-zero. Naming one claim twice with different runs refuses the whole
batch. A lead holding many claims renews them as one batch, because
one commit per claim per loop floods trunk.
A renew writes only inside the last 60 minutes of the lease; earlier it
prints "Not due", writes nothing and exits 0, so a renew loop can run on any
cadence under an hour. `--force` writes a fresh 4 h lease regardless.

**Attempts and parking.** `Attempts: fix=N, doubt=N, indeterminate=N` records
durable unattended retry history. `Held branch` preserves useful fixer work that
needs human follow-up: release it with `deltic bugs resume <ID> --reason <why>`,
which clears the hold and claims the bug in one step, then prints the
`deltic start-task --bug <ID> --base <branch>` that resumes the preserved work.
Nothing deletes a held branch. `Blocked` is left with
`deltic bugs reopen <ID> --reason <why>`. `Verify retry after` temporarily parks indeterminate
verification without blocking future fix attempts; after three indeterminate
passes, automation leaves the bug parked as `Fixed` for human follow-up.
`Legacy fixed run` is set on pre-schema `Fixed` bugs so the verify loop has
explicit provenance.
**When verification cannot even start — `Verify unstarted` / `Verify blocker`.** A verify
pass sometimes fails before the verifier runs at all: the task worktree cannot be created,
the agent cannot be spawned. That is an infrastructure failure and is deliberately **not**
scored against the bug, so it leaves the `Attempts` counters alone. Both fields are
optional and absent until the first such failure, exactly like `Resolution`.

`- **Verify unstarted:**` counts CONSECUTIVE pre-start failures and resets to `0` the moment
a verifier actually runs. The retry delay lengthens on that count, and once it reaches three
each further failure scores an indeterminate, so a persistently broken box eventually parks
the bug as `Blocked` for a human instead of retrying forever. `- **Verify blocker:**` records
why the last one failed — a single flattened line, or `-`.

Both exist because the escalation ladder was previously unclimbable on this path: the
delay is read from the indeterminate counter, which a never-scoring path never moves, so
the retry stayed at fifteen minutes indefinitely and nothing ever surfaced. Three bugs
looped for over a day, and the reason lived only in one machine's local log
(MDK-BUG-ARC-00071).

**Which machine can verify it — `Platform`.** `- **Platform:**` names the OS family a
bug's verification must run on: `windows`, `macos`, `linux`, or `-`. The verify queue is
host-relative, so a record naming a platform is offered only to hosts of that family; a
record naming `-` is offered to every host.

**The requirement is learned, never declared.** The field is absent from the template and
from every record written before it existed, and nothing backfills it: no backfill could
know what a record needs, and guessing widely restores the defect while guessing narrowly
strands the backlog. Instead a verifier that finds it cannot confirm a bug on ANY machine
of its own family returns a `wrong-platform` verdict naming the family that can. deltic
records that once, releases the claim, and scores nothing — no attempt, no retry delay, no
pre-start count. WSL is deliberately not a value: deltic's verifier runs on the host OS, so
a Windows box with WSL is a Windows verifier.

The same write puts the requirement in `Verify blocker` in words. Once `Platform` narrows
the record to one family, that line is the only thing that says it is waiting, so a record
no online host can serve surfaces there rather than vanishing from every queue. A
`Platform` value this binary cannot read parks the record as malformed rather than widening
it back to "anyone". The whole-lane human override is unchanged: pin the drain entry's
`pinned_hosts` (MDK-BUG-FLU-00261).


**Requirement backlink.** `Req` names the sibling `reqs/` requirement this defect
breaches, or `-` when there is none. It is **detail-only**: read for the Repos→Bugs
detail row and nothing else — never resolved, never validated, never a flag, and
never an input to the queue. The checked direction is the reqs side's
`Violated-by:` (req→bug, shape-checked and resolved against `bugs/`); this is the
convenience reverse, so a stale or typo'd value here costs nothing.

**Titles.** The H1 title after the ID is the symptom, stated first, in at most
100 characters (`Area` in at most 64). It exists so a human or an agent can
recognise a duplicate at a glance and so an archived bug stays one lean manifest
row; everything else about the defect belongs in the body. `deltic bugs new`
refuses a longer one. A fixer may tighten a title — never re-aim it at the fix,
because a bug is named by what went wrong, not by what was done about it.

**File format.** Each `bugs/<ID>.md`:

```markdown
# MDH-BUG-HOST-00001 — Short title

- **State:** Open
- **Priority:** Should
- **Severity:** High
- **Area:** ui
- **Raised:** YYYY-MM-DDTHH:MM:SSZ
- **Discovery source:** Human
- **Owner:** -
- **Owner role:** -
- **Owner run:** -
- **Owner host:** -
- **Owner branch:** -
- **Owner base:** -
- **Owner fingerprint:** -
- **Owner since:** -
- **Owner until:** -
- **Verify retry after:** -
- **Held branch:** -
- **Legacy fixed run:** -
- **Attempts:** fix=0, doubt=0, indeterminate=0
- **Req:** -
- **State history:** Open (YYYY-MM-DDTHH:MM:SSZ, raised by …) → Fixed (YYYY-MM-DDTHH:MM:SSZ, code=<sha>)

## Observation
<symptom, repro, expected vs actual>

## Fix
<accepted fix summary and verification notes>

## Notes
<other notes, links, failed attempts>
```

> This ledger was bootstrapped by `deltic bugs init`. Edit it freely — it is your
> repo's own copy, not a fleet-managed cache.

Deltic appends successful `### Fix summary (...)` and `### Verification summary (...)`
sections under `## Fix`. Failed autonomous attempts still append `### Fix attempt
summary (...)` under `## Notes`. Older ledgers may have `### Fix summary (...)`
under `## Notes`; the Repos browser treats that legacy content as Fix prose. The
modeled header fields and `State history` remain the machine-readable ledger truth.

A transition may carry a `model=<id>@<effort>` token in its detail, recording who did
that step: an autonomous fix/verify records the agent family plus the effort deltic
pinned on that lane (`model=codex@max` for a Luna fix, `model=codex@medium` for an
Astra verify, `model=claude@high`), and a raise via
`deltic bugs new --model <id> --effort <tier>` records the precise raising model.
