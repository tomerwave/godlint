fn flat(x: u32) -> u32 {
    if x < 0 { return 1; }
    if x == 0 { return 2; }
    if x < 10 { return 3; }
    4
}
