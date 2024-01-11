mod simulation {

    const G: f64 = 3.711;
    const ANGLE_STEP: f64 = 15.;
    const POWER_STEP: i32 = 1;
    const DT: f64 = 1.;
    const MAP_WIDTH: u32 = 7000;
    const MAP_HEIGHT: u32 = 3000;

    #[derive(Clone, Debug)]
    pub struct Thrust {
        angle: f64,
        power: i32,
    }

    impl Default for Thrust {
        fn default() -> Self {
            Self::zero()
        }
    }

    impl Thrust {
        fn into_vector(self) -> (f64, f64) {
            let angle = (self.angle + 90.).to_radians();
            let (sin, cos) = angle.sin_cos();
            let power = self.power as f64;
            (cos * power, sin * power)
        }

        fn zero() -> Self {
            Self {
                angle: 0.,
                power: 0,
            }
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct LanderState {
        pub x: f64,
        pub y: f64,
        pub vx: f64,
        pub vy: f64,
        pub fuel: i32,
        pub thrust: Thrust,
    }

    impl LanderState {
        pub fn with_x(self, x: f64) -> Self {
            Self { x, ..self }
        }

        pub fn with_y(self, y: f64) -> Self {
            Self { y, ..self }
        }

        pub fn with_vx(self, vx: f64) -> Self {
            Self { vx, ..self }
        }

        pub fn with_vy(self, vy: f64) -> Self {
            Self { vy, ..self }
        }

        pub fn with_fuel(self, fuel: i32) -> Self {
            Self { fuel, ..self }
        }

        pub fn with_thrust(self, thrust: Thrust) -> Self {
            Self { thrust, ..self }
        }
    }

    pub struct LandPoint {
        x: f64,
        y: f64,
    }

    #[derive(Debug)]
    struct Land {
        x: Vec<f64>,
        y: Vec<f64>,
    }

    #[derive(Debug)]
    pub struct Simulation {
        land: Land,
        lander: LanderState,
    }

    #[derive(Debug)]
    pub enum SimulationError {
        InvalidThrust(Thrust),
    }

    impl Simulation {
        pub fn new(
            land_points: impl Iterator<Item = LandPoint>,
            initial_lander_state: LanderState,
        ) -> Self {
            let (x, y) = land_points.fold(
                (Vec::new(), Vec::new()),
                |(mut out_x, mut out_y), LandPoint { x, y }| {
                    out_x.push(x);
                    out_y.push(y);
                    (out_x, out_y)
                },
            );

            Self {
                land: Land { x, y },
                lander: initial_lander_state,
            }
        }

        pub fn current_state(&self) -> LanderState {
            self.lander.clone()
        }

        pub fn iterate(&mut self, cmd: Thrust) -> Result<(), SimulationError> {
            // validate cmd
            if !validate_thrust(&cmd) {
                return Err(SimulationError::InvalidThrust(cmd));
            }

            let lander = &mut self.lander;

            // update thrust
            if lander.fuel > cmd.power {
                let Thrust {
                    power: current_power,
                    angle: current_angle,
                } = lander.thrust;
                let (e_power, e_angle) = (
                    clamp(cmd.power - current_power, -POWER_STEP, POWER_STEP),
                    clamp(cmd.angle - current_angle, -ANGLE_STEP, ANGLE_STEP),
                );
                lander.thrust.power += e_power;
                lander.thrust.angle += e_angle;
            } else {
                lander.thrust = Thrust::zero();
            }

            // vectorize thrust
            let (t_x, t_y) = lander.thrust.clone().into_vector();

            // update position
            lander.x += lander.vx * DT + t_x / 2. * DT.powf(2.);
            lander.y += lander.vy * DT + (t_y - G) / 2. * DT.powf(2.);

            // update velocity
            lander.vx += t_x * DT;
            lander.vy += (t_y - G) * DT;

            // consume fuel
            lander.fuel -= lander.thrust.power;
            if lander.fuel < 0 {
                lander.fuel = 0;
            }
            Ok(())
        }
    }

    fn validate_thrust(thrust: &Thrust) -> bool {
        thrust.angle.abs() <= 90. && thrust.power <= 4
    }

    fn clamp<T: PartialOrd>(value: T, lower: T, upper: T) -> T {
        assert!(lower < upper);
        if value < lower {
            lower
        } else if value > upper {
            upper
        } else {
            value
        }
    }

    #[derive(Clone, Copy)]
    struct Vec2 {
        x: f64,
        y: f64,
    }

    impl Vec2 {
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

    impl Into<Vec2> for LandPoint {
        fn into(self) -> Vec2 {
            Vec2 {
                x: self.x,
                y: self.y,
            }
        }
    }

    fn check_collision(
        segment_a: (LandPoint, LandPoint),
        segment_b: (LandPoint, LandPoint),
    ) -> Option<(f64, f64)> {
        // https://stackoverflow.com/questions/563198/how-do-you-detect-where-two-line-segments-intersect

        let (p, p1): (Vec2, Vec2) = (segment_a.0.into(), segment_a.1.into());
        let r = p1.subtract(p);

        let (q, q1): (Vec2, Vec2) = (segment_b.0.into(), segment_b.1.into());
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
    mod physics_tests {
        // Note this useful idiom: importing names from outer (for mod tests) scope.
        use super::*;

        fn assert_feq(left: f64, right: f64) {
            if (left - right).abs() > 1e-9 {
                panic!("Float equal assertion failed, {left} != {right}");
            }
        }

        fn assert_close(left: f64, right: f64, range: f64) {
            if (left - right).abs() > range {
                panic!("Assertion failed {left} not close to {right} within a range {range}");
            }
        }

        fn flat_ground() -> impl Iterator<Item = LandPoint> {
            [0, MAP_WIDTH].into_iter().map(|x| LandPoint {
                x: x as f64,
                y: 0f64,
            })
        }

        fn sim_with_flat_ground(initial_lander: LanderState) -> Simulation {
            Simulation::new(flat_ground(), initial_lander)
        }

        #[test]
        fn clamping() {
            let mut sim = sim_with_flat_ground(
                LanderState::default()
                    .with_y(500.)
                    .with_x(500.)
                    .with_fuel(200),
            );
            sim.iterate(Thrust {
                angle: 16.,
                power: 2,
            })
            .unwrap();
            let state = sim.current_state();
            assert_eq!(state.thrust.angle, 15.);
            assert_eq!(state.thrust.power, 1);
        }

        #[test]
        fn free_fall() {
            let mut sim = sim_with_flat_ground(LanderState::default().with_y(500.));
            let initial_x = sim.current_state().x;
            sim.iterate(Thrust::zero()).unwrap();
            let state = sim.current_state();
            assert_feq(state.x, initial_x);
            assert_close(state.y, 498., 0.15);
        }

        #[test]
        fn thrust_up() {
            let mut sim = sim_with_flat_ground(LanderState::default().with_y(500.).with_fuel(100));
            let initial_x = sim.current_state().x;
            sim.iterate(Thrust {
                angle: 0.,
                power: 4,
            })
            .unwrap();
            let state = sim.current_state();
            assert_feq(state.x, initial_x);
            assert_close(state.y, 498.5, 0.15);
        }

        #[test]
        fn fly_up() {
            let mut sim = sim_with_flat_ground(LanderState::default().with_y(500.).with_fuel(500));
            let initial_x = sim.current_state().x;
            let initial_y = sim.current_state().y;
            for _ in 0..41 {
                sim.iterate(Thrust {
                    angle: 0.,
                    power: 4,
                })
                .unwrap();
            }
            let state = sim.current_state();
            assert!(state.y > initial_y);
            assert_feq(state.x, initial_x);
        }

        #[test]
        fn fuel_consumption() {
            let mut sim = sim_with_flat_ground(LanderState::default().with_y(500.).with_fuel(200));
            let iniitial_fuel = sim.current_state().fuel;
            sim.iterate(Thrust {
                angle: 0.,
                power: 1,
            })
            .unwrap();
            let state = sim.current_state();
            assert_eq!(iniitial_fuel - state.fuel, 1);
        }
    }

    #[cfg(test)]
    mod collision_tests {
        use super::{check_collision, LandPoint};

        fn check(
            ((a00, a01), (a10, a11)): ((f64, f64), (f64, f64)),
            ((b00, b01), (b10, b11)): ((f64, f64), (f64, f64)),
        ) -> Option<(f64, f64)> {
            check_collision(
                (LandPoint { x: a00, y: a01 }, LandPoint { x: a10, y: a11 }),
                (LandPoint { x: b00, y: b01 }, LandPoint { x: b10, y: b11 }),
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
}
