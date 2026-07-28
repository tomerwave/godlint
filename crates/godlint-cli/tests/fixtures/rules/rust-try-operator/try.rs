fn example() -> Result<u32, Error> {
    let first = one()?;
    let second = two()?;
    let third = three()?;

    Ok(first + second + third)
}
