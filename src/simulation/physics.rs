pub(crate) mod defaults {
    pub const G: f64 = 3.711;
    pub const ANGLE_STEP: f64 = 15.;
    pub const POWER_STEP: i32 = 1;
    pub const POWER_MAX: i32 = 4;
    pub const ANGLE_LIMIT: f64 = 90.;
    pub const DT: f64 = 1.;
}

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
    pub fn new(angle: f64, power: i32) -> Self {
        Self { angle, power }
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
    pub angle: f64,
    pub power: i32,
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

    pub fn with_angle(self, angle: f64) -> Self {
        Self { angle, ..self }
    }

    pub fn with_power(self, power: i32) -> Self {
        Self { power, ..self }
    }
}

#[derive(Debug)]
pub enum SimulationError {
    InvalidThrust(Thrust),
}

pub struct Physics {
    g: f64,
    dt: f64,
    power_step: i32,
    angle_step: f64,
    power_max: i32,
    angle_limit: f64,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            g: defaults::G,
            dt: defaults::DT,
            power_step: defaults::POWER_STEP,
            angle_step: defaults::ANGLE_STEP,
            power_max: defaults::POWER_MAX,
            angle_limit: defaults::ANGLE_LIMIT,
        }
    }
}

impl Physics {
    pub fn with_g(self, g: f64) -> Self {
        Self { g, ..self }
    }
    pub fn with_dt(self, dt: f64) -> Self {
        Self { dt, ..self }
    }
    pub fn with_power_step(self, power_step: i32) -> Self {
        Self { power_step, ..self }
    }
    pub fn with_angle_step(self, angle_step: f64) -> Self {
        Self { angle_step, ..self }
    }
    pub fn with_power_max(self, power_max: i32) -> Self {
        Self { power_max, ..self }
    }
    pub fn with_angle_limit(self, angle_limit: f64) -> Self {
        Self {
            angle_limit,
            ..self
        }
    }

    pub fn iterate(
        &self,
        mut lander: LanderState,
        cmd: Thrust,
    ) -> Result<LanderState, SimulationError> {
        // validate cmd
        if !self.validate_thrust(&cmd) {
            return Err(SimulationError::InvalidThrust(cmd));
        }

        // update thrust
        if lander.fuel > cmd.power {
            let (current_power, current_angle) = (lander.power, lander.angle);
            let (e_power, e_angle) = (
                clamp(cmd.power - current_power, -self.power_step, self.power_step),
                clamp(cmd.angle - current_angle, -self.angle_step, self.angle_step),
            );
            lander.power += e_power;
            lander.angle += e_angle;
        } else {
            lander.power = 0;
        }

        // vectorize thrust
        let (t_x, t_y) = Thrust {
            angle: lander.angle,
            power: lander.power,
        }
        .into_vector();

        // update position
        lander.x += lander.vx * self.dt + t_x / 2. * self.dt.powf(2.);
        lander.y += lander.vy * self.dt + (t_y - self.g) / 2. * self.dt.powf(2.);

        // update velocity
        lander.vx += t_x * self.dt;
        lander.vy += (t_y - self.g) * self.dt;

        // consume fuel
        lander.fuel -= lander.power;
        if lander.fuel < 0 {
            lander.fuel = 0;
        }
        Ok(lander)
    }

    fn validate_thrust(&self, thrust: &Thrust) -> bool {
        thrust.angle.abs() <= self.angle_limit
            && thrust.power <= self.power_max
            && thrust.power >= 0
    }
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

    #[test]
    fn clamping() {
        let state = Physics::default()
            .iterate(
                LanderState::default()
                    .with_y(500.)
                    .with_x(500.)
                    .with_fuel(200),
                Thrust {
                    angle: 16.,
                    power: 2,
                },
            )
            .unwrap();
        assert_eq!(state.angle, 15.);
        assert_eq!(state.power, 1);
    }

    #[test]
    fn free_fall() {
        let lander = LanderState::default().with_y(500.);
        let initial_x = lander.x;
        let lander = Physics::default().iterate(lander, Thrust::zero()).unwrap();
        assert_feq(lander.x, initial_x);
        assert_close(lander.y, 498., 0.15);
    }

    #[test]
    fn thrust_up() {
        let lander = LanderState::default().with_y(500.).with_fuel(100);
        let initial_x = lander.x;
        let lander = Physics::default()
            .iterate(
                lander,
                Thrust {
                    angle: 0.,
                    power: 4,
                },
            )
            .unwrap();
        assert_feq(lander.x, initial_x);
        assert_close(lander.y, 498.5, 0.15);
    }

    #[test]
    fn fly_up() {
        let mut lander = LanderState::default().with_y(500.).with_fuel(500);
        let initial_x = lander.x;
        let initial_y = lander.y;
        for _ in 0..41 {
            lander = Physics::default()
                .iterate(
                    lander,
                    Thrust {
                        angle: 0.,
                        power: 4,
                    },
                )
                .unwrap();
        }
        assert!(lander.y > initial_y);
        assert_feq(lander.x, initial_x);
    }

    #[test]
    fn fuel_consumption() {
        let lander = LanderState::default().with_y(500.).with_fuel(200);
        let iniitial_fuel = lander.fuel;
        let lander = Physics::default()
            .iterate(
                lander,
                Thrust {
                    angle: 0.,
                    power: 1,
                },
            )
            .unwrap();
        assert_eq!(iniitial_fuel - lander.fuel, 1);
    }
}
