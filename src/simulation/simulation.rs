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
        x: u32,
        y: u32,
    }

    #[derive(Debug)]
    struct Land {
        x: Vec<u32>,
        y: Vec<u32>,
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

    #[cfg(test)]
    mod tests {
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
            [0, MAP_WIDTH].into_iter().map(|x| LandPoint { x, y: 0 })
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
}
