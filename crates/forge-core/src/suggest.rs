/// Find the closest matching artifact name using edit distance.
/// This powers the "did you mean?" hint when a user mistypes an artifact.
///
/// Levenshtein distance measures how many single-character edits
/// (insertions, deletions, substitutions) separate two strings.
/// "servce" → "service" = 1 edit (insert 'i') → very likely a typo.
pub fn closest_match<'a>(input: &str, candidates: &[&'a str]) -> Option<&'a str> {
    let input_lower = input.to_lowercase();

    candidates
        .iter()
        .map(|candidate| {
            let dist = edit_distance::edit_distance(&input_lower, candidate);
            (candidate, dist)
        })
        .filter(|(_, dist)| *dist <= 3)
        .min_by_key(|(_, dist)| *dist)
        .map(|(candidate, _)| *candidate)
}
