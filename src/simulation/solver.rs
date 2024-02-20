use std::ops::RangeInclusive;

use rand::Rng;

type Angle = i32;
type Thrust = i32;

const ANGLE_RANGE: RangeInclusive<Angle> = -90..=90;
const THRUST_RANGE: RangeInclusive<Thrust> = 0..=4;
const ANGLE_STEP_RANGE: RangeInclusive<Angle> = -15..=15;
const THRUST_STEP_RANGE: RangeInclusive<Thrust> = -1..=1;

type AngleGenes = Vec<Angle>;
type ThrustGenes = Vec<Thrust>;

pub trait CommandProvider {
    fn get_cmd(&self, id: usize, sub_id: usize) -> Option<super::Thrust>;
}

pub struct Settings {
    pub num_of_runners: usize,
}

struct Chromosome {
    angles: AngleGenes,
    thrusts: ThrustGenes,
}

fn new_random_angle(angle: Angle) -> Angle {
    clamp(
        angle + rand::thread_rng().gen_range(ANGLE_STEP_RANGE),
        ANGLE_RANGE,
    )
}

fn new_random_thrust(thrust: Thrust) -> Thrust {
    clamp(
        thrust + rand::thread_rng().gen_range(THRUST_STEP_RANGE),
        THRUST_RANGE,
    )
}

fn clamp(v: i32, range: RangeInclusive<i32>) -> i32 {
    *range.start().max(range.end().min(&v))
}

fn crossed(a: &Vec<i32>, b: &Vec<i32>, i: usize) -> Option<(Vec<i32>, Vec<i32>)> {
    if a.len() != b.len() {
        return None;
    } else if i >= a.len() {
        return None;
    }

    let (mut x, mut y) = (a[..i].to_vec(), b[..i].to_vec());
    x.extend(&b[i..]);
    y.extend(&a[i..]);
    Some((x, y))
}

impl Chromosome {
    pub fn new_random(size: usize, initial_angle: Angle, initial_thrust: Thrust) -> Self {
        (0..size - 1).fold(
            Self {
                angles: vec![new_random_angle(initial_angle)],
                thrusts: vec![new_random_thrust(initial_thrust)],
            },
            |mut s, _| {
                let new_angle = new_random_angle(*s.angles.last().unwrap());
                let new_thrust = new_random_thrust(*s.thrusts.last().unwrap());
                s.angles.push(new_angle);
                s.thrusts.push(new_thrust);
                s
            },
        )
    }

    pub fn get_cmd(&self, id: usize) -> Option<super::Thrust> {
        Some(super::Thrust::new(
            *self.angles.get(id)? as f64,
            *self.thrusts.get(id)?,
        ))
    }

    pub fn crossover(&self, other: &Self, cross_point: f64) -> Option<(Self, Self)> {
        let i = (cross_point * self.angles.len() as f64) as usize;
        let (angles_a, angles_b) = crossed(&self.angles, &other.angles, i)?;
        let (thrusts_a, thrusts_b) = crossed(&self.thrusts, &other.thrusts, i)?;
        Some((
            Self {
                angles: angles_a,
                thrusts: thrusts_a,
            },
            Self {
                angles: angles_b,
                thrusts: thrusts_b,
            },
        ))
    }

    pub fn mutate(&mut self, mutation_point: f64) -> Option<()> {
        let i = (mutation_point * self.angles.len() as f64) as usize;
        let new_angle = new_random_angle(*self.angles.get(i)?);
        let new_thrust = new_random_thrust(*self.angles.get(i)?);
        self.angles[i] = new_angle;
        self.thrusts[i] = new_thrust;
        Some(())
    }
}

pub struct Solver {
    population: Vec<Chromosome>,
}

impl CommandProvider for Solver {
    fn get_cmd(&self, id: usize, sub_id: usize) -> Option<super::Thrust> {
        self.population.get(id)?.get_cmd(sub_id)
    }
}

pub struct SolverSettings {
    pub population_size: usize,
    pub chromosome_size: usize,
    pub initial_angle: i32,
    pub initial_thrust: i32,
}

impl Solver {
    pub fn new(settings: SolverSettings) -> Self {
        Self {
            population: (0..settings.population_size).fold(Vec::new(), |mut population, _| {
                population.push(Chromosome::new_random(
                    settings.chromosome_size,
                    settings.initial_angle,
                    settings.initial_thrust,
                ));
                population
            }),
        }
    }
}

struct FitnessCalculator {
    target: ((f64, f64), f64),
    landing_bias: f64,
}

type Point = (f64, f64);

fn normalized(v: impl Iterator<Item = f64> + Clone) -> Option<impl Iterator<Item = f64>> {
    let max = v.clone().map(|v| v.abs()).max_by(|a, b| a.total_cmp(b))?;
    let max = if max == 0. { 1. } else { max };
    Some(v.map(move |v| v / max))
}

impl FitnessCalculator {
    fn dist_to_target(&self, (x, y): Point) -> f64 {
        let ((tx1, tx2), ty) = &self.target;
        let dist =
            |(a1, a2): Point, (b1, b2): Point| ((a1 - a2).powi(2) + (b1 - b2).powi(2)).sqrt();
        if x < *tx1 {
            dist((x, y), (*tx1, *ty))
        } else if x > *tx2 {
            dist((x, y), (*tx2, *ty))
        } else {
            (ty - y).abs()
        }
    }

    pub fn calculate_fitness(
        &self,
        landing_points: &Vec<Point>,
        landing_results: &Vec<super::Landing>,
    ) -> Option<Vec<f64>> {
        let distances =
            normalized(landing_points.iter().map(|p| self.dist_to_target(*p)))?.map(|v| 1. - v);

        use crate::Landing;
        let some_or_max = |a: Option<f64>, er: f64| Some(a.map_or(er, |v| v.max(er)));
        let landed_normalized = |error: f64, max: Option<f64>| {
            self.landing_bias + (1. - self.landing_bias) * error / max.unwrap()
        };

        let (nv, tfh, tfv) =
            landing_results
                .iter()
                .fold((None, None, None), |(a, b, c), landing| match landing {
                    Landing::NotVertical { error } => (some_or_max(a, error.abs()), b, c),
                    Landing::TooFastHorizontal { error } => (a, some_or_max(b, error.abs()), c),
                    Landing::TooFastVertical { error } => (a, b, some_or_max(c, error.abs())),
                    _ => (a, b, c),
                });
        Some(
            landing_results
                .iter()
                .zip(distances)
                .map(|(result, dist_points)| match result {
                    &Landing::Correct => 1.,
                    &Landing::NotVertical { error } => landed_normalized(error, nv),
                    &Landing::TooFastHorizontal { error } => landed_normalized(error, tfh),
                    &Landing::TooFastVertical { error } => landed_normalized(error, tfv),
                    &Landing::WrongTerrain | &Landing::OutOfMap => dist_points * self.landing_bias,
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod crossing_test {
    use super::crossed;

    #[test]
    fn different_vecs() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5];
        assert!(crossed(&a, &b, 2).is_none());
    }

    #[test]
    fn wrong_i1() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        assert!(crossed(&a, &b, 3).is_none());
    }

    #[test]
    fn wrong_i2() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        assert!(crossed(&a, &b, 4).is_none());
    }

    #[test]
    fn crossing1() {
        let a1 = vec![1, 2, 3, 4];
        let b1 = vec![5, 6, 7, 8];
        let (a2, b2) = crossed(&a1, &b1, 1).unwrap();
        assert_eq!(a2, vec![1, 6, 7, 8]);
        assert_eq!(b2, vec![5, 2, 3, 4]);
    }

    #[test]
    fn crossing2() {
        let a1 = vec![1, 2, 3, 4];
        let b1 = vec![5, 6, 7, 8];
        let (a2, b2) = crossed(&a1, &b1, 2).unwrap();
        assert_eq!(a2, vec![1, 2, 7, 8]);
        assert_eq!(b2, vec![5, 6, 3, 4]);
    }
}
