fn nested(x: u32) -> u32 {
    if x > 0 {
        if x > 1 {
            if x > 2 {
                return 1;
            }
        }
    }
    0
}
