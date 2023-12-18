mod simulation {

    const G: f64 = 3.711;
    const ANGLE_STEP: f64 = 15.;
    const POWER_STEP: i32 = 1;
    const DT: f64 = 1.;

    #[derive(Clone)]
    pub struct Thrust {
        angle: f64,
        power: i32,
    }

    impl Thrust {
        fn into_vector(self) -> (f64, f64) {
            let angle = self.angle.to_radians();
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

    #[derive(Clone)]
    pub struct LanderState {
        x: f64,
        y: f64,
        vx: f64,
        vy: f64,
        fuel: f64,
        thrust: Thrust,
    }

    pub struct LandPoint {
        x: u32,
        y: u32,
    }

    struct Land {
        x: Vec<u32>,
        y: Vec<u32>,
    }

    pub struct Simulation {
        land: Land,
        lander: LanderState,
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

        pub fn iterate(&mut self, cmd: Thrust) -> Result<(), Thrust> {
            // validate cmd
            if !validate_thrust(&cmd) {
                return Err(cmd);
            }

            // update position
            let lander = &mut self.lander;
            lander.x += lander.vx * DT;
            lander.y += lander.vy * DT;

            // update velocity
            let (t_x, t_y) = lander.thrust.clone().into_vector();
            lander.vx += t_x * DT;
            lander.vy += (t_y - G) * DT;

            // consume fuel
            lander.fuel -= lander.thrust.power as f64;
            if lander.fuel < 0. {
                lander.fuel = 0.;
            }

            // update thrust
            if lander.fuel > 0. {
                let Thrust {
                    power: current_power,
                    angle: current_angle,
                } = lander.thrust;
                let (e_power, e_angle) = (
                    clamp(current_power - cmd.power, -POWER_STEP, POWER_STEP),
                    clamp(current_angle - cmd.angle, -ANGLE_STEP, ANGLE_STEP),
                );
                lander.thrust.power += e_power;
                lander.thrust.angle += e_angle;
            } else {
                lander.thrust = Thrust::zero();
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
}
