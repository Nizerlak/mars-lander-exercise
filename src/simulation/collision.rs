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

struct MapIterator<'a> {
    previous_point: Vec2,
    map_iter: Box<dyn Iterator<Item = Vec2> + 'a>,
}

impl<'a> MapIterator<'a> {
    fn new(max_x: f64, max_y: f64, terrain: &'a Terrain) -> Self {
        let map_iter = terrain
            .x
            .iter()
            .zip(terrain.y.iter())
            .map(|(&x, &y)| Vec2::new(x, y))
            .chain(std::iter::once(Vec2::new(max_x, max_y)))
            .chain(std::iter::once(Vec2::new(0., max_y)));
        Self {
            previous_point: Vec2::new(0., max_y),
            map_iter: Box::new(map_iter),
        }
    }
}

impl Iterator for MapIterator<'_> {
    type Item = (Vec2, Vec2);

    fn next(&mut self) -> Option<Self::Item> {
        let next_point = self.map_iter.next()?;
        let segment = (self.previous_point, next_point);
        self.previous_point = next_point;
        Some(segment)
    }
}

impl CollisionChecker {
    fn iter_map<'a>(&self, terrain: &'a Terrain) -> MapIterator<'a> {
        MapIterator::new(self.max_x, self.max_y, terrain)
    }

    pub fn check(
        &self,
        terrain: &Terrain,
        previous_state: &LanderState,
        current_state: &LanderState,
    ) -> Option<((f64, f64), Landing)> {
        for terrain_segment in self.iter_map(terrain) {
            let lander_path_segment = (
                Vec2::new(previous_state.x, previous_state.y),
                Vec2::new(current_state.x, current_state.y),
            );
            if let Some(collsiion_point) = check_collision(terrain_segment, lander_path_segment) {
                // non-flat terrain
                let colision_state = if terrain_segment.0.y != terrain_segment.1.y {
                    Landing::WrongTerrain
                } else if terrain_segment.0.y >= self.max_y {
                    //ceiling
                    Landing::WrongTerrain
                } else if current_state.angle != 0. {
                    let error_abs = current_state.angle.abs();
                    Landing::NotVertical {
                        error_abs,
                        error_rel: (error_abs - self.angle_step).max(0f64) / self.angle_step,
                    }
                } else if current_state.vx.abs() > self.max_horizontal_speed {
                    let error_abs = current_state.vx.abs() - self.max_horizontal_speed;
                    Landing::TooFastHorizontal {
                        error_abs,
                        error_rel: error_abs / self.max_horizontal_speed,
                    }
                } else if current_state.vy.abs() > self.max_vertical_speed {
                    let error_abs = current_state.vy.abs() - self.max_vertical_speed;
                    Landing::TooFastVertical {
                        error_abs,
                        error_rel: error_abs / self.max_vertical_speed,
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
        assert!(max_vertical_speed > 0.);
        Self {
            max_vertical_speed,
            ..self
        }
    }

    pub fn with_max_horizontal_speed(self, max_horizontal_speed: f64) -> Self {
        assert!(max_horizontal_speed > 0.);
        Self {
            max_horizontal_speed,
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
            y: vec![100., 100., 150.],
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
        let previous_state = LanderState::default().with_x(1.).with_y(700.);
        let current_state = LanderState::default().with_x(-5.).with_y(700.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::WrongTerrain) if x ==0. && y == 700.
        ));
    }

    #[test]
    fn out_of_map2() {
        let previous_state = LanderState::default().with_x(6900.).with_y(700.);
        let current_state = LanderState::default().with_x(7100.).with_y(700.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::WrongTerrain) if x == 7000. && y == 700.
        ));
    }

    #[test]
    fn out_of_map3() {
        let previous_state = LanderState::default().with_x(1000.).with_y(2900.);
        let current_state = LanderState::default().with_x(1000.).with_y(3100.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::WrongTerrain) if x == 1000. && y == 3000.
        ));
    }

    #[test]
    fn out_of_map4() {
        let previous_state = LanderState::default().with_x(6900.).with_y(2900.);
        let current_state = LanderState::default().with_x(7100.).with_y(3100.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::WrongTerrain) if x == 7000. && y == 3000.
        ));
    }

    #[test]
    fn out_of_map5() {
        let previous_state = LanderState::default().with_x(100.).with_y(2900.);
        let current_state = LanderState::default().with_x(-100.).with_y(3100.);

        assert!(matches!(
            checker()
                .check(&terrain(), &previous_state, &current_state)
                .unwrap(),
            ((x, y), Landing::WrongTerrain) if x == 0. && y == 3000.
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
            ((x, y), Landing::NotVertical{error_abs, error_rel:_}) if x == 1500. && y == 100. && error_abs == 10.
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

#[cfg(test)]
mod map_iterator_tests {
    use super::*;

    fn terrain() -> Terrain {
        Terrain {
            x: vec![0., 3500.],
            y: vec![100., 100.],
        }
    }

    #[test]
    fn map_iterator_basic() {
        let terrain = terrain();
        let mut map_iter = MapIterator::new(7000., 3000., &terrain);

        assert_eq!(
            map_iter.next().unwrap(),
            (Vec2::new(0., 3000.), Vec2::new(0., 100.))
        );
        assert_eq!(
            map_iter.next(),
            Some((Vec2::new(0., 100.), Vec2::new(3500., 100.)))
        );
        assert_eq!(
            map_iter.next(),
            Some((Vec2::new(3500., 100.), Vec2::new(7000., 3000.)))
        );
        assert_eq!(
            map_iter.next(),
            Some((Vec2::new(7000., 3000.), Vec2::new(0., 3000.)))
        );
        assert_eq!(map_iter.next(), None);
    }

    #[test]
    fn map_iterator_empty_terrain() {
        let terrain = Terrain {
            x: vec![],
            y: vec![],
        };
        let mut map_iter = MapIterator::new(7000., 3000., &terrain);

        assert_eq!(
            map_iter.next(),
            Some((Vec2::new(0., 3000.), Vec2::new(7000., 3000.)))
        );
        assert_eq!(
            map_iter.next(),
            Some((Vec2::new(7000., 3000.), Vec2::new(0., 3000.)))
        );
        assert_eq!(map_iter.next(), None);
    }

    #[test]
    fn map_iterator_single_point_terrain() {
        let terrain = Terrain {
            x: vec![3500.],
            y: vec![100.],
        };
        let mut map_iter = MapIterator::new(7000., 3000., &terrain);

        assert_eq!(
            map_iter.next(),
            Some((Vec2::new(0., 3000.), Vec2::new(3500., 100.)))
        );
        assert_eq!(
            map_iter.next(),
            Some((Vec2::new(3500., 100.), Vec2::new(7000., 3000.)))
        );
        assert_eq!(
            map_iter.next(),
            Some((Vec2::new(7000., 3000.), Vec2::new(0., 3000.)))
        );
        assert_eq!(map_iter.next(), None);
    }
}
