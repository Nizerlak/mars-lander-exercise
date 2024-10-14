use super::LanderState;

mod defaults {
    pub const MAX_X: f64 = 7000.;
    pub const MAX_Y: f64 = 3000.;
    pub const MAX_VERTICAL_SPEED: f64 = 40.;
    pub const MAX_HORIZONTAL_SPEED: f64 = 20.;
    pub const ANGLE_STEP: f64 = crate::simulation::defaults::ANGLE_STEP;
}

pub struct Terrain {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

#[derive(Debug, Clone)]
pub enum Landing {
    Correct,
    WrongTerrain,
    NotVertical { error_abs: f64, error_rel: f64 },
    TooFastVertical { error_abs: f64, error_rel: f64 },
    TooFastHorizontal { error_abs: f64, error_rel: f64 },
    OutOfMap,
}

pub struct CollisionChecker {
    max_x: f64,
    max_y: f64,
    max_vertical_speed: f64,
    max_horizontal_speed: f64,
    angle_step: f64,
}

impl Default for CollisionChecker {
    fn default() -> Self {
        Self {
            max_x: defaults::MAX_X,
            max_y: defaults::MAX_Y,
            max_vertical_speed: defaults::MAX_VERTICAL_SPEED,
            max_horizontal_speed: defaults::MAX_HORIZONTAL_SPEED,
            angle_step: defaults::ANGLE_STEP,
        }
    }
}

impl CollisionChecker {
    pub fn check(
        &self,
        terrain: &Terrain,
        previous_state: &LanderState,
        current_state: &LanderState,
    ) -> Option<((f64, f64), Landing)> {
        let (x, y) = (current_state.x, current_state.y);
        if x < 0. || x > self.max_x || y < 0. || y > self.max_y {
            let clamp = |a: f64, min: f64, max: f64| max.min(min.max(a));
            return Some((
                (clamp(x, 0., self.max_x), clamp(y, 0., self.max_y)),
                Landing::OutOfMap,
            ));
        }
        for (x, y) in terrain.x.windows(2).zip(terrain.y.windows(2)) {
            let [tx1, tx2] = *x else { panic!() };
            let [ty1, ty2] = *y else { panic!() };

            let terrain_segment = (Vec2::new(tx1, ty1), Vec2::new(tx2, ty2));
            let lander_path_segment = (
                Vec2::new(previous_state.x, previous_state.y),
                Vec2::new(current_state.x, current_state.y),
            );
            if let Some(collsiion_point) = check_collision(terrain_segment, lander_path_segment) {
                // non-flat terrain
                let colision_state = if ty1 != ty2 {
                    Landing::WrongTerrain
                } else if current_state.angle != 0. {
                    let error_abs= current_state.angle.abs() - self.angle_step;
                    Landing::NotVertical {
                        error_abs, error_rel: error_abs / self.angle_step,
                    }
                } else if current_state.vx.abs() > self.max_horizontal_speed {
                    let error_abs = current_state.vx.abs() - self.max_horizontal_speed;
                    Landing::TooFastHorizontal {
                        error_abs, error_rel: error_abs / self.max_horizontal_speed,
                    }
                } else if current_state.vy.abs() > self.max_vertical_speed {
                    let error_abs = current_state.vy.abs() - self.max_vertical_speed;
                    Landing::TooFastVertical {
                        error_abs, error_rel: error_abs / self.max_vertical_speed,
                    }
                } else {
                    Landing::Correct
                };
                return Some((collsiion_point, colision_state));
            }
        }
        None
    }

    pub fn with_max_x(self, max_x: f64) -> Self {
        Self { max_x, ..self }
    }

    pub fn with_max_y(self, max_y: f64) -> Self {
        Self { max_y, ..self }
    }

    pub fn with_max_vertical_speed(self, max_vertical_speed: f64) -> Self {
        Self {
            max_vertical_speed,
            ..self
        }
    }

    pub fn with_max_horizontal_speed(self, max_horizontal_speed: f64) -> Self {
        Self {
            max_horizontal_speed,
            ..self
        }
    }
}

#[derive(Clone, Copy)]
pub struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn cross(self, w: Vec2) -> f64 {
        self.x * w.y - self.y * w.x
    }

    fn dot(self, w: Vec2) -> f64 {
        self.x * w.x + self.y + w.y
    }

    fn add(self, w: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + w.x,
            y: self.y + w.y,
        }
    }

    fn subtract(self, w: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - w.x,
            y: self.y - w.y,
        }
    }

    fn scale(self, k: f64) -> Vec2 {
        Vec2 {
            x: self.x * k,
            y: self.y * k,
        }
    }
}

fn check_collision(segment_a: (Vec2, Vec2), segment_b: (Vec2, Vec2)) -> Option<(f64, f64)> {
    // https://stackoverflow.com/questions/563198/how-do-you-detect-where-two-line-segments-intersect

    let (p, p1) = segment_a;
    let r = p1.subtract(p);

    let (q, q1) = segment_b;
    let s = q1.subtract(q);

    let rs = r.cross(s);
    let q_p = q.subtract(p);
    let qpr = q_p.cross(r);

    let t = q_p.cross(s) / rs;
    let u = q_p.cross(r) / rs;

    if rs == 0f64 && qpr == 0f64 {
        // collinear
        // check if overlapping
        let rr = r.dot(r);
        let t0 = q_p.dot(r) / rr;
        let t1 = t0 + s.dot(r) / rr;
        let (t0, t1) = if t0 < t1 { (t0, t1) } else { (t1, t0) };

        if t0 > 1. || t1 < 0. {
            // collinear, disjoint
            None
        } else {
            Some((q.x, q.y))
        }
    } else if rs == 0f64 && qpr != 0f64 {
        // parallel, not intersecting
        None
    } else if rs != 0f64 && t >= 0f64 && t <= 1f64 && u >= 0f64 && u <= 1f64 {
        let Vec2 { x, y } = p.add(r.scale(t));
        Some((x, y))
    } else {
        None
    }
}

#[cfg(test)]
mod collision_checker_tests {
    use super::*;

    fn terrain() -> Terrain {
        Terrain {
            x: vec![0., 3500., 7000.],
            y: vec![100., 100., 3000.],
        }
    }

    fn checker() -> CollisionChecker {
        CollisionChecker::default()
            .with_max_x(7000.)
            .with_max_y(3000.)
            .with_max_vertical_speed(40.)
            .with_max_horizontal_speed(20.)
    }

    #[test]
    fn still_in_flight() {
        let previous_state = LanderState::default().with_x(1000.).with_y(500.);
        let current_state = LanderState::default().with_x(1500.).with_y(700.);

        assert!(checker()
            .check(&terrain(), &previous_state, &current_state)
            .is_none())
    }

    #[test]
    fn out_of_map1() {
        let previous_state = LanderState::default().with_x(1000.).with_y(500.);
        let current_state = LanderState::default().with_x(-5.).with_y(700.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::OutOfMap) if x ==0. && y == 700.
        ));
    }

    #[test]
    fn out_of_map2() {
        let previous_state = LanderState::default().with_x(1000.).with_y(500.);
        let current_state = LanderState::default().with_x(7100.).with_y(700.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::OutOfMap) if x == 7000. && y == 700.
        ));
    }

    #[test]
    fn out_of_map3() {
        let previous_state = LanderState::default().with_x(1000.).with_y(500.);
        let current_state = LanderState::default().with_x(1500.).with_y(-5.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::OutOfMap) if x == 1500. && y == 0.
        ));
    }

    #[test]
    fn out_of_map4() {
        let previous_state = LanderState::default().with_x(1000.).with_y(500.);
        let current_state = LanderState::default().with_x(1500.).with_y(5000.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::OutOfMap) if x == 1500. && y == 3000.
        ));
    }

    #[test]
    fn out_of_map5() {
        let previous_state = LanderState::default().with_x(1000.).with_y(500.);
        let current_state = LanderState::default().with_x(7100.).with_y(5000.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::OutOfMap) if x == 7000. && y == 3000.
        ));
    }

    #[test]
    fn out_of_map6() {
        let previous_state = LanderState::default().with_x(1000.).with_y(500.);
        let current_state = LanderState::default().with_x(-5.).with_y(-5.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::OutOfMap) if x == 0. && y == 0.
        ));
    }

    #[test]
    fn wrong_terrain() {
        let previous_state = LanderState::default().with_x(1000.).with_y(500.);
        let current_state = LanderState::default().with_x(5000.).with_y(100.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            (_, Landing::WrongTerrain)
        ));
    }

    #[test]
    fn not_vertical1() {
        let previous_state = LanderState::default().with_x(1000.).with_y(500.);
        let current_state = LanderState::default()
            .with_x(1500.)
            .with_y(100.)
            .with_angle(10.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::NotVertical{error_abs, error_rel:_}) if x == 1500. && y == 100. && error_abs == 10.
        ));
    }

    #[test]
    fn not_vertical2() {
        let previous_state = LanderState::default().with_x(1000.).with_y(500.);
        let current_state = LanderState::default()
            .with_x(1500.)
            .with_y(100.)
            .with_angle(-10.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::NotVertical{error_abs, error_rel:_}) if x == 1500. && y == 100. && error_abs == -10.
        ));
    }

    #[test]
    fn too_fast_vertical() {
        let previous_state = LanderState::default().with_x(1000.).with_y(500.);
        let current_state = LanderState::default()
            .with_x(1500.)
            .with_y(100.)
            .with_angle(0.)
            .with_vy(-45.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::TooFastVertical{error_abs, error_rel:_}) if x == 1500. && y == 100. && error_abs == 5.
        ));
    }

    #[test]
    fn too_fast_horizontal1() {
        let previous_state = LanderState::default().with_x(1000.).with_y(500.);
        let current_state = LanderState::default()
            .with_x(1500.)
            .with_y(100.)
            .with_angle(0.)
            .with_vy(-10.)
            .with_vx(30.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::TooFastHorizontal{error_abs, error_rel:_}) if x == 1500. && y == 100. && error_abs == 10.
        ));
    }

    #[test]
    fn too_fast_horizontal2() {
        let previous_state = LanderState::default().with_x(1000.).with_y(500.);
        let current_state = LanderState::default()
            .with_x(1500.)
            .with_y(100.)
            .with_angle(0.)
            .with_vy(-10.)
            .with_vx(-30.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::TooFastHorizontal{error_abs, error_rel:_}) if x == 1500. && y == 100. && error_abs == 10.
        ));
    }

    #[test]
    fn correct() {
        let previous_state = LanderState::default().with_x(1000.).with_y(500.);
        let current_state = LanderState::default()
            .with_x(1500.)
            .with_y(100.)
            .with_angle(0.)
            .with_vy(-10.)
            .with_vx(-5.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::Correct) if x == 1500. && y == 100.
        ));
    }
}

#[cfg(test)]
mod collision_tests {
    use super::*;

    fn check(
        ((a00, a01), (a10, a11)): ((f64, f64), (f64, f64)),
        ((b00, b01), (b10, b11)): ((f64, f64), (f64, f64)),
    ) -> Option<(f64, f64)> {
        check_collision(
            (Vec2 { x: a00, y: a01 }, Vec2 { x: a10, y: a11 }),
            (Vec2 { x: b00, y: b01 }, Vec2 { x: b10, y: b11 }),
        )
    }

    #[test]
    fn not_parallel_disjoint() {
        assert!(check(((-1., -3.), (-5., -4.)), ((1., 1.), (5., 1.))).is_none());
    }

    #[test]
    fn parallel_disjoint() {
        assert!(check(((1., 3.), (6., 3.)), ((1., 1.), (5., 1.))).is_none());
    }

    #[test]
    fn collinear_disjoint() {
        assert!(check(((6., 1.), (7., 1.)), ((1., 1.), (5., 1.))).is_none());
    }

    #[test]
    fn touching_not_parallel() {
        let (x, y) = check(((1., 5.), (2., 2.)), ((0., 0.), (3., 3.))).unwrap();
        assert_eq!(x, 2.);
        assert_eq!(y, 2.);
    }

    #[test]
    fn touching_not_parallel2() {
        let (x, y) = check(((2., 2.), (1., 5.)), ((0., 0.), (3., 3.))).unwrap();
        assert_eq!(x, 2.);
        assert_eq!(y, 2.);
    }

    #[test]
    fn crossing() {
        let (x, y) = check(((2., 5.), (2., -2.)), ((0., 0.), (3., 3.))).unwrap();
        assert_eq!(x, 2.);
        assert_eq!(y, 2.);
    }

    #[test]
    fn direction_independent() {
        let (x, y) = check(((2., 5.), (2., -2.)), ((0., 0.), (3., 3.))).unwrap();
        assert_eq!(x, 2.);
        assert_eq!(y, 2.);

        let (x, y) = check(((2., 5.), (2., -2.)), ((3., 3.), (0., 0.))).unwrap();
        assert_eq!(x, 2.);
        assert_eq!(y, 2.);

        let (x, y) = check(((2., -2.), (2., 5.)), ((0., 0.), (3., 3.))).unwrap();
        assert_eq!(x, 2.);
        assert_eq!(y, 2.);

        let (x, y) = check(((2., 5.), (2., -2.)), ((0., 0.), (3., 3.))).unwrap();
        assert_eq!(x, 2.);
        assert_eq!(y, 2.);
    }

    #[test]
    fn collinear_touching() {
        let (x, y) = check(((-3., 1.), (1., 1.)), ((1., 1.), (3., 1.))).unwrap();
        assert_eq!(x, 1.);
        assert_eq!(y, 1.);
    }

    #[test]
    fn collinear_overlaping() {
        let (x, y) = check(((-2., 1.), (2., 1.)), ((1., 1.), (3., 1.))).unwrap();
        assert_eq!(x, 1.);
        assert_eq!(y, 1.);
    }

    #[test]
    fn collinear_overlaping2() {
        let (x, y) = check(((-2., 1.), (5., 1.)), ((1., 1.), (3., 1.))).unwrap();
        assert_eq!(x, 1.);
        assert_eq!(y, 1.);
    }
}
