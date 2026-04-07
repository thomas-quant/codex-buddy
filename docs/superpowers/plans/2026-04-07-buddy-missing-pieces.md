# Buddy Missing Pieces Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore a complete Buddy feature in `src/` by implementing the missing slash command, hatch/soul generation, pet/mute actions, and reaction observer while preserving the existing sprite, prompt, and attachment integrations.

**Architecture:** Keep the existing split between deterministic bones and persisted soul. Add a small `/buddy` command module under `src/commands/buddy/`, centralize Buddy config writes in `src/buddy/companion.ts`, generate soul text and reactions through lightweight side queries, and let `REPL.tsx` remain the single place that owns `companionReaction` state writes into the UI.

**Tech Stack:** TypeScript, React/Ink slash commands, existing command registry in `src/commands.ts`, global config persistence via `saveGlobalConfig`, Buddy rendering in `src/buddy/CompanionSprite.tsx`, side-model calls via `src/utils/sideQuery.ts`.

---

**Workspace note:** This plan targets the full upstream Codex repo with a
`src/` tree. The current extracted snapshot in this workspace only contains the
`buddy/` module, so this plan is not directly executable here without bringing
in the host application.

## File Structure

### Create

- `src/commands/buddy/index.ts`
  Buddy command descriptor registered by the existing lazy `require('./commands/buddy/index.js')` path in `src/commands.ts`.
- `src/commands/buddy/buddy.ts`
  `/buddy` command implementation for hatch, status, pet, mute, and unmute subcommands.
- `src/buddy/soul.ts`
  Soul-generation helper that turns deterministic bones plus user identity into a persisted `{ name, personality }` object.
- `src/buddy/observer.ts`
  Lightweight post-turn reaction generator and sanitizer; exports `fireCompanionObserver(...)`.

### Modify

- `src/buddy/companion.ts`
  Add Buddy-specific config helpers so command code and observer code do not write raw config shape inline.
- `src/screens/REPL.tsx`
  Import the observer module explicitly and keep the existing post-query `fireCompanionObserver(...)` call.

### Leave As-Is

- `src/buddy/CompanionSprite.tsx`
  Already consumes `companionReaction` and `companionPetAt`.
- `src/components/PromptInput/PromptInput.tsx`
  Already highlights `/buddy`, shows the Buddy footer item, and submits `/buddy`.
- `src/buddy/prompt.ts`
  Already injects the hidden `companion_intro` attachment.
- `src/utils/attachments.ts`
  Already threads `companion_intro` into model-visible messages.
- `src/utils/messages.ts`
  Already turns `companion_intro` into the main-assistant reminder text.

## Constraints

- Persist only `StoredCompanion` fields: `name`, `personality`, `hatchedAt`.
- Do not persist deterministic bones; always regenerate via `roll(companionUserId())`.
- Do not widen the Buddy UI footprint; keep the command text-first and let the existing sprite/footer remain the primary UI.
- Use `sideQuery()` for Buddy soul/reaction generation rather than the full streaming query loop.
- In this extracted snapshot there is no visible package manifest, `tsconfig`, or test harness, so verification steps below use direct source checks that run here. If implementation happens in the full upstream repo, add Buddy-focused automated tests in the repo’s existing harness during the same tasks.

## Task 1: Reconstitute the Buddy Command Entry Point

**Files:**
- Create: `src/commands/buddy/index.ts`
- Create: `src/commands/buddy/buddy.ts`
- Modify: `src/buddy/companion.ts`

- [ ] **Step 1: Add the command descriptor expected by `src/commands.ts`**

```ts
import type { Command } from '../../commands.js'

const buddy = {
  type: 'local',
  name: 'buddy',
  description: 'Hatch, pet, mute, unmute, or inspect your companion',
  argumentHint: '[status|pet|mute|unmute]',
  load: () => import('./buddy.js'),
} satisfies Command

export default buddy
```

- [ ] **Step 2: Add Buddy persistence helpers to `src/buddy/companion.ts`**

```ts
import { getGlobalConfig, saveGlobalConfig } from '../utils/config.js'
import type { CompanionSoul, StoredCompanion } from './types.js'

export function getStoredCompanion(): StoredCompanion | undefined {
  return getGlobalConfig().companion
}

export function saveStoredCompanion(
  soul: CompanionSoul,
  hatchedAt = Date.now(),
): void {
  saveGlobalConfig(current => ({
    ...current,
    companion: { ...soul, hatchedAt },
  }))
}

export function setCompanionMuted(muted: boolean): void {
  saveGlobalConfig(current => ({
    ...current,
    companionMuted: muted,
  }))
}
```

- [ ] **Step 3: Keep `getCompanion()` as the single merge point**

```ts
export function getCompanion(): Companion | undefined {
  const stored = getStoredCompanion()
  if (!stored) return undefined
  const { bones } = roll(companionUserId())
  return { ...stored, ...bones }
}
```

- [ ] **Step 4: Verify the command path now exists**

Run:

```bash
test -f src/commands/buddy/index.ts
test -f src/commands/buddy/buddy.ts
rg -n "commands/buddy/index.js|name: 'buddy'" src/commands.ts src/commands/buddy/index.ts
```

Expected:

- both `test -f` commands succeed
- `rg` prints the lazy loader in `src/commands.ts` and the new Buddy descriptor

- [ ] **Step 5: Commit**

```bash
git add src/commands/buddy/index.ts src/commands/buddy/buddy.ts src/buddy/companion.ts
git commit -m "feat: restore buddy command entrypoint"
```

## Task 2: Implement Hatch and Status Flow

**Files:**
- Create: `src/buddy/soul.ts`
- Modify: `src/commands/buddy/buddy.ts`
- Modify: `src/buddy/companion.ts`

- [ ] **Step 1: Add a soul-generation helper using `sideQuery()`**

```ts
import { sideQuery } from '../utils/sideQuery.js'
import { roll, companionUserId } from './companion.js'
import type { CompanionBones, CompanionSoul } from './types.js'

export async function generateCompanionSoul(
  bones: CompanionBones,
): Promise<CompanionSoul> {
  const prompt = [
    `Create a tiny coding companion soul.`,
    `Species: ${bones.species}`,
    `Rarity: ${bones.rarity}`,
    `Stats: ${Object.entries(bones.stats)
      .map(([k, v]) => `${k}=${v}`)
      .join(', ')}`,
    `Return strict JSON: {"name":"...", "personality":"..."}`,
  ].join('\n')

  const response = await sideQuery({
    model: 'default',
    querySource: 'side_question',
    system: prompt,
    messages: [{ role: 'user', content: `Seed: ${companionUserId()}` }],
    max_tokens: 120,
    temperature: 0.7,
    thinking: false,
  })

  return parseSoulResponse(response)
}
```

- [ ] **Step 2: Add a deterministic fallback so hatch never bricks**

```ts
export function fallbackCompanionSoul(bones: CompanionBones): CompanionSoul {
  return {
    name: `Buddy ${bones.species}`,
    personality: `${bones.species} energy; mildly judgmental, still helpful.`,
  }
}
```

- [ ] **Step 3: Implement `/buddy` and `/buddy status` in `src/commands/buddy/buddy.ts`**

```ts
import type { LocalCommandCall } from '../../types/command.js'
import {
  getCompanion,
  getStoredCompanion,
  roll,
  companionUserId,
  saveStoredCompanion,
} from '../../buddy/companion.js'
import { generateCompanionSoul, fallbackCompanionSoul } from '../../buddy/soul.js'

export const call: LocalCommandCall = async (args, context) => {
  const subcommand = args.trim()
  const existing = getCompanion()

  if (!subcommand || subcommand === 'status') {
    if (existing) {
      return { type: 'text', value: formatBuddyStatus(existing) }
    }

    const { bones } = roll(companionUserId())
    const soul = await generateCompanionSoul(bones).catch(() =>
      fallbackCompanionSoul(bones),
    )
    saveStoredCompanion(soul)
    return {
      type: 'text',
      value: formatHatchMessage({ ...bones, ...soul }),
    }
  }

  return dispatchBuddySubcommand(subcommand, context)
}
```

- [ ] **Step 4: Keep status output single-screen and text-first**

```ts
function formatBuddyStatus(companion: Companion): string {
  return [
    `${companion.name} the ${companion.species}`,
    `${companion.rarity} companion`,
    companion.personality,
    'Try /buddy pet, /buddy mute, or /buddy unmute.',
  ].join('\n')
}
```

- [ ] **Step 5: Verify hatch/status codepaths are present**

Run:

```bash
rg -n "generateCompanionSoul|fallbackCompanionSoul|formatBuddyStatus|saveStoredCompanion" src/buddy/soul.ts src/commands/buddy/buddy.ts src/buddy/companion.ts
```

Expected:

- all four symbols are found in the new Buddy hatch/status implementation

- [ ] **Step 6: Commit**

```bash
git add src/buddy/soul.ts src/commands/buddy/buddy.ts src/buddy/companion.ts
git commit -m "feat: add buddy hatch and status flow"
```

## Task 3: Implement `pet`, `mute`, and `unmute`

**Files:**
- Modify: `src/commands/buddy/buddy.ts`
- Modify: `src/buddy/companion.ts`

- [ ] **Step 1: Parse the supported subcommands explicitly**

```ts
switch (subcommand) {
  case 'pet':
    return handleBuddyPet(context)
  case 'mute':
    return handleBuddyMute(true)
  case 'unmute':
    return handleBuddyMute(false)
  case 'status':
    return handleBuddyStatus()
  default:
    return {
      type: 'text',
      value: 'Unknown /buddy subcommand. Use /buddy, /buddy status, /buddy pet, /buddy mute, or /buddy unmute.',
    }
}
```

- [ ] **Step 2: Drive `companionPetAt` from the command context**

```ts
function handleBuddyPet(context: LocalJSXCommandContext): LocalCommandResult {
  const companion = getCompanion()
  if (!companion) {
    return { type: 'text', value: 'Hatch your companion first with /buddy.' }
  }

  context.setAppState(prev => ({
    ...prev,
    companionPetAt: Date.now(),
  }))

  return {
    type: 'text',
    value: `You pet ${companion.name}.`,
  }
}
```

- [ ] **Step 3: Persist mute state through the helper instead of writing config inline**

```ts
function handleBuddyMute(muted: boolean): LocalCommandResult {
  const companion = getCompanion()
  if (!companion) {
    return { type: 'text', value: 'Nothing to mute yet. Hatch your companion first with /buddy.' }
  }

  setCompanionMuted(muted)
  return {
    type: 'text',
    value: muted
      ? `${companion.name} is now muted.`
      : `${companion.name} is back.`,
  }
}
```

- [ ] **Step 4: Verify there is now a writer for `companionPetAt`**

Run:

```bash
rg -n "companionPetAt: Date.now\\(\\)|setCompanionMuted\\(" src/commands/buddy/buddy.ts src/buddy/companion.ts
```

Expected:

- one hit for the `companionPetAt` write
- one hit for the mute helper call

- [ ] **Step 5: Commit**

```bash
git add src/commands/buddy/buddy.ts src/buddy/companion.ts
git commit -m "feat: add buddy pet and mute subcommands"
```

## Task 4: Implement the Buddy Observer

**Files:**
- Create: `src/buddy/observer.ts`
- Modify: `src/screens/REPL.tsx`

- [ ] **Step 1: Add a narrow observer API that matches the current REPL callsite**

```ts
import type { Message } from '../types/message.js'

export async function fireCompanionObserver(
  messages: Message[],
  onReaction: (reaction: string | undefined) => void,
): Promise<void> {
  const reaction = await maybeGenerateCompanionReaction(messages)
  onReaction(reaction)
}
```

- [ ] **Step 2: Add cheap early exits before making any side query**

```ts
function shouldSkipBuddyReaction(messages: Message[]): boolean {
  const companion = getCompanion()
  if (!companion) return true
  if (getGlobalConfig().companionMuted) return true
  if (!hasRecentAssistantText(messages)) return true
  if (!passesOccasionalReactionGate(messages, companion)) return true
  return false
}
```

- [ ] **Step 3: Generate a single-line reaction with `sideQuery()`**

```ts
async function maybeGenerateCompanionReaction(
  messages: Message[],
): Promise<string | undefined> {
  if (shouldSkipBuddyReaction(messages)) return undefined

  const companion = getCompanion()!
  const transcript = extractRecentTranscript(messages)
  const response = await sideQuery({
    model: 'default',
    querySource: 'side_question',
    system: buildBuddyObserverPrompt(companion),
    messages: [{ role: 'user', content: transcript }],
    max_tokens: 80,
    temperature: 0.8,
    thinking: false,
    stop_sequences: ['\n'],
  }).catch(() => null)

  return sanitizeReaction(response)
}
```

- [ ] **Step 4: Sanitize aggressively so UI only sees clean quips**

```ts
function sanitizeReaction(response: BetaMessage | null): string | undefined {
  const text = extractText(response).trim().replace(/\s+/g, ' ')
  if (!text) return undefined
  if (/^(none|silent|skip)$/i.test(text)) return undefined
  return text.slice(0, 120)
}
```

- [ ] **Step 5: Import the observer explicitly in `src/screens/REPL.tsx`**

```ts
import { fireCompanionObserver } from '../buddy/observer.js'
```

- [ ] **Step 6: Keep the existing REPL write path unchanged**

```ts
if (feature('BUDDY')) {
  void fireCompanionObserver(messagesRef.current, reaction =>
    setAppState(prev =>
      prev.companionReaction === reaction
        ? prev
        : { ...prev, companionReaction: reaction },
    ),
  )
}
```

- [ ] **Step 7: Verify the symbol now resolves from both ends**

Run:

```bash
test -f src/buddy/observer.ts
rg -n "fireCompanionObserver" src/buddy/observer.ts src/screens/REPL.tsx
```

Expected:

- the observer file exists
- `rg` shows one export in `src/buddy/observer.ts` and one import/call path in `src/screens/REPL.tsx`

- [ ] **Step 8: Commit**

```bash
git add src/buddy/observer.ts src/screens/REPL.tsx
git commit -m "feat: restore buddy reaction observer"
```

## Task 5: End-to-End Buddy Smoke Pass

**Files:**
- Modify: `src/commands/buddy/buddy.ts` if any final copy or edge-case fixes are needed
- Modify: `src/buddy/soul.ts` if any sanitization fixes are needed
- Modify: `src/buddy/observer.ts` if any reaction gating fixes are needed

- [ ] **Step 1: Run structural verification for all missing surfaces**

Run:

```bash
test -f src/commands/buddy/index.ts
test -f src/commands/buddy/buddy.ts
test -f src/buddy/soul.ts
test -f src/buddy/observer.ts
rg -n "saveStoredCompanion|setCompanionMuted|companionPetAt: Date.now\\(\\)|fireCompanionObserver|generateCompanionSoul" src
```

Expected:

- all four files exist
- `rg` finds one implementation site for each of the previously missing capabilities

- [ ] **Step 2: Run manual REPL scenarios in this order**

1. Start with no `config.companion`; invoke `/buddy`; confirm a soul is generated and persisted.
2. Invoke `/buddy status`; confirm the output uses the persisted soul plus deterministic bones.
3. Invoke `/buddy pet`; confirm hearts appear in `CompanionSprite`.
4. Invoke `/buddy mute`; confirm sprite/prompt intro/reactions stop rendering.
5. Invoke `/buddy unmute`; confirm rendering returns.
6. Complete one normal chat turn; confirm the observer may populate `companionReaction`.
7. Scroll the transcript; confirm `companionReaction` clears as already implemented in `REPL.tsx`.

- [ ] **Step 3: If the full upstream repo is available, run its normal typecheck/build immediately after the smoke pass**

Run the repo’s standard compile command from the full workspace, then fix any import or type drift before merging. In this extracted snapshot, do not claim compile success because the package manifest and toolchain are not present.

- [ ] **Step 4: Commit**

```bash
git add src/commands/buddy/index.ts src/commands/buddy/buddy.ts src/buddy/companion.ts src/buddy/soul.ts src/buddy/observer.ts src/screens/REPL.tsx
git commit -m "feat: complete buddy feature wiring"
```

## Self-Review

- Spec coverage: this plan covers every hard absence found in `src/`: command module, hatch flow, soul generation, pet writer, mute writer, and observer implementation.
- Placeholder scan: no TBDs or implicit “do the rest later” steps remain; every task names exact files and exact verification commands.
- Type consistency: the plan keeps the existing `StoredCompanion` contract, keeps `companionReaction` and `companionPetAt` in app state, and uses the exact missing symbol name `fireCompanionObserver`.
