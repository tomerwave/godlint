fn accepted(value: u32) -> u32 {
    if value == 1 {
        10
    } else if value == 2 {
        20
    } else if value == 3 {
        30
    } else {
        0
    }
}

fn reported(value: u32) {
    if value == 1 {
        for item in items {
            work(item);
        }
    }
}
