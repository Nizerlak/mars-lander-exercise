use std::ops::RangeInclusive;

use super::Thrust;
use rand::Rng;

const ANGLE_RANGE: RangeInclusive<i32> = -90..=90;
const THRUST_RANGE: RangeInclusive<i32> = 0..=4;
const ANGLE_STEP_RANGE: RangeInclusive<i32> = -15..=15;
const THRUST_STEP_RANGE: RangeInclusive<i32> = -1..=1;

type Genes = Vec<i32>;

pub trait CommandProvider {
    fn get_cmd(&self, id: usize, sub_id: usize) -> Option<Thrust>;
}

pub struct Settings {
    pub num_of_runners: usize,
}

struct Chromosome {
    angles: Genes,
    thrusts: Genes,
}

fn new_random_angle(angle: i32) -> i32 {
    clamp(
        angle + rand::thread_rng().gen_range(ANGLE_STEP_RANGE),
        ANGLE_RANGE,
    )
}

fn new_random_thrust(thrust: i32) -> i32 {
    clamp(
        thrust + rand::thread_rng().gen_range(THRUST_STEP_RANGE),
        THRUST_RANGE,
    )
}

fn clamp(v: i32, range: RangeInclusive<i32>) -> i32 {
    *range.start().max(range.end().min(&v))
}

fn crossed(a: &Genes, b: &Genes, i: usize) -> Option<(Genes, Genes)> {
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
    pub fn new_random(size: usize, initial_angle: i32, inital_thrust: i32) -> Self {
        (0..size).fold(
            Self {
                angles: vec![new_random_angle(initial_angle)],
                thrusts: vec![new_random_thrust(inital_thrust)],
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

    pub fn mutate(&mut self, mutation_point: f64)-> Option<()>{
        let i = (mutation_point * self.angles.len() as f64) as usize;
        let new_angle = new_random_angle(*self.angles.get(i)?);
        let new_thrust = new_random_thrust(*self.angles.get(i)?);
        self.angles[i] = new_angle;
        self.thrusts[i] = new_thrust;
        Some(())
    }
}

struct Solver {
    population: Vec<Chromosome>,
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
