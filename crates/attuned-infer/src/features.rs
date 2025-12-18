//! Fast linguistic feature extraction from text.
//!
//! This module extracts interpretable signals from natural language
//! without any ML models - just deterministic text analysis.
//!
//! Target: <100μs per message on commodity hardware.

use serde::{Deserialize, Serialize};

/// Linguistic features extracted from text.
///
/// All features are normalized to [0.0, 1.0] or are counts/ratios
/// that can be mapped to axis values.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LinguisticFeatures {
    // === Raw metrics ===
    /// Total character count.
    pub char_count: usize,
    /// Total word count.
    pub word_count: usize,
    /// Number of sentences.
    pub sentence_count: usize,

    // === Complexity indicators (→ cognitive axes) ===
    /// Average word length in characters.
    pub avg_word_length: f32,
    /// Average sentence length in words.
    pub avg_sentence_length: f32,
    /// Ratio of long words (>6 chars) to total words.
    pub long_word_ratio: f32,
    /// Flesch-Kincaid grade level approximation.
    pub reading_grade_level: f32,

    // === Emotional indicators (→ emotional axes) ===
    /// Ratio of exclamation marks to sentences.
    pub exclamation_ratio: f32,
    /// Ratio of question marks to sentences.
    pub question_ratio: f32,
    /// Count of ALL CAPS words (excluding single letters/acronyms).
    pub caps_word_count: usize,
    /// Ratio of caps words to total words.
    pub caps_ratio: f32,

    // === Hedge/uncertainty markers (→ anxiety, assertiveness) ===
    /// Count of hedge words ("maybe", "perhaps", "I think", etc.).
    pub hedge_count: usize,
    /// Hedge words per sentence.
    pub hedge_density: f32,
    /// Count of certainty markers ("definitely", "absolutely", "clearly").
    pub certainty_count: usize,

    // === Anxiety/stress indicators (→ anxiety axes) ===
    // Research-validated features from Dreaddit validation (TASK-015)
    /// Count of negative emotion words ("worried", "anxious", "stressed", etc.).
    pub negative_emotion_count: usize,
    /// Negative emotion words per sentence.
    pub negative_emotion_density: f32,
    /// Count of absolutist words ("always", "never", "everything", etc.).
    /// Linked to anxious/depressive thinking patterns.
    pub absolutist_count: usize,
    /// Absolutist words per sentence.
    pub absolutist_density: f32,

    // === Formality indicators (→ social axes) ===
    /// Ratio of contractions to potential contraction sites.
    pub contraction_ratio: f32,
    /// Count of politeness markers ("please", "thank you", "appreciate").
    pub politeness_count: usize,
    /// First-person pronoun ratio ("I", "me", "my").
    pub first_person_ratio: f32,

    // === Urgency indicators (→ urgency, control axes) ===
    /// Count of urgency words ("urgent", "asap", "immediately", "now").
    pub urgency_word_count: usize,
    /// Count of imperative sentence starters.
    pub imperative_count: usize,

    // === Verbosity indicators (→ preference axes) ===
    /// Words per minute (if timing available, else 0).
    pub words_per_minute: f32,
    /// Ratio of filler words ("just", "actually", "basically", "really").
    pub filler_ratio: f32,
}

/// Configuration for the linguistic extractor.
#[derive(Clone, Debug)]
pub struct ExtractorConfig {
    /// Words considered "long" for complexity analysis.
    pub long_word_threshold: usize,
    /// Minimum word length to count as CAPS (filters "I", "A", acronyms).
    pub min_caps_word_length: usize,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            long_word_threshold: 6,
            min_caps_word_length: 3,
        }
    }
}

/// Fast linguistic feature extractor.
///
/// Extracts interpretable signals from text in sub-millisecond time.
/// No ML models, no external dependencies - just deterministic parsing.
#[derive(Clone, Debug, Default)]
pub struct LinguisticExtractor {
    config: ExtractorConfig,
}

impl LinguisticExtractor {
    /// Create a new extractor with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an extractor with custom configuration.
    pub fn with_config(config: ExtractorConfig) -> Self {
        Self { config }
    }

    /// Extract linguistic features from text.
    ///
    /// This is the main entry point. Designed to be fast (<100μs typical).
    pub fn extract(&self, text: &str) -> LinguisticFeatures {
        let chars: Vec<char> = text.chars().collect();
        let char_count = chars.len();

        if char_count == 0 {
            return LinguisticFeatures::default();
        }

        // Tokenize into words (simple whitespace + punctuation split)
        let words = self.tokenize_words(text);
        let word_count = words.len();

        if word_count == 0 {
            return LinguisticFeatures {
                char_count,
                ..Default::default()
            };
        }

        // Sentence detection
        let sentence_count = self.count_sentences(text).max(1);

        // Complexity metrics
        let total_word_chars: usize = words.iter().map(|w| w.len()).sum();
        let avg_word_length = total_word_chars as f32 / word_count as f32;
        let avg_sentence_length = word_count as f32 / sentence_count as f32;
        let long_words = words
            .iter()
            .filter(|w| w.len() > self.config.long_word_threshold)
            .count();
        let long_word_ratio = long_words as f32 / word_count as f32;

        // Flesch-Kincaid grade level approximation
        let syllables = self.estimate_syllables(&words);
        let reading_grade_level = self.flesch_kincaid_grade(word_count, sentence_count, syllables);

        // Punctuation analysis
        let exclamation_count = chars.iter().filter(|&&c| c == '!').count();
        let question_count = chars.iter().filter(|&&c| c == '?').count();
        let exclamation_ratio = exclamation_count as f32 / sentence_count as f32;
        let question_ratio = question_count as f32 / sentence_count as f32;

        // CAPS analysis
        let caps_words: Vec<_> = words
            .iter()
            .filter(|w| {
                w.len() >= self.config.min_caps_word_length
                    && w.chars().all(|c| c.is_uppercase() || !c.is_alphabetic())
                    && w.chars().any(|c| c.is_alphabetic())
            })
            .collect();
        let caps_word_count = caps_words.len();
        let caps_ratio = caps_word_count as f32 / word_count as f32;

        // Hedge words
        let hedge_count = self.count_hedge_words(&words);
        let hedge_density = hedge_count as f32 / sentence_count as f32;

        // Certainty markers
        let certainty_count = self.count_certainty_markers(&words);

        // Formality indicators
        let contraction_ratio = self.estimate_contraction_ratio(text);
        let politeness_count = self.count_politeness_markers(&words);
        let first_person_ratio = self.first_person_ratio(&words);

        // Urgency indicators
        let urgency_word_count = self.count_urgency_words(&words);
        let imperative_count = self.count_imperatives(text);

        // Filler words
        let filler_count = self.count_filler_words(&words);
        let filler_ratio = filler_count as f32 / word_count as f32;

        // Anxiety/stress indicators (TASK-016: research-validated)
        let negative_emotion_count = self.count_negative_emotion_words(&words);
        let negative_emotion_density = negative_emotion_count as f32 / sentence_count as f32;
        let absolutist_count = self.count_absolutist_words(&words);
        let absolutist_density = absolutist_count as f32 / sentence_count as f32;

        LinguisticFeatures {
            char_count,
            word_count,
            sentence_count,
            avg_word_length,
            avg_sentence_length,
            long_word_ratio,
            reading_grade_level,
            exclamation_ratio,
            question_ratio,
            caps_word_count,
            caps_ratio,
            hedge_count,
            hedge_density,
            certainty_count,
            negative_emotion_count,
            negative_emotion_density,
            absolutist_count,
            absolutist_density,
            contraction_ratio,
            politeness_count,
            first_person_ratio,
            urgency_word_count,
            imperative_count,
            words_per_minute: 0.0, // Requires external timing
            filler_ratio,
        }
    }

    /// Simple word tokenization.
    fn tokenize_words<'a>(&self, text: &'a str) -> Vec<&'a str> {
        text.split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == ':')
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\''))
            .filter(|w| !w.is_empty())
            .collect()
    }

    /// Count sentences (approximation based on terminal punctuation).
    fn count_sentences(&self, text: &str) -> usize {
        let mut count = 0;
        let mut prev_char = ' ';

        for c in text.chars() {
            if (c == '.' || c == '!' || c == '?') && prev_char != '.' {
                count += 1;
            }
            prev_char = c;
        }

        // If no terminal punctuation, count as 1 sentence if there's content
        if count == 0 && !text.trim().is_empty() {
            count = 1;
        }

        count
    }

    /// Estimate total syllables (simple heuristic).
    fn estimate_syllables(&self, words: &[&str]) -> usize {
        words.iter().map(|w| self.syllables_in_word(w)).sum()
    }

    /// Count syllables in a word (rough approximation).
    fn syllables_in_word(&self, word: &str) -> usize {
        let word = word.to_lowercase();
        let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];

        let mut count = 0;
        let mut prev_was_vowel = false;

        for c in word.chars() {
            let is_vowel = vowels.contains(&c);
            if is_vowel && !prev_was_vowel {
                count += 1;
            }
            prev_was_vowel = is_vowel;
        }

        // Silent 'e' adjustment
        if word.ends_with('e') && count > 1 {
            count -= 1;
        }

        count.max(1)
    }

    /// Flesch-Kincaid grade level formula.
    fn flesch_kincaid_grade(&self, words: usize, sentences: usize, syllables: usize) -> f32 {
        if words == 0 || sentences == 0 {
            return 0.0;
        }

        let asl = words as f32 / sentences as f32; // Average sentence length
        let asw = syllables as f32 / words as f32; // Average syllables per word

        // FK Grade = 0.39 * ASL + 11.8 * ASW - 15.59
        (0.39 * asl + 11.8 * asw - 15.59).clamp(0.0, 20.0)
    }

    /// Count hedge/uncertainty words.
    fn count_hedge_words(&self, words: &[&str]) -> usize {
        const HEDGE_WORDS: &[&str] = &[
            "maybe",
            "perhaps",
            "possibly",
            "probably",
            "might",
            "could",
            "seem",
            "seems",
            "seemed",
            "appear",
            "appears",
            "appeared",
            "think",
            "believe",
            "guess",
            "suppose",
            "assume",
            "somewhat",
            "fairly",
            "rather",
            "quite",
            "sort",
            "kind",
            "mostly",
            "generally",
            "usually",
            "often",
            "uncertain",
            "unsure",
            "unclear",
        ];

        words
            .iter()
            .filter(|w| HEDGE_WORDS.contains(&w.to_lowercase().as_str()))
            .count()
    }

    /// Count certainty/confidence markers.
    fn count_certainty_markers(&self, words: &[&str]) -> usize {
        const CERTAINTY_WORDS: &[&str] = &[
            "definitely",
            "absolutely",
            "certainly",
            "clearly",
            "obviously",
            "surely",
            "undoubtedly",
            "always",
            "never",
            "must",
            "will",
            "proven",
            "fact",
            "guarantee",
            "positive",
            "confident",
        ];

        words
            .iter()
            .filter(|w| CERTAINTY_WORDS.contains(&w.to_lowercase().as_str()))
            .count()
    }

    /// Estimate contraction usage ratio.
    fn estimate_contraction_ratio(&self, text: &str) -> f32 {
        let contractions = ["n't", "'re", "'ve", "'ll", "'m", "'d", "'s"];
        let text_lower = text.to_lowercase();

        let contraction_count = contractions
            .iter()
            .map(|c| text_lower.matches(c).count())
            .sum::<usize>();

        // Normalize by approximate opportunities (auxiliary verbs, pronouns)
        let opportunities = text_lower.matches(" i ").count()
            + text_lower.matches(" you ").count()
            + text_lower.matches(" we ").count()
            + text_lower.matches(" they ").count()
            + text_lower.matches(" he ").count()
            + text_lower.matches(" she ").count()
            + text_lower.matches(" it ").count()
            + text_lower.matches(" not ").count()
            + text_lower.matches(" will ").count()
            + text_lower.matches(" would ").count()
            + text_lower.matches(" have ").count()
            + text_lower.matches(" has ").count()
            + text_lower.matches(" is ").count()
            + text_lower.matches(" are ").count();

        if opportunities == 0 {
            return 0.5; // No signal, return neutral
        }

        (contraction_count as f32 / opportunities as f32).clamp(0.0, 1.0)
    }

    /// Count politeness markers.
    fn count_politeness_markers(&self, words: &[&str]) -> usize {
        const POLITE_WORDS: &[&str] = &[
            "please",
            "thanks",
            "thank",
            "appreciate",
            "grateful",
            "sorry",
            "apologies",
            "apologize",
            "excuse",
            "pardon",
            "kindly",
            "welcome",
            "regards",
        ];

        words
            .iter()
            .filter(|w| POLITE_WORDS.contains(&w.to_lowercase().as_str()))
            .count()
    }

    /// Calculate first-person pronoun ratio.
    fn first_person_ratio(&self, words: &[&str]) -> f32 {
        const FIRST_PERSON: &[&str] =
            &["i", "me", "my", "mine", "myself", "we", "us", "our", "ours"];

        let count = words
            .iter()
            .filter(|w| FIRST_PERSON.contains(&w.to_lowercase().as_str()))
            .count();

        if words.is_empty() {
            return 0.0;
        }

        count as f32 / words.len() as f32
    }

    /// Count urgency words.
    fn count_urgency_words(&self, words: &[&str]) -> usize {
        const URGENCY_WORDS: &[&str] = &[
            "urgent",
            "urgently",
            "asap",
            "immediately",
            "emergency",
            "critical",
            "crucial",
            "vital",
            "essential",
            "pressing",
            "now",
            "today",
            "deadline",
            "hurry",
            "quick",
            "quickly",
            "fast",
            "rush",
            "priority",
            "important",
        ];

        words
            .iter()
            .filter(|w| URGENCY_WORDS.contains(&w.to_lowercase().as_str()))
            .count()
    }

    /// Count imperative sentence starters.
    fn count_imperatives(&self, text: &str) -> usize {
        const IMPERATIVE_STARTERS: &[&str] = &[
            "do ", "don't ", "please ", "make ", "let ", "get ", "take ", "give ", "tell ",
            "show ", "help ", "send ", "check ", "read ", "write ", "call ", "stop ", "start ",
            "go ", "come ",
        ];

        let text_lower = text.to_lowercase();
        let mut count = 0;

        // Check start of text
        for starter in IMPERATIVE_STARTERS {
            if text_lower.starts_with(starter) {
                count += 1;
                break;
            }
        }

        // Check after sentence boundaries
        for boundary in [". ", "! ", "? "] {
            for part in text_lower.split(boundary) {
                let trimmed = part.trim();
                for starter in IMPERATIVE_STARTERS {
                    if trimmed.starts_with(starter) {
                        count += 1;
                        break;
                    }
                }
            }
        }

        count
    }

    /// Count filler words.
    fn count_filler_words(&self, words: &[&str]) -> usize {
        const FILLER_WORDS: &[&str] = &[
            "just",
            "actually",
            "basically",
            "really",
            "very",
            "literally",
            "honestly",
            "like",
            "so",
            "well",
            "anyway",
            "anyways",
            "totally",
            "completely",
            "definitely",
            "absolutely",
        ];

        words
            .iter()
            .filter(|w| FILLER_WORDS.contains(&w.to_lowercase().as_str()))
            .count()
    }

    /// Count negative emotion words (research-validated for anxiety/stress detection).
    ///
    /// Based on LIWC Anxiety category and Dreaddit validation (TASK-015).
    /// These showed r=0.266 correlation with stress labels.
    fn count_negative_emotion_words(&self, words: &[&str]) -> usize {
        const NEGATIVE_EMOTION_WORDS: &[&str] = &[
            // Anxiety-specific (LIWC Anxiety category)
            "worried",
            "worry",
            "worries",
            "worrying",
            "anxious",
            "anxiety",
            "nervous",
            "nervously",
            "afraid",
            "fear",
            "fears",
            "feared",
            "fearful",
            "scared",
            "scary",
            "panic",
            "panicked",
            "panicking",
            "stressed",
            "stress",
            "stressful",
            "tense",
            "tension",
            "uneasy",
            "dread",
            "dreading",
            "dreaded",
            // General negative affect
            "upset",
            "upsetting",
            "frustrated",
            "frustrating",
            "frustration",
            "annoyed",
            "annoying",
            "annoyance",
            "angry",
            "anger",
            "mad",
            "sad",
            "sadness",
            "depressed",
            "depressing",
            "depression",
            "hopeless",
            "hopelessness",
            "miserable",
            "terrible",
            "terribly",
            "awful",
            "horrible",
            "horribly",
            "worst",
            // Distress markers
            "struggling",
            "struggle",
            "struggles",
            "suffering",
            "suffer",
            "suffers",
            "overwhelmed",
            "overwhelming",
            "exhausted",
            "exhausting",
            "exhaustion",
            "desperate",
            "desperately",
            "desperation",
            "helpless",
            "helplessness",
            "stuck",
            "lost",
        ];

        words
            .iter()
            .filter(|w| NEGATIVE_EMOTION_WORDS.contains(&w.to_lowercase().as_str()))
            .count()
    }

    /// Count absolutist words (linked to anxious/depressive thinking).
    ///
    /// Research shows "always", "never", "completely" etc. correlate with
    /// anxious and depressive cognitive patterns.
    fn count_absolutist_words(&self, words: &[&str]) -> usize {
        const ABSOLUTIST_WORDS: &[&str] = &[
            "always",
            "never",
            "nothing",
            "everything",
            "completely",
            "totally",
            "absolutely",
            "entirely",
            "impossible",
            "perfectly",
            "forever",
            "everyone",
            "nobody",
            "nowhere",
            "anywhere",
            "constant",
            "constantly",
        ];

        words
            .iter()
            .filter(|w| ABSOLUTIST_WORDS.contains(&w.to_lowercase().as_str()))
            .count()
    }
}

impl LinguisticFeatures {
    /// Get a normalized complexity score [0, 1].
    ///
    /// Combines multiple complexity signals.
    pub fn complexity_score(&self) -> f32 {
        // Weighted combination of complexity indicators
        let grade_component = (self.reading_grade_level / 16.0).clamp(0.0, 1.0);
        let length_component = (self.avg_sentence_length / 30.0).clamp(0.0, 1.0);
        let word_component = (self.avg_word_length / 8.0).clamp(0.0, 1.0);

        (0.4 * grade_component + 0.3 * length_component + 0.3 * word_component).clamp(0.0, 1.0)
    }

    /// Get a normalized emotional intensity score [0, 1].
    pub fn emotional_intensity(&self) -> f32 {
        let exclaim = (self.exclamation_ratio * 2.0).clamp(0.0, 1.0);
        let caps = (self.caps_ratio * 10.0).clamp(0.0, 1.0);

        (0.6 * exclaim + 0.4 * caps).clamp(0.0, 1.0)
    }

    /// Get a normalized uncertainty score [0, 1].
    ///
    /// Original v1 implementation using hedges and questions.
    pub fn uncertainty_score(&self) -> f32 {
        let hedge = (self.hedge_density / 2.0).clamp(0.0, 1.0);
        let question = (self.question_ratio).clamp(0.0, 1.0);
        let certainty_inverse = 1.0 - (self.certainty_count as f32 / 3.0).clamp(0.0, 1.0);

        (0.5 * hedge + 0.3 * question + 0.2 * certainty_inverse).clamp(0.0, 1.0)
    }

    /// Get a normalized anxiety/stress score [0, 1].
    ///
    /// Research-validated score combining:
    /// - Negative emotion words (r=0.266 with stress, highest predictor)
    /// - First-person pronouns (r=0.321 with stress, self-focus indicator)
    /// - Hedge/uncertainty markers (r=0.103 with stress)
    ///
    /// Based on Dreaddit validation (TASK-015) which showed proposed_v2
    /// improves F1 by 16.7% over uncertainty_score alone.
    pub fn anxiety_score(&self) -> f32 {
        // Weights from Dreaddit validation - feature coefficients
        // first_person_ratio: 0.718, negative_emotion_density: 0.696
        let neg_emotion = (self.negative_emotion_density / 2.0).clamp(0.0, 1.0);
        let first_person = (self.first_person_ratio * 5.0).clamp(0.0, 1.0); // Scale up since typical is ~0.1
        let uncertainty = self.uncertainty_score();
        let absolutist = (self.absolutist_density / 2.0).clamp(0.0, 1.0);

        // Weighted combination based on research validation
        (0.35 * neg_emotion + 0.35 * first_person + 0.20 * uncertainty + 0.10 * absolutist)
            .clamp(0.0, 1.0)
    }

    /// Get a normalized urgency score [0, 1].
    pub fn urgency_score(&self) -> f32 {
        let words = (self.urgency_word_count as f32 / 3.0).clamp(0.0, 1.0);
        let imperatives = (self.imperative_count as f32 / 2.0).clamp(0.0, 1.0);
        let exclaim = (self.exclamation_ratio).clamp(0.0, 1.0);

        (0.5 * words + 0.3 * imperatives + 0.2 * exclaim).clamp(0.0, 1.0)
    }

    /// Get a normalized formality score [0, 1].
    ///
    /// Higher = more formal.
    pub fn formality_score(&self) -> f32 {
        // Less contractions = more formal
        let contraction_inverse = 1.0 - self.contraction_ratio;
        // More complexity = more formal
        let complexity = self.complexity_score();
        // Less caps/exclamation = more formal
        let emotional_inverse = 1.0 - self.emotional_intensity();

        (0.4 * contraction_inverse + 0.3 * complexity + 0.3 * emotional_inverse).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_extraction() {
        let extractor = LinguisticExtractor::new();
        let features = extractor.extract("Hello, this is a simple test.");

        assert_eq!(features.word_count, 6);
        assert_eq!(features.sentence_count, 1);
        assert!(features.avg_word_length > 0.0);
    }

    #[test]
    fn test_empty_text() {
        let extractor = LinguisticExtractor::new();
        let features = extractor.extract("");

        assert_eq!(features.word_count, 0);
        assert_eq!(features.char_count, 0);
    }

    #[test]
    fn test_urgency_detection() {
        let extractor = LinguisticExtractor::new();
        let features = extractor.extract("URGENT! I need help immediately! This is critical!!!");

        assert!(features.urgency_word_count >= 3);
        assert!(features.urgency_score() > 0.5);
        // 4+ exclamation marks across 3 sentences
        assert!(features.exclamation_ratio >= 1.0);
    }

    #[test]
    fn test_hedge_detection() {
        let extractor = LinguisticExtractor::new();
        let features = extractor.extract(
            "I think maybe this might possibly work, but I'm not sure. Perhaps we should try.",
        );

        assert!(features.hedge_count >= 4);
        assert!(features.uncertainty_score() > 0.3);
    }

    #[test]
    fn test_formality_contrast() {
        let extractor = LinguisticExtractor::new();

        let casual = extractor.extract("hey what's up! can't wait to see ya there lol");
        let formal = extractor.extract(
            "I am writing to inquire about the status of my application. Thank you for your consideration.",
        );

        assert!(formal.formality_score() > casual.formality_score());
    }

    #[test]
    fn test_caps_detection() {
        let extractor = LinguisticExtractor::new();
        let features = extractor.extract("This is VERY IMPORTANT and you MUST read it NOW!");

        assert!(features.caps_word_count >= 3);
        assert!(features.caps_ratio > 0.1);
    }

    #[test]
    fn test_complexity_score() {
        let extractor = LinguisticExtractor::new();

        let simple = extractor.extract("I like cats. They are cute.");
        let complex = extractor.extract(
            "The epistemological implications of quantum mechanical phenomena necessitate \
             a fundamental reconsideration of our ontological presuppositions regarding \
             the nature of observable reality.",
        );

        assert!(complex.complexity_score() > simple.complexity_score());
    }

    #[test]
    fn test_flesch_kincaid() {
        let extractor = LinguisticExtractor::new();
        let features = extractor.extract("The cat sat on the mat.");

        // Simple sentence should have low grade level
        assert!(features.reading_grade_level < 5.0);
    }

    #[test]
    fn test_negative_emotion_detection() {
        let extractor = LinguisticExtractor::new();

        let anxious = extractor.extract(
            "I'm so worried and stressed about this. I feel anxious and scared. \
             The situation is terrible and I'm struggling to cope.",
        );
        let calm = extractor.extract(
            "The project is going well. We have made good progress and the team \
             is confident about the outcome.",
        );

        assert!(anxious.negative_emotion_count >= 5);
        assert!(anxious.negative_emotion_density > 0.5);
        assert!(calm.negative_emotion_count <= 1);
        assert!(anxious.anxiety_score() > calm.anxiety_score());
    }

    #[test]
    fn test_absolutist_detection() {
        let extractor = LinguisticExtractor::new();

        let absolutist = extractor.extract(
            "Everything is always terrible. Nothing ever works. I can never do anything right.",
        );
        let balanced =
            extractor.extract("Sometimes things work out. Other times they don't. It varies.");

        assert!(absolutist.absolutist_count >= 4);
        assert!(balanced.absolutist_count <= 1);
    }

    #[test]
    fn test_anxiety_score_vs_uncertainty() {
        let extractor = LinguisticExtractor::new();

        // Message with high anxiety but low hedging
        let stressed = extractor.extract(
            "I am so stressed and worried. I feel overwhelmed and exhausted. \
             This is terrible and I don't know what to do.",
        );

        // Message with hedging but no anxiety markers
        let hedging = extractor.extract(
            "I think maybe we could possibly try this approach. \
             Perhaps it might work, but I'm not entirely sure.",
        );

        // anxiety_score should be higher for stressed message
        assert!(stressed.anxiety_score() > hedging.anxiety_score());
        // uncertainty_score should be higher for hedging message
        assert!(hedging.uncertainty_score() > stressed.uncertainty_score());
    }
}
