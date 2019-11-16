use std::f32;

use ggez::nalgebra as na;

fn circle_flattening_step(radius: f32, mut tolerance: f32) -> f32 {
    tolerance = f32::min(tolerance, radius);
    2.0 * f32::sqrt(2.0 * tolerance * radius - tolerance * tolerance)
}

pub fn stroke_border_radius(
    radius: f32,
    angle: f32,
    size: f32,
    tolerance: f32,
) -> Vec<na::Point2<f32>> {
    let step = circle_flattening_step(radius, tolerance);
    let arc_len = size * radius;
    let num_points = (arc_len / step).ceil() as u32 - 1;

    let mut points = Vec::with_capacity(num_points as usize);
    let angle_size = size;
    let starting_angle = angle;

    for i in 1..=num_points {
        let new_angle = i as f32 * (angle_size) / (num_points + 1) as f32 + starting_angle;
        let v = na::Vector2::new(new_angle.cos(), new_angle.sin()) * radius;
        let p = na::Point2::from(v);
        points.push(p);
    }

    points
}

pub fn arc(
    radius: f32,
    start_angle: f32,
    radian_size: f32,
    width: f32,
    width_inner: bool,
    tolerance: f32,
) -> Vec<na::Point2<f32>> {
    let mut points: Vec<na::Point2<f32>> =
        Vec::with_capacity(circle_flattening_step(radius, tolerance) as usize * 2);

    let (radius1, radius2) = if width_inner {
        (radius - width, radius)
    } else {
        (radius, radius + width)
    };

    points.push(na::Point2::new(
        start_angle.cos() * radius1,
        start_angle.sin() * radius1,
    ));
    points.push(na::Point2::new(
        start_angle.cos() * radius2,
        start_angle.sin() * radius2,
    ));

    let mut side1 = stroke_border_radius(radius2, start_angle, radian_size, tolerance);

    points.append(&mut side1);

    points.push(na::Point2::new(
        (start_angle + radian_size).cos() * radius2,
        (start_angle + radian_size).sin() * radius2,
    ));
    points.push(na::Point2::new(
        (start_angle + radian_size).cos() * radius1,
        (start_angle + radian_size).sin() * radius1,
    ));

    let mut side2 = stroke_border_radius(radius1, start_angle, radian_size, tolerance);
    side2.reverse();

    points.append(&mut side2);

    points
}

pub fn player(radius: f32, angle: f32, size: f32, width: f32) -> Vec<na::Point2<f32>> {
    let x1 = angle.cos() * radius;
    let y1 = angle.sin() * radius;

    let points = vec![
        na::Point2::new(
            angle.cos() * (radius - width),
            angle.sin() * (radius - width),
        ),
        na::Point2::new(
            x1 + size * (angle - f32::consts::FRAC_PI_2).cos(),
            y1 + size * (angle - f32::consts::FRAC_PI_2).sin(),
        ),
        na::Point2::new(
            angle.cos() * (radius - width / 3.5),
            angle.sin() * (radius - width / 3.5),
        ),
        na::Point2::new(
            x1 + size * (angle + f32::consts::FRAC_PI_2).cos(),
            y1 + size * (angle + f32::consts::FRAC_PI_2).sin(),
        ),
    ];
    points
}
