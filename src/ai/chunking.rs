//! Text chunking for embedding generation.
//!
//! This module provides utilities for splitting text into overlapping chunks
//! suitable for embedding generation and semantic search.

use tracing::debug;

/// Chunks text into overlapping segments.
///
/// Uses a word-based sliding window approach where each chunk contains
/// approximately `chunk_size` words, with `overlap` words overlapping
/// between consecutive chunks.
///
/// # Arguments
///
/// * `text` - Text to chunk
/// * `chunk_size` - Target number of words per chunk
/// * `overlap` - Number of words to overlap between chunks
///
/// # Returns
///
/// A vector of text chunks. Returns empty vector if text is empty.
///
/// # Examples
///
/// ```
/// use ponder::ai::chunking::chunk_text;
///
/// let text = "word1 word2 word3 word4 word5 word6";
/// let chunks = chunk_text(text, 3, 1);
/// assert_eq!(chunks.len(), 3); // 6 words with chunk_size=3, overlap=1 produces 3 chunks
/// ```
pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    if overlap >= chunk_size {
        debug!(
            "Warning: overlap ({}) >= chunk_size ({}), using overlap = chunk_size - 1",
            overlap, chunk_size
        );
        let overlap = chunk_size.saturating_sub(1);
        return chunk_text_internal(text, chunk_size, overlap);
    }

    chunk_text_internal(text, chunk_size, overlap)
}

fn chunk_text_internal(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();

    if words.len() <= chunk_size {
        // Text fits in a single chunk
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let step = chunk_size - overlap;
    let mut start = 0;

    while start < words.len() {
        let end = (start + chunk_size).min(words.len());
        let chunk = words[start..end].join(" ");
        chunks.push(chunk);

        if end >= words.len() {
            break;
        }

        start += step;
    }

    debug!(
        "Chunked {} words into {} chunks (size: {}, overlap: {})",
        words.len(),
        chunks.len(),
        chunk_size,
        overlap
    );

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_text() {
        let chunks = chunk_text("", 10, 2);
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_text_smaller_than_chunk_size() {
        let text = "one two three";
        let chunks = chunk_text(text, 10, 2);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_exact_chunk_size() {
        let text = "one two three four five";
        let chunks = chunk_text(text, 5, 0);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_multiple_chunks_no_overlap() {
        let text = "one two three four five six";
        let chunks = chunk_text(text, 3, 0);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], "one two three");
        assert_eq!(chunks[1], "four five six");
    }

    #[test]
    fn test_multiple_chunks_with_overlap() {
        let text = "one two three four five six seven";
        let chunks = chunk_text(text, 4, 1);
        // With 7 words, chunk_size=4, overlap=1:
        // Chunk 1: words 0-3 (4 words)
        // Step = 4-1 = 3, next start = 3
        // Chunk 2: words 3-6 (4 words, ends at word 7)
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], "one two three four");
        assert_eq!(chunks[1], "four five six seven"); // Overlaps "four"
    }

    #[test]
    fn test_overlap_correctness() {
        let text = "a b c d e f g h i j";
        let chunks = chunk_text(text, 5, 2);

        // First chunk: words 0-4 (a b c d e)
        assert_eq!(chunks[0], "a b c d e");
        // Second chunk: words 3-7 (d e f g h) - overlaps "d e"
        assert_eq!(chunks[1], "d e f g h");
        // Third chunk: words 6-9 (g h i j) - overlaps "g h"
        assert_eq!(chunks[2], "g h i j");
    }

    #[test]
    fn test_large_overlap_clamped() {
        let text = "one two three four five";
        // overlap >= chunk_size should be handled gracefully
        let chunks = chunk_text(text, 3, 5);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_single_word() {
        let chunks = chunk_text("word", 10, 2);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "word");
    }

    #[test]
    fn test_whitespace_handling() {
        let text = "one  two   three    four"; // Multiple spaces
        let chunks = chunk_text(text, 2, 0);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], "one two");
        assert_eq!(chunks[1], "three four");
    }

    #[test]
    fn test_realistic_journal_entry() {
        let text = "Today was a good day. I woke up early and went for a run. \
                    The weather was perfect. Later, I worked on my project and \
                    made significant progress. In the evening, I relaxed with \
                    a book. Overall, very productive and satisfying.";

        let chunks = chunk_text(text, 20, 5);

        // Should create multiple chunks
        assert!(chunks.len() >= 2);

        // Each chunk should have roughly 20 words (except possibly the last)
        for (i, chunk) in chunks.iter().enumerate() {
            let word_count = chunk.split_whitespace().count();
            if i < chunks.len() - 1 {
                // Not the last chunk
                assert!(
                    (15..=25).contains(&word_count),
                    "Chunk {} has {} words",
                    i,
                    word_count
                );
            }
        }
    }
}
