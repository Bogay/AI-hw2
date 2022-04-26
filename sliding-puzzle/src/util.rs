use sliding_puzzle_core::Vec2;

pub fn vec2_from_str(input: &str) -> Result<Vec2, String> {
    let input = input.split(',').collect::<Vec<_>>();

    if input.len() != 2 {
        return Err("Input shoud be 2 comma-delimited number. e.g. 4,2".to_string());
    }

    let x = input[0]
        .parse::<i8>()
        .map_err(|e| format!("Cannot parse x: {}", e))?;
    let y = input[1]
        .parse::<i8>()
        .map_err(|e| format!("Cannot parse y: {}", e))?;
    Ok(Vec2::new(x, y))
}
