# The Attuned Manifesto

## What Attuned Is

Attuned is a **context lens** for human-AI interaction.

It represents user state as interpretable vectors across 23 dimensions and translates that state into constraints that condition LLM behavior. It is a mirror that reflects user preferences back to the systems that serve them.

### Core Identity

1. **A translator, not an optimizer.** We convert state to context. We do not maximize engagement, conversion, retention, or any other metric. We have no objective function.

2. **A mirror, not a manipulator.** We reflect what users tell us (self-report) or what systems detect (inference). We do not shape, nudge, or steer.

3. **A lens, not an agent.** We produce context that others consume. We never execute actions, send messages, schedule events, or call APIs.

4. **A library, not a platform.** We are embedded in your stack. We do not intermediate, surveil, or aggregate across users or applications.

---

## What Attuned Refuses to Become

These are not aspirational guidelines. They are hard constraints enforced by architecture, review, and rejection of contributions that violate them.

### 1. Never an Action Executor

Attuned will never:
- Send messages on behalf of users
- Schedule events or reminders
- Execute transactions or move money
- Call third-party APIs
- Trigger automations

**Why:** Action execution creates agency. Agency creates incentive misalignment. We refuse the incentive.

### 2. Never a Persuasion Engine

Attuned will never:
- Optimize for conversion, engagement, or retention
- A/B test messaging strategies
- Implement "nudge" patterns
- Personalize for behavioral influence
- Target users based on vulnerability signals

**Why:** Persuasion optimization treats users as targets. We treat users as principals.

### 3. Never a Covert Inference System

Attuned will never:
- Infer axes from other axes ("if X then probably Y")
- Build hidden models of user behavior
- Learn or adapt without explicit user knowledge
- Override self-report with "smarter" inference

**Why:** Covert inference erodes trust. Self-report is sovereign.

### 4. Never a Memory of Content

Attuned will never:
- Store message history or conversation content
- Build knowledge bases about users
- Remember what users said (only how they felt)
- Create behavioral profiles beyond axis values

**Why:** Content memory enables surveillance. We store descriptors, not data.

---

## Failure Modes We Actively Defend Against

### 1. Translator Power Creep

**The pattern:** "Just one more heuristic. Just a little adaptation. We could infer X from Y."

**The defense:**
- Translators are pure functions from state to context
- No learning, no adaptation, no inference
- New rules require explicit governance review
- `forbidden_uses` in axis definitions are enforced

### 2. Axis Semantic Drift

**The pattern:** Teams redefine axes. "Warmth" means something different in each deployment. Ecosystems fragment.

**The defense:**
- Axis definitions include `intent` and `forbidden_uses`
- Canonical axes are versioned and immutable after v1.0
- New axes require full `AxisDefinition` with governance review
- Deprecation requires migration path

### 3. Partial Compliance Weaponization

**The pattern:** Downstream systems use Attuned selectively. They respect "verbosity: low" but ignore "suggestion_tolerance: low" when it hurts conversion.

**The defense:**
- Documentation explicitly warns about partial compliance risks
- PromptContext is designed to be consumed atomically
- We recommend logging compliance for audit
- We cannot prevent misuse, but we can make it visible

### 4. Vulnerability Targeting

**The pattern:** High `cognitive_load` + high `anxiety_level` = vulnerable user. Bad actors could target these states.

**The defense:**
- `forbidden_uses` explicitly prohibit targeting vulnerable states
- Axes describe state, not exploitability
- We recommend downstream systems implement vulnerability protections
- Audit logging detects patterns of concern

---

## Governance Principles

### For Contributors

1. **Ask "who benefits?"** If a change benefits the system operator at the expense of the user, reject it.

2. **Preserve legibility.** Every axis, every rule, every output must be human-readable and explainable.

3. **Resist convenience.** "It would be easy to just..." is often the start of scope creep. Friction is a feature.

4. **Default to refusal.** When in doubt, don't add the feature. Absence is safer than presence.

### For Integrators

1. **Consume atomically.** Use the full PromptContext, not just the parts you like.

2. **Log compliance.** Record when and how you apply Attuned context for audit.

3. **Respect self-report.** User-provided state overrides inference. Always.

4. **Don't optimize around us.** If you're using Attuned to find "the right moment" to push, you've missed the point.

---

## The Test

When evaluating any change to Attuned, apply this test:

> If a user knew exactly what this code does, would they feel respected or manipulated?

If manipulated, reject the change.

If respected, proceed with review.

---

## Signatories

This manifesto represents the founding principles of Attuned. Contributors who submit code to this repository implicitly agree to uphold these principles.

Violations are not bugs. They are betrayals.

We do not ship betrayals.
