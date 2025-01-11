use std::ops::RangeInclusive;

use rand::{seq::SliceRandom, Rng};

type Angle = i32;
type Thrust = i32;

const ANGLE_RANGE: RangeInclusive<Angle> = -90..=90;
const THRUST_RANGE: RangeInclusive<Thrust> = 0..=4;
const ANGLE_STEP_RANGE: RangeInclusive<Angle> = -15..=15;
const THRUST_STEP_RANGE: RangeInclusive<Thrust> = -1..=1;

macro_rules! clamp {
    ($range:ident) => {
        |x| clamp(x, $range)
    };
}

type AngleGenes = Vec<Angle>;
type ThrustGenes = Vec<Thrust>;

pub trait CommandProvider {
    fn get_cmd(&self, id: usize, sub_id: usize) -> Option<super::Thrust>;
    fn get_last_cmd(&self, id: usize) -> Option<super::Thrust>;
}

pub struct Settings {
    pub population_size: usize,
    pub chromosome_size: usize,
    pub elitism: f64,
    pub mutation_prob: f64,
    pub landing_bias: f64,
}

pub struct SolverSettings {
    pub population_size: usize,
    pub chromosome_size: usize,
    pub initial_angle: i32,
    pub initial_thrust: i32,
    pub elitism: f64,
    pub mutation_prob: f64,
}

#[derive(Clone, Debug)]
pub struct Chromosome {
    pub angles: AngleGenes,
    pub thrusts: ThrustGenes,
}

pub struct Solver {
    population: Vec<Chromosome>,
    elitism: f64,
    mutation_prob: f64,
    initial_angle: Angle,
    initial_thrust: Thrust,
    accumulated_population: Vec<Chromosome>,
}

pub struct FitnessCalculator {
    target: (f64, f64),
    landing_bias: f64,
}

fn new_random_angle() -> Angle {
    rand::thread_rng().gen_range(ANGLE_STEP_RANGE)
}

fn new_random_thrust() -> Thrust {
    rand::thread_rng().gen_range(THRUST_STEP_RANGE)
}

fn clamp(v: i32, range: RangeInclusive<i32>) -> i32 {
    *range.start().max(range.end().min(&v))
}

fn crossed(
    a: &[i32],
    b: &[i32],
    i: f64,
    clamp: impl Fn(i32) -> i32,
) -> Result<(Vec<i32>, Vec<i32>), String> {
    if a.len() != b.len() {
        return Err(format!("a.len() != b.len() ({} != {})", a.len(), b.len()));
    } else if !(0f64..=1f64).contains(&i) {
        return Err(format!("i out of range [0,1], i={i}"));
    }

    let (x, y) = a
        .iter()
        .zip(b)
        .fold((Vec::new(), Vec::new()), |(mut x, mut y), (a, b)| {
            let a = *a as f64;
            let b = *b as f64;
            let xp = (i * a + (1f64 - i) * b).round() as i32;
            let yp = (i * b + (1f64 - i) * a).round() as i32;
            x.push(clamp(xp));
            y.push(clamp(yp));
            (x, y)
        });
    Ok((x, y))
}

fn accumulated(
    initial_value: i32,
    i: impl Iterator<Item = i32>,
    clamp: impl Fn(i32) -> i32,
) -> impl Iterator<Item = i32> {
    i.scan(initial_value, move |value, o| {
        *value = clamp(*value + o);
        Some(*value)
    })
}

impl Chromosome {
    pub fn new_random(size: usize) -> Self {
        Self {
            angles: (0..size).map(|_| new_random_angle()).collect(),
            thrusts: (0..size).map(|_| new_random_thrust()).collect(),
        }
    }

    pub fn get_cmd(&self, id: usize) -> Option<super::Thrust> {
        Some(super::Thrust::new(
            *self.angles.get(id)? as f64,
            *self.thrusts.get(id)?,
        ))
    }

    pub fn get_last_cmd(&self) -> Option<super::Thrust> {
        Some(super::Thrust::new(
            *self.angles.last()? as f64,
            *self.thrusts.last()?,
        ))
    }

    pub fn crossover(&self, other: &Self, cross_point: f64) -> Result<(Self, Self), String> {
        let (angles_a, angles_b) = crossed(
            &self.angles,
            &other.angles,
            cross_point,
            clamp!(ANGLE_STEP_RANGE),
        )
        .map_err(|e| format!("Failed to cross angles, cross point: {cross_point}\n{e}"))?;
        let (thrusts_a, thrusts_b) = crossed(
            &self.thrusts,
            &other.thrusts,
            cross_point,
            clamp!(THRUST_STEP_RANGE),
        )
        .map_err(|e| format!("Failed to cross thrusts, cross point: {cross_point}\n{e}"))?;
        Ok((
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

    pub fn mutate(&mut self, mutation_prob: f64) -> Option<()> {
        self.angles
            .iter_mut()
            .zip(self.thrusts.iter_mut())
            .for_each(|(angle, thrust)| {
                if rand::thread_rng().gen_range(0f64..1f64) < mutation_prob {
                    *angle = clamp(new_random_angle(), ANGLE_STEP_RANGE);
                    *thrust = clamp(new_random_thrust(), THRUST_STEP_RANGE);
                }
            });
        Some(())
    }
}

impl CommandProvider for Solver {
    fn get_cmd(&self, id: usize, sub_id: usize) -> Option<super::Thrust> {
        self.accumulated_population.get(id)?.get_cmd(sub_id)
    }

    fn get_last_cmd(&self, id: usize) -> Option<super::Thrust> {
        self.accumulated_population.get(id)?.get_last_cmd()
    }
}

impl Solver {
    pub fn try_new(settings: SolverSettings) -> Result<Self, String> {
        if settings.elitism < 0f64 || settings.elitism > 1f64 {
            return Err(format!("Elitism ({}) out of range [0,1]", settings.elitism));
        }
        if settings.mutation_prob < 0f64 || settings.mutation_prob > 1f64 {
            return Err(format!(
                "MutationProb ({}) out of range [0,1]",
                settings.mutation_prob
            ));
        }
        let population: Vec<_> = (0..settings.population_size)
            .map(|_| Chromosome::new_random(settings.chromosome_size))
            .collect();
        let accumulated_population = Self::accumulated_population(
            settings.initial_angle,
            settings.initial_thrust,
            &population,
        );
        Ok(Self {
            population,
            elitism: settings.elitism,
            mutation_prob: settings.mutation_prob,
            initial_angle: settings.initial_angle,
            initial_thrust: settings.initial_thrust,
            accumulated_population,
        })
    }

    pub fn new_generation(&mut self, fitness: impl Iterator<Item = f64>) -> Result<(), String> {
        let len_population_before = self.population.len();
        let parents = self.choose_parents(fitness);
        let n_children = len_population_before - parents.len();
        let mut new_pop = self.mate(self.population.iter().collect(), n_children)?;
        new_pop.extend(parents.iter().map(|c| (**c).clone()));
        self.population = new_pop;
        self.accumulated_population =
            Self::accumulated_population(self.initial_angle, self.initial_thrust, &self.population);
        assert_eq!(len_population_before, self.population.len());
        Ok(())
    }

    fn choose_parents(&self, fitness: impl Iterator<Item = f64>) -> Vec<&Chromosome> {
        let mut ranking = self.population.iter().zip(fitness).collect::<Vec<_>>();
        ranking.sort_by(|(_, fitness1), (_, fitness2)| fitness1.total_cmp(fitness2).reverse());

        let n_best = (self.elitism * self.population.len() as f64) as usize;
        ranking[..n_best].iter().map(|(c, _)| *c).collect()
    }

    fn mate(
        &self,
        parents: Vec<&Chromosome>,
        n_children: usize,
    ) -> Result<Vec<Chromosome>, String> {
        let new_population =
            (0..n_children / 2).try_fold(Vec::new(), |mut new_population, _| {
                let mut r = parents.choose_multiple(&mut rand::thread_rng(), 2);
                let parent1 = r.next().ok_or("Can't get parent1")?;
                let parent2 = r.next().ok_or("Can't get parent2")?;
                let (mut c1, mut c2) =
                    parent1.crossover(parent2, rand::thread_rng().gen_range(0f64..1f64))?;
                c1.mutate(self.mutation_prob);
                c2.mutate(self.mutation_prob);
                new_population.push(c1);
                new_population.push(c2);
                Ok::<Vec<_>, String>(new_population)
            })?;
        Ok(new_population)
    }

    fn accumulated_population(
        initial_angle: i32,
        initial_thrust: i32,
        population: &[Chromosome],
    ) -> Vec<Chromosome> {
        population.iter().fold(
            Vec::new(),
            |mut population, Chromosome { angles, thrusts }| {
                population.push(Chromosome {
                    angles: accumulated(initial_angle, angles.iter().copied(), clamp!(ANGLE_RANGE))
                        .collect(),
                    thrusts: accumulated(
                        initial_thrust,
                        thrusts.iter().copied(),
                        clamp!(THRUST_RANGE),
                    )
                    .collect(),
                });
                population
            },
        )
    }

    pub fn iter_population(&self) -> impl Iterator<Item = &Chromosome> {
        self.population.iter()
    }

    pub fn iter_accumulated_population(&self) -> impl Iterator<Item = &Chromosome> {
        self.accumulated_population.iter()
    }
}

impl FitnessCalculator {
    pub fn new(target: (f64, f64), landing_bias: f64) -> Self {
        Self {
            target,
            landing_bias,
        }
    }

    pub fn calculate_fitness(&self, landing_results: &[super::Landing]) -> Option<Vec<f64>> {
        use crate::Landing;
        let some_or_max = |a: Option<f64>, er: f64| Some(a.map_or(er, |v| v.max(er)));
        let landed_normalized = |error: f64, max: Option<f64>| {
            self.landing_bias + (1. - self.landing_bias) * error / max.unwrap()
        };

        let dist_points = |dist: f64| {
            (self.target.0 - dist)
                .abs()
                .min((self.target.1 - dist).abs())
        };

        let (err_max, dist_max) =
            landing_results
                .iter()
                .fold(
                    (None, None),
                    |(landing_err_max, dist_max), landing| match landing {
                        &Landing::NotVertical { error_rel, .. }
                        | &Landing::TooFastHorizontal { error_rel, .. }
                        | &Landing::TooFastVertical { error_rel, .. } => {
                            (some_or_max(landing_err_max, error_rel), dist_max)
                        }
                        &Landing::WrongTerrain { dist } => {
                            (landing_err_max, some_or_max(dist_max, dist_points(dist)))
                        }
                        _ => (landing_err_max, dist_max),
                    },
                );
        Some(
            landing_results
                .iter()
                .map(|result| match result {
                    &Landing::Correct => 1.,
                    &Landing::NotVertical { error_rel, .. }
                    | &Landing::TooFastHorizontal { error_rel, .. }
                    | &Landing::TooFastVertical { error_rel, .. } => {
                        landed_normalized(error_rel, err_max)
                    }
                    &Landing::WrongTerrain { dist } => {
                        (1. - dist_points(dist) / dist_max.unwrap()) * self.landing_bias
                    }
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod crossing_test {
    use super::crossed;

    fn pass(x: i32) -> i32 {
        x
    }

    #[test]
    fn different_vecs() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5];
        assert!(crossed(&a, &b, 0.5, pass).is_err());
    }

    #[test]
    fn wrong_i1() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        assert!(crossed(&a, &b, -0.5, pass).is_err());
    }

    #[test]
    fn wrong_i2() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        assert!(crossed(&a, &b, 1.5, pass).is_err());
    }

    #[test]
    fn crossing1() {
        let a1 = vec![1, 2, 3, 4];
        let b1 = vec![5, 6, 7, 8];
        let (a2, b2) = crossed(&a1, &b1, 0.25, pass).unwrap();
        assert_eq!(a2, vec![4, 5, 6, 7]);
        assert_eq!(b2, vec![2, 3, 4, 5]);
    }

    #[test]
    fn crossing2() {
        let a1 = vec![1, 2, 3, 4];
        let b1 = vec![5, 6, 7, 8];
        let (a2, b2) = crossed(&a1, &b1, 0.5, pass).unwrap();
        assert_eq!(a2, vec![3, 4, 5, 6]);
        assert_eq!(b2, vec![3, 4, 5, 6]);
    }

    #[test]
    fn crossing_clamped() {
        let a1 = vec![1, 2, 3, 4];
        let b1 = vec![5, 6, 7, 8];
        let (a2, b2) = crossed(&a1, &b1, 0.5, |x| x.min(4)).unwrap();
        assert_eq!(a2, vec![3, 4, 4, 4]);
        assert_eq!(b2, vec![3, 4, 4, 4]);
    }
}

#[cfg(test)]
mod accumulation_test {
    use super::accumulated;

    fn pass(x: i32) -> i32 {
        x
    }

    #[test]
    fn accumulate1() {
        let a = [1, 1, 1, 1];
        let a: Vec<_> = accumulated(0, a.iter().copied(), pass).collect();
        assert_eq!(a, vec![1, 2, 3, 4]);
    }

    #[test]
    fn accumulate2() {
        let a = [1, 1, 1, 1];
        let a: Vec<_> = accumulated(3, a.iter().copied(), pass).collect();
        assert_eq!(a, vec![4, 5, 6, 7]);
    }

    #[test]
    fn accumulate_clamped() {
        let a = [1, 1, 1, 1];
        let a: Vec<_> = accumulated(3, a.iter().copied(), |x| x.min(6)).collect();
        assert_eq!(a, vec![4, 5, 6, 6]);
    }
}
