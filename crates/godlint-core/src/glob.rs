pub fn matches(pattern: &str, path: &str) -> bool {
    let pattern_segments = split(pattern);
    let path_segments = split(path);

    if pattern_segments.len() == 1 && !pattern.contains('/') {
        return path_segments
            .iter()
            .any(|segment| segment_matches(pattern_segments[0], segment));
    }

    match_segments(&pattern_segments, &path_segments)
}

fn split(value: &str) -> Vec<&str> {
    value.split('/').filter(|part| !part.is_empty()).collect()
}

fn match_segments(pattern: &[&str], path: &[&str]) -> bool {
    let Some((head, rest)) = pattern.split_first() else {
        return path.is_empty();
    };

    if *head == "**" {
        return (0..=path.len()).any(|skipped| match_segments(rest, &path[skipped..]));
    }

    let Some((candidate, remaining)) = path.split_first() else {
        return false;
    };

    segment_matches(head, candidate) && match_segments(rest, remaining)
}

fn segment_matches(pattern: &str, segment: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let segment: Vec<char> = segment.chars().collect();
    let mut table = vec![false; segment.len() + 1];

    table[0] = true;

    for &expected in &pattern {
        advance(&mut table, &segment, expected);
    }

    table[segment.len()]
}

fn advance(table: &mut [bool], segment: &[char], expected: char) {
    if expected == '*' {
        let mut matched = false;

        for entry in table.iter_mut() {
            matched |= *entry;
            *entry = matched;
        }

        return;
    }

    for index in (0..segment.len()).rev() {
        table[index + 1] = table[index] && (expected == '?' || expected == segment[index]);
    }

    table[0] = false;
}
