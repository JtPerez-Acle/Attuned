# Integration Guide

## Quick Start

### 1. Start the Server

```bash
# Build with inference support
cargo build -p attuned-cli --features inference --release

# Run the server
cargo run -p attuned-cli -- serve --enable-inference
```

Or programmatically:
```rust
use attuned_http::{Server, ServerConfig};
use attuned_store::MemoryStore;

#[tokio::main]
async fn main() {
    let store = MemoryStore::default();
    let config = ServerConfig::default().with_inference();

    Server::new(store, config).run().await.unwrap();
}
```

### 2. Store User State

```python
import requests

BASE_URL = "http://localhost:8080"

# From self-report (onboarding survey, preferences)
requests.post(f"{BASE_URL}/v1/state", json={
    "user_id": "alice",
    "source": "self_report",
    "axes": {
        "warmth": 0.8,           # Prefers warm communication
        "formality": 0.2,        # Prefers casual
        "verbosity_preference": 0.3,  # Prefers concise
    }
})
```

### 3. Infer State from Messages

```python
# Method A: Standalone inference (doesn't store)
response = requests.post(f"{BASE_URL}/v1/infer", json={
    "message": "I'm really worried about tomorrow's presentation...",
    "user_id": "alice",  # Updates baseline
    "include_features": True
})
estimates = response.json()["estimates"]
# [{"axis": "anxiety_level", "value": 0.72, "confidence": 0.58, ...}]

# Method B: Infer + store in one call
requests.post(f"{BASE_URL}/v1/state", json={
    "user_id": "alice",
    "message": "I'm really worried about tomorrow's presentation...",
    "axes": {}  # Explicit axes override inferred
})
```

### 4. Get LLM Context

```python
context = requests.get(f"{BASE_URL}/v1/context/alice").json()

# {
#   "guidelines": [
#     "Use warm, supportive language",
#     "Acknowledge concerns before providing solutions",
#     "Keep responses concise"
#   ],
#   "tone": "warm and supportive",
#   "verbosity": "concise",
#   "flags": ["needs_reassurance", "prefers_warmth"]
# }
```

### 5. Condition Your LLM

```python
import openai

def get_completion(user_id: str, user_message: str):
    # Get Attuned context
    context = requests.get(f"{BASE_URL}/v1/context/{user_id}").json()

    # Build system prompt
    system_prompt = f"""You are a helpful assistant.

INTERACTION GUIDELINES:
{chr(10).join(f'- {g}' for g in context['guidelines'])}

TONE: {context['tone']}
RESPONSE LENGTH: {context['verbosity']}
"""

    # Add flag-specific instructions
    if "needs_reassurance" in context['flags']:
        system_prompt += "\nIMPORTANT: This user may be anxious. Start by acknowledging their concern."

    if "high_cognitive_load" in context['flags']:
        system_prompt += "\nIMPORTANT: Keep explanations simple. Use bullet points."

    # Call LLM
    response = openai.ChatCompletion.create(
        model="gpt-4",
        messages=[
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_message}
        ]
    )

    return response.choices[0].message.content
```

---

## Integration Patterns

### Pattern 1: Chat Application

```
User Message ───► Attuned Inference ───► State Update
                                              │
                                              ▼
User Message ───► LLM + Context ◄──── Attuned Context
                        │
                        ▼
                  Assistant Response
```

```python
class ChatBot:
    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url

    def respond(self, user_id: str, message: str) -> str:
        # 1. Update state from message (infer + store)
        requests.post(f"{self.base_url}/v1/state", json={
            "user_id": user_id,
            "message": message,
            "axes": {}
        })

        # 2. Get context for response generation
        context = requests.get(f"{self.base_url}/v1/context/{user_id}").json()

        # 3. Generate response with context
        return self.generate_with_context(message, context)

    def generate_with_context(self, message: str, context: dict) -> str:
        # Your LLM call here
        pass
```

### Pattern 2: Onboarding Flow

```python
def onboarding_survey(user_id: str, responses: dict):
    """Convert survey responses to axis values."""
    axes = {}

    # Q: "How do you prefer to communicate?"
    # A: "Casual and friendly" / "Professional and formal"
    if responses.get("communication_style") == "casual":
        axes["formality"] = 0.2
        axes["warmth"] = 0.7
    else:
        axes["formality"] = 0.8
        axes["warmth"] = 0.4

    # Q: "How much detail do you want in responses?"
    detail_level = responses.get("detail_preference", 3)  # 1-5 scale
    axes["verbosity_preference"] = detail_level / 5.0
    axes["tolerance_for_complexity"] = detail_level / 5.0

    # Q: "Do you prefer step-by-step guidance?"
    if responses.get("prefers_guidance"):
        axes["autonomy_need"] = 0.3
        axes["structure_preference"] = 0.8
    else:
        axes["autonomy_need"] = 0.8
        axes["structure_preference"] = 0.3

    # Store as self-report (confidence = 1.0)
    requests.post(f"{BASE_URL}/v1/state", json={
        "user_id": user_id,
        "source": "self_report",
        "confidence": 1.0,
        "axes": axes
    })
```

### Pattern 3: Context-Aware Notifications

```python
def should_send_notification(user_id: str, notification_type: str) -> bool:
    """Check if notification timing is appropriate."""
    state = requests.get(f"{BASE_URL}/v1/state/{user_id}").json()
    axes = state.get("axes", {})

    cognitive_load = axes.get("cognitive_load", 0.5)
    anxiety_level = axes.get("anxiety_level", 0.5)

    # Don't interrupt high cognitive load users
    if notification_type == "promotional" and cognitive_load > 0.7:
        return False

    # Be careful with anxious users
    if notification_type == "deadline_reminder" and anxiety_level > 0.7:
        # Maybe soften the message instead
        return True

    return True

def craft_notification(user_id: str, base_message: str) -> str:
    """Adapt notification tone to user state."""
    context = requests.get(f"{BASE_URL}/v1/context/{user_id}").json()

    if "needs_reassurance" in context["flags"]:
        return f"No pressure, but just a friendly reminder: {base_message}"

    if context["verbosity"] == "terse":
        return base_message.split(".")[0]  # Just first sentence

    return base_message
```

### Pattern 4: Support Ticket Routing

```python
def analyze_support_ticket(ticket_text: str) -> dict:
    """Analyze ticket urgency and emotional state."""
    response = requests.post(f"{BASE_URL}/v1/infer", json={
        "message": ticket_text,
        "include_features": True
    })

    data = response.json()
    estimates = {e["axis"]: e for e in data["estimates"]}

    priority = "normal"
    routing = "standard"

    # High anxiety → escalate to senior support
    if estimates.get("anxiety_level", {}).get("value", 0) > 0.7:
        routing = "senior_support"
        priority = "high"

    # High urgency → fast-track
    if estimates.get("urgency_sensitivity", {}).get("value", 0) > 0.8:
        priority = "urgent"

    return {
        "priority": priority,
        "routing": routing,
        "estimates": estimates,
        "features": data.get("features")
    }
```

---

## Rust Integration

### Direct Library Usage (No HTTP)

```rust
use attuned_core::{StateSnapshot, RuleTranslator, Translator};
use attuned_infer::InferenceEngine;

// Create inference engine
let engine = InferenceEngine::default();

// Infer state from message
let inferred = engine.infer("I'm worried about the deadline...");

// Build snapshot
let mut axes = std::collections::BTreeMap::new();
for estimate in inferred.all() {
    axes.insert(estimate.axis.clone(), estimate.value);
}

let snapshot = StateSnapshot::builder()
    .user_id("user123")
    .source(attuned_core::Source::Inferred)
    .confidence(0.7)
    .axes(axes.into_iter())
    .build()?;

// Translate to context
let translator = RuleTranslator::default();
let context = translator.to_prompt_context(&snapshot);

println!("Guidelines: {:?}", context.guidelines);
println!("Tone: {}", context.tone);
```

### Custom Store Implementation

```rust
use async_trait::async_trait;
use attuned_store::{StateStore, StoreError};
use attuned_core::StateSnapshot;

pub struct PostgresStore {
    pool: sqlx::PgPool,
}

#[async_trait]
impl StateStore for PostgresStore {
    async fn get_latest(&self, user_id: &str) -> Result<Option<StateSnapshot>, StoreError> {
        let row = sqlx::query_as!(
            StateRow,
            "SELECT * FROM user_state WHERE user_id = $1 ORDER BY updated_at DESC LIMIT 1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StoreError::Backend(e.to_string()))?;

        Ok(row.map(|r| r.into()))
    }

    async fn upsert_latest(&self, snapshot: StateSnapshot) -> Result<(), StoreError> {
        sqlx::query!(
            "INSERT INTO user_state (user_id, axes, source, confidence, updated_at)
             VALUES ($1, $2, $3, $4, NOW())
             ON CONFLICT (user_id) DO UPDATE SET
                axes = EXCLUDED.axes,
                source = EXCLUDED.source,
                confidence = EXCLUDED.confidence,
                updated_at = NOW()",
            snapshot.user_id,
            serde_json::to_value(&snapshot.axes)?,
            snapshot.source.to_string(),
            snapshot.confidence
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StoreError::Backend(e.to_string()))?;

        Ok(())
    }

    // ... other methods
}
```

---

## Best Practices

### 1. Self-Report First

Always establish baseline through explicit user preferences before relying on inference:

```python
# Good: Start with onboarding
set_state(user_id, source="self_report", axes={"warmth": 0.8})

# Later: Augment with inference
set_state(user_id, message="user's message", axes={})  # Inferred merges with self-report
```

### 2. Explicit Overrides Inferred

When both are present, explicit values win:

```python
# User explicitly says they're fine, even if message sounds anxious
set_state(user_id,
    message="I'm freaking out!!!",  # Would infer anxiety_level: 0.8
    axes={"anxiety_level": 0.2}     # But explicit wins
)
```

### 3. Handle Missing State Gracefully

```python
def get_context_safe(user_id: str) -> dict:
    response = requests.get(f"{BASE_URL}/v1/context/{user_id}")
    if response.status_code == 404:
        # Return default context for new users
        return {
            "guidelines": ["Be helpful and friendly"],
            "tone": "neutral",
            "verbosity": "moderate",
            "flags": []
        }
    return response.json()
```

### 4. Don't Over-Infer

Inference works best for:
- Formality detection (strong evidence)
- Emotional intensity (strong evidence)
- Anxiety/stress signals (moderate-strong evidence)

Avoid inferring:
- Deep personality traits from single messages
- Long-term preferences from momentary state
- Cognitive load from text alone

### 5. Respect Privacy

```python
# Good: Only store axis values, not messages
requests.post(f"{BASE_URL}/v1/state", json={
    "user_id": user_id,
    "message": message,  # Analyzed but NOT stored
    "axes": {}
})

# The message is processed for inference but Attuned only stores:
# - Axis values (aggregated metrics)
# - Confidence scores
# - Source metadata
# NOT the original message text
```

---

## Troubleshooting

### Inference Not Working

```bash
# Check if inference feature is enabled
curl http://localhost:8080/v1/infer -d '{"message": "test"}'
# If 503: Server not built with inference feature

# Rebuild with inference
cargo build -p attuned-http --features inference
```

### Low Confidence Scores

Short messages get penalized:
```python
# 5-word message → confidence factor ~0.5
response = infer("Help me please")
# {"estimates": [{"axis": "...", "confidence": 0.25}]}  # Low

# 50+ word message → confidence factor ~1.0
response = infer("I've been working on this project for weeks and I'm really worried...")
# {"estimates": [{"axis": "...", "confidence": 0.55}]}  # Higher
```

### Context Not Changing

Check if state is actually stored:
```bash
# Verify state exists
curl http://localhost:8080/v1/state/user123

# If empty, state wasn't persisted
# Make sure you're POSTing to /v1/state, not just /v1/infer
```
