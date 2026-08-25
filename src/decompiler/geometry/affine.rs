#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct Affine2 {
    pub(crate) m00: f32,
    pub(crate) m01: f32,
    pub(crate) m02: f32,
    pub(crate) m10: f32,
    pub(crate) m11: f32,
    pub(crate) m12: f32,
}

impl Default for Affine2 {
    fn default() -> Self {
        Self {
            m00: 1.0,
            m01: 0.0,
            m02: 0.0,
            m10: 0.0,
            m11: 1.0,
            m12: 0.0,
        }
    }
}

pub(super) fn fit_page_to_canvas(
    uvs: &[[f32; 2]],
    positions: &[[f32; 2]],
    page_width: u32,
    page_height: u32,
) -> Affine2 {
    let vertex_count = uvs.len().min(positions.len());
    if vertex_count == 0 {
        return Affine2::default();
    }

    let count = vertex_count as f64;
    let width = f64::from(page_width);
    let height = f64::from(page_height);
    let mut page_mean = [0.0; 2];
    let mut canvas_mean = [0.0; 2];
    for (uv, position) in uvs.iter().zip(positions).take(vertex_count) {
        page_mean[0] += f64::from(uv[0]) * width;
        page_mean[1] += f64::from(uv[1]) * height;
        canvas_mean[0] += f64::from(position[0]);
        canvas_mean[1] += f64::from(position[1]);
    }
    for value in page_mean.iter_mut().chain(canvas_mean.iter_mut()) {
        *value /= count;
    }

    let mut page_xx = 0.0;
    let mut page_xy = 0.0;
    let mut page_yy = 0.0;
    let mut canvas_page_x = [0.0; 2];
    let mut canvas_page_y = [0.0; 2];
    for (uv, position) in uvs.iter().zip(positions).take(vertex_count) {
        let page_x = f64::from(uv[0]) * width - page_mean[0];
        let page_y = f64::from(uv[1]) * height - page_mean[1];
        page_xx += page_x * page_x;
        page_xy += page_x * page_y;
        page_yy += page_y * page_y;
        for axis in 0..2 {
            let canvas = f64::from(position[axis]) - canvas_mean[axis];
            canvas_page_x[axis] += page_x * canvas;
            canvas_page_y[axis] += page_y * canvas;
        }
    }

    let determinant = page_xx * page_yy - page_xy * page_xy;
    let has_area = determinant > 1.0e-12 * page_xx * page_yy;
    let mut linear = [[0.0; 2]; 2];
    for axis in 0..2 {
        if has_area {
            linear[axis][0] =
                (canvas_page_x[axis] * page_yy - canvas_page_y[axis] * page_xy) / determinant;
            linear[axis][1] =
                (canvas_page_y[axis] * page_xx - canvas_page_x[axis] * page_xy) / determinant;
        } else if page_xx >= page_yy && page_xx > 0.0 {
            linear[axis][0] = canvas_page_x[axis] / page_xx;
            linear[axis][1] = if axis == 1 { 1.0 } else { 0.0 };
        } else if page_yy > 0.0 {
            linear[axis][0] = if axis == 0 { 1.0 } else { 0.0 };
            linear[axis][1] = canvas_page_y[axis] / page_yy;
        } else {
            linear[axis][axis] = 1.0;
        }
    }

    if (linear[0][0] - 1.0).abs() < 1.0e-3
        && linear[0][1].abs() < 1.0e-3
        && linear[1][0].abs() < 1.0e-3
        && (linear[1][1] - 1.0).abs() < 1.0e-3
    {
        linear = [[1.0, 0.0], [0.0, 1.0]];
    }
    for value in linear.iter_mut().flatten() {
        if value.abs() < 1.0e-4 {
            *value = 0.0;
        }
    }

    Affine2 {
        m00: linear[0][0] as f32,
        m01: linear[0][1] as f32,
        m02: (canvas_mean[0] - linear[0][0] * page_mean[0] - linear[0][1] * page_mean[1]) as f32,
        m10: linear[1][0] as f32,
        m11: linear[1][1] as f32,
        m12: (canvas_mean[1] - linear[1][0] * page_mean[0] - linear[1][1] * page_mean[1]) as f32,
    }
}
