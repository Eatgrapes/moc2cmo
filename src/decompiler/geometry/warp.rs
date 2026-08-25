pub(super) fn apply(
    control_points: &[[f32; 2]],
    columns: usize,
    rows: usize,
    point: [f32; 2],
) -> [f32; 2] {
    let grid_x = point[0] * columns as f32;
    let grid_y = point[1] * rows as f32;
    let column = cell_index(grid_x, columns);
    let row = cell_index(grid_y, rows);
    let u = grid_x - column as f32;
    let v = grid_y - row as f32;
    let row_width = columns + 1;
    let top_left = row * row_width + column;
    let p00 = control_points[top_left];
    let p10 = control_points[top_left + 1];
    let p01 = control_points[top_left + row_width];
    let p11 = control_points[top_left + row_width + 1];

    if u + v <= 1.0 {
        combine(p00, p10, p01, 1.0 - u - v, u, v)
    } else {
        combine(p11, p01, p10, u + v - 1.0, 1.0 - u, 1.0 - v)
    }
}

fn cell_index(value: f32, cell_count: usize) -> usize {
    if value <= 0.0 {
        0
    } else if value >= cell_count as f32 {
        cell_count - 1
    } else {
        value as usize
    }
}

fn combine(
    first: [f32; 2],
    second: [f32; 2],
    third: [f32; 2],
    first_weight: f32,
    second_weight: f32,
    third_weight: f32,
) -> [f32; 2] {
    [
        first[0] * first_weight + second[0] * second_weight + third[0] * third_weight,
        first[1] * first_weight + second[1] * second_weight + third[1] * third_weight,
    ]
}
