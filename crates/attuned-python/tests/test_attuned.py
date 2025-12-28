"""Tests for Attuned Python bindings."""

import pytest
from attuned import (
    StateSnapshot,
    StateSnapshotBuilder,
    Source,
    PromptContext,
    Verbosity,
    RuleTranslator,
    Thresholds,
    AxisDefinition,
    AxisCategory,
    get_axis,
    is_valid_axis_name,
    get_axis_names,
    get_all_axes,
    CANONICAL_AXES,
)


class TestSource:
    """Tests for Source enum."""

    def test_source_values(self):
        assert Source.SelfReport == 0
        assert Source.Inferred == 1
        assert Source.Mixed == 2

    def test_source_from_str(self):
        assert Source.from_str("self_report") == Source.SelfReport
        assert Source.from_str("inferred") == Source.Inferred
        assert Source.from_str("mixed") == Source.Mixed

    def test_source_from_str_invalid(self):
        with pytest.raises(ValueError):
            Source.from_str("invalid")

    def test_source_str(self):
        assert str(Source.SelfReport) == "self_report"


class TestStateSnapshot:
    """Tests for StateSnapshot and builder."""

    def test_builder_basic(self):
        snapshot = StateSnapshot.builder() \
            .user_id("test_user") \
            .axis("warmth", 0.7) \
            .build()

        assert snapshot.user_id == "test_user"
        assert snapshot.get_axis("warmth") == pytest.approx(0.7, rel=1e-5)

    def test_builder_with_source(self):
        snapshot = StateSnapshot.builder() \
            .user_id("test_user") \
            .source(Source.Inferred) \
            .confidence(0.8) \
            .build()

        assert snapshot.source == Source.Inferred
        assert snapshot.confidence == pytest.approx(0.8, rel=1e-5)

    def test_builder_multiple_axes(self):
        snapshot = StateSnapshot.builder() \
            .user_id("test_user") \
            .axis("warmth", 0.7) \
            .axis("formality", 0.3) \
            .axis("cognitive_load", 0.9) \
            .build()

        assert len(snapshot.axes) == 3
        assert snapshot.get_axis("warmth") == pytest.approx(0.7, rel=1e-5)
        assert snapshot.get_axis("formality") == pytest.approx(0.3, rel=1e-5)

    def test_builder_axes_dict(self):
        snapshot = StateSnapshot.builder() \
            .user_id("test_user") \
            .axes({"warmth": 0.6, "formality": 0.4}) \
            .build()

        assert snapshot.get_axis("warmth") == pytest.approx(0.6, rel=1e-5)
        assert snapshot.get_axis("formality") == pytest.approx(0.4, rel=1e-5)

    def test_get_axis_default(self):
        snapshot = StateSnapshot.builder() \
            .user_id("test_user") \
            .build()

        # Unknown axis returns default 0.5
        assert snapshot.get_axis("warmth") == pytest.approx(0.5, rel=1e-5)

    def test_get_axis_opt(self):
        snapshot = StateSnapshot.builder() \
            .user_id("test_user") \
            .axis("warmth", 0.7) \
            .build()

        assert snapshot.get_axis_opt("warmth") == pytest.approx(0.7, rel=1e-5)
        assert snapshot.get_axis_opt("unknown") is None

    def test_missing_user_id(self):
        with pytest.raises(ValueError, match="user_id"):
            StateSnapshot.builder().build()

    def test_invalid_axis_value(self):
        with pytest.raises(ValueError):
            StateSnapshot.builder() \
                .user_id("test_user") \
                .axis("warmth", 1.5) \
                .build()

    def test_json_roundtrip(self):
        original = StateSnapshot.builder() \
            .user_id("test_user") \
            .source(Source.SelfReport) \
            .axis("warmth", 0.7) \
            .build()

        json_str = original.to_json()
        restored = StateSnapshot.from_json(json_str)

        assert restored.user_id == original.user_id
        assert restored.get_axis("warmth") == pytest.approx(0.7, rel=1e-5)


class TestRuleTranslator:
    """Tests for RuleTranslator."""

    def test_default_translator(self):
        translator = RuleTranslator()
        assert translator.thresholds.hi == pytest.approx(0.7, rel=1e-5)
        assert translator.thresholds.lo == pytest.approx(0.3, rel=1e-5)

    def test_custom_thresholds(self):
        thresholds = Thresholds(hi=0.8, lo=0.2)
        translator = RuleTranslator(thresholds=thresholds)
        assert translator.thresholds.hi == pytest.approx(0.8, rel=1e-5)

    def test_invalid_thresholds(self):
        with pytest.raises(ValueError):
            Thresholds(hi=0.3, lo=0.7)  # hi must be > lo

    def test_to_prompt_context_basic(self):
        snapshot = StateSnapshot.builder() \
            .user_id("test_user") \
            .build()

        translator = RuleTranslator()
        context = translator.to_prompt_context(snapshot)

        # Base guidelines always present
        assert len(context.guidelines) >= 3
        assert any("suggestions" in g.lower() for g in context.guidelines)

    def test_high_cognitive_load_flag(self):
        snapshot = StateSnapshot.builder() \
            .user_id("test_user") \
            .axis("cognitive_load", 0.9) \
            .build()

        translator = RuleTranslator()
        context = translator.to_prompt_context(snapshot)

        assert "high_cognitive_load" in context.flags

    def test_warm_tone(self):
        snapshot = StateSnapshot.builder() \
            .user_id("test_user") \
            .axis("warmth", 0.9) \
            .build()

        translator = RuleTranslator()
        context = translator.to_prompt_context(snapshot)

        assert "warm" in context.tone

    def test_verbosity_levels(self):
        translator = RuleTranslator()

        low = StateSnapshot.builder().user_id("u").axis("verbosity_preference", 0.1).build()
        high = StateSnapshot.builder().user_id("u").axis("verbosity_preference", 0.9).build()
        med = StateSnapshot.builder().user_id("u").axis("verbosity_preference", 0.5).build()

        assert translator.to_prompt_context(low).verbosity == Verbosity.Low
        assert translator.to_prompt_context(high).verbosity == Verbosity.High
        assert translator.to_prompt_context(med).verbosity == Verbosity.Medium


class TestPromptContext:
    """Tests for PromptContext."""

    def test_format_for_prompt(self):
        snapshot = StateSnapshot.builder() \
            .user_id("test_user") \
            .axis("warmth", 0.9) \
            .axis("cognitive_load", 0.9) \
            .build()

        translator = RuleTranslator()
        context = translator.to_prompt_context(snapshot)

        formatted = context.format_for_prompt()
        assert "## Interaction Guidelines" in formatted
        assert "Tone:" in formatted
        assert "Verbosity:" in formatted

    def test_json_roundtrip(self):
        snapshot = StateSnapshot.builder() \
            .user_id("test_user") \
            .axis("warmth", 0.9) \
            .build()

        translator = RuleTranslator()
        original = translator.to_prompt_context(snapshot)

        json_str = original.to_json()
        restored = PromptContext.from_json(json_str)

        assert restored.tone == original.tone
        assert restored.guidelines == original.guidelines


class TestAxisDefinition:
    """Tests for axis definitions and governance."""

    def test_get_axis(self):
        axis = get_axis("cognitive_load")
        assert axis is not None
        assert axis.name == "cognitive_load"
        assert axis.category == AxisCategory.Cognitive

    def test_get_axis_not_found(self):
        assert get_axis("nonexistent") is None

    def test_is_valid_axis_name(self):
        # Tests naming conventions, not canonical existence
        assert is_valid_axis_name("cognitive_load") is True
        assert is_valid_axis_name("warmth") is True
        assert is_valid_axis_name("valid_custom_axis") is True
        # Invalid names
        assert is_valid_axis_name("_starts_underscore") is False
        assert is_valid_axis_name("ends_underscore_") is False
        assert is_valid_axis_name("HAS_UPPERCASE") is False
        assert is_valid_axis_name("") is False

    def test_get_axis_names(self):
        names = get_axis_names()
        assert len(names) == 23
        assert "cognitive_load" in names
        assert "warmth" in names

    def test_get_all_axes(self):
        axes = get_all_axes()
        assert len(axes) == 23
        assert all(isinstance(a, AxisDefinition) for a in axes)

    def test_canonical_axes_constant(self):
        assert len(CANONICAL_AXES) == 23

    def test_axis_governance_fields(self):
        axis = get_axis("cognitive_load")

        # All governance fields should be populated
        assert axis.description
        assert axis.low_anchor
        assert axis.high_anchor
        assert len(axis.intent) > 0
        assert len(axis.forbidden_uses) > 0
        assert axis.since

    def test_axis_format_summary(self):
        axis = get_axis("cognitive_load")
        summary = axis.format_summary()

        assert "cognitive_load" in summary
        assert "Intended Uses" in summary
        assert "FORBIDDEN Uses" in summary


class TestAxisCategories:
    """Tests for axis categories."""

    def test_all_categories_represented(self):
        axes = get_all_axes()
        # Use string representation since enums may not be hashable
        categories = {str(a.category) for a in axes}

        assert "cognitive" in categories
        assert "emotional" in categories
        assert "social" in categories
        assert "preferences" in categories
        assert "control" in categories
        assert "safety" in categories

    def test_category_counts(self):
        axes = get_all_axes()
        by_category = {}
        for a in axes:
            cat = str(a.category)
            by_category[cat] = by_category.get(cat, 0) + 1

        assert by_category["cognitive"] == 4
        assert by_category["emotional"] == 4
        assert by_category["social"] == 5
        assert by_category["preferences"] == 4
        assert by_category["control"] == 4
        assert by_category["safety"] == 2


class TestInference:
    """Tests for inference functionality."""

    def test_infer_function(self):
        from attuned import infer
        
        state = infer("I need this done ASAP!!!")
        
        assert len(state) > 0
        assert not state.is_empty()
        
        # Should detect urgency
        urgency = state.get("urgency_sensitivity")
        assert urgency is not None
        assert 0.0 <= urgency.value <= 1.0
        assert 0.0 <= urgency.confidence <= 1.0
    
    def test_inference_engine(self):
        from attuned import InferenceEngine
        
        engine = InferenceEngine()
        state = engine.infer("Help me please!")
        
        assert len(state) > 0
        for estimate in state.all():
            assert estimate.axis is not None
            assert 0.0 <= estimate.value <= 1.0
            assert 0.0 <= estimate.confidence <= 1.0
    
    def test_extract_features(self):
        from attuned import extract_features
        
        features = extract_features("This is a test sentence! Another one here.")
        
        assert features.word_count == 8
        assert features.sentence_count == 2
        assert features.exclamation_ratio > 0
        assert 0.0 <= features.urgency_score() <= 1.0
        assert 0.0 <= features.formality_score() <= 1.0
    
    def test_inference_source(self):
        from attuned import infer
        
        state = infer("I definitely need this NOW!")
        
        for estimate in state.all():
            source = estimate.source
            # All inferred values should not be self-report
            assert source.is_inferred()
            assert not source.is_self_report()
            assert source.source_type == "linguistic"
            assert source.features_used is not None
    
    def test_self_report_override(self):
        from attuned import infer
        
        state = infer("I am calm and relaxed.")
        
        # Override with self-report
        state.override_with_self_report("anxiety_level", 0.9)
        
        anxiety = state.get("anxiety_level")
        assert anxiety is not None
        assert anxiety.value == pytest.approx(0.9, abs=0.001)
        assert anxiety.confidence == pytest.approx(1.0, abs=0.001)
        assert anxiety.source.is_self_report()
    
    def test_inferred_state_to_dict(self):
        from attuned import infer
        
        state = infer("Please help me!")
        
        # Simple dict
        simple = state.to_dict()
        assert isinstance(simple, dict)
        for key, value in simple.items():
            assert isinstance(key, str)
            assert isinstance(value, float)
            assert 0.0 <= value <= 1.0
    
    def test_confidence_bounded(self):
        from attuned import infer
        
        state = infer("I ABSOLUTELY NEED THIS RIGHT NOW!!!")
        
        for estimate in state.all():
            # Inferred confidence should be capped at 0.7
            if estimate.source.is_inferred():
                assert estimate.confidence <= 0.7
    
    def test_axis_estimate_json(self):
        from attuned import infer
        import json
        
        state = infer("Hello there!")
        
        for estimate in state.all():
            json_str = estimate.to_json()
            parsed = json.loads(json_str)
            assert "axis" in parsed
            assert "value" in parsed
            assert "confidence" in parsed
            break  # Only test first one
    
    def test_linguistic_features_computed_scores(self):
        from attuned import extract_features
        
        # Formal text
        formal = extract_features("I hereby request your immediate attention to this matter.")
        
        # Informal text
        informal = extract_features("hey can u help me plz?? lol")
        
        # Formal should have higher formality score
        assert formal.formality_score() > informal.formality_score()
    
    def test_empty_text(self):
        from attuned import infer, extract_features
        
        # Should not crash on empty text
        state = infer("")
        features = extract_features("")
        
        assert features.word_count == 0
        assert features.sentence_count == 0
