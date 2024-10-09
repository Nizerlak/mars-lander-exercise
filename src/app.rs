use crate::{init, simulation::*};

pub struct App {
    terrain: Terrain,
    lander_runner: LanderRunner,
    initial_lander_state: LanderState,
    flight_histories: Vec<LanderHistory>,
    solver: Solver,
    fitness_calculator: FitnessCalculator,
    current_fitness: Vec<f64>,
    population_id: usize,
}

impl App {
    pub fn try_new(sim_file_path: &String, settings_file_path: &String) -> Result<Self, String> {
        let (initial_lander_state, terrain) = init::json::parse_sim(sim_file_path)?;
        let settings = init::json::parse_settings(settings_file_path)?;
        let solver_settings = SolverSettings {
            chromosome_size: settings.chromosome_size,
            elitism: settings.elitism,
            mutation_prob: settings.mutation_prob,
            population_size: settings.population_size,
            initial_angle: initial_lander_state.angle as i32,
            initial_thrust: initial_lander_state.power,
        };
        let solver = Solver::try_new(solver_settings)?;
        let fitness_calculator = FitnessCalculator::new(
            Self::target_from_terrain(&terrain).ok_or("Cannot get target from terrain")?,
            settings.landing_bias,
        );
        let lander_runner = LanderRunner::new(
            initial_lander_state.clone(),
            settings.population_size,
            Physics::default(),
            CollisionChecker::default(),
        );
        let flight_histories: Vec<_> =
            vec![
                LanderHistory::with_initial_state(initial_lander_state.clone());
                settings.population_size
            ];

        Ok(Self {
            terrain,
            lander_runner,
            initial_lander_state,
            flight_histories,
            solver,
            fitness_calculator,
            current_fitness: vec![0f64; settings.population_size],
            population_id: 0,
        })
    }

    pub fn next_population(&mut self) -> Result<(), String> {
        let fitness = self.fitness_calculator.calculate_fitness(
            &self
                .lander_runner
                .current_landers_states()
                .map(|l| (l.x, l.y))
                .collect(),
            &self
                .lander_runner
                .current_flight_states()
                .map(|f| {
                    if let FlightState::Landed(l) = f {
                        Ok(l.clone())
                    } else {
                        Err(format!("Lander not landed: {f:?}"))
                    }
                })
                .collect::<Result<Vec<_>, String>>()?,
        ).ok_or("Failed to calculate fitness")?;
        self.solver.new_generation(fitness.iter().copied())?;
        self.lander_runner
            .reinitialize(self.initial_lander_state.clone());
        self.flight_histories.iter_mut().for_each(|h| {
            *h = LanderHistory::with_initial_state(self.initial_lander_state.clone())
        });
        self.current_fitness = fitness;
        self.population_id += 1;
        Ok(())
    }

    pub fn run(&mut self) -> Result<ExecutionStatus, String> {
        let res = loop {
            match self
                .lander_runner
                .iterate(&self.solver, &self.terrain)
                .map_err(|e| e.to_string())
            {
                Ok(ExecutionStatus::InProgress) => self.save_last_lander_states_in_flight(),
                e => break e,
            }
        };
        self.save_last_lander_states();
        res
    }

    pub fn get_routes(&self) -> impl Iterator<Item = impl Iterator<Item = LanderState> + '_> + '_ {
        self.flight_histories.iter().map(|h| h.iter_history())
    }

    pub fn get_current_fitness(&self) -> impl Iterator<Item = f64> + '_ {
        self.current_fitness.iter().copied()
    }

    pub fn get_current_states(&self) -> impl Iterator<Item = &FlightState> + '_ {
        self.lander_runner.current_flight_states()
    }

    pub fn get_population(&self) -> impl Iterator<Item = &Chromosome> + '_ {
        self.solver.iter_population()
    }

    pub fn get_population_accumulated(&self) -> impl Iterator<Item = &Chromosome> + '_ {
        self.solver.iter_accumulated_population()
    }

    pub fn get_terrain(&self) -> &Terrain {
        &self.terrain
    }

    pub fn get_population_id(&self) -> usize {
        self.population_id
    }

    fn target_from_terrain(terrain: &Terrain) -> Option<((f64, f64), f64)> {
        for (x, y) in terrain
            .x
            .as_slice()
            .windows(2)
            .zip(terrain.y.as_slice().windows(2))
        {
            if y[0] == y[1] {
                return Some(((x[0], x[1]), y[0]));
            }
        }
        None
    }

    fn save_last_lander_states_in_flight(&mut self) {
        self.flight_histories
            .iter_mut()
            .zip(self.lander_runner.current_landers_states())
            .zip(self.lander_runner.current_flight_states())
            .filter_map(|(x, s)| {
                if let FlightState::Flying = s {
                    Some(x)
                } else {
                    None
                }
            })
            .for_each(|(h, s)| h.append_lander_state(s))
    }

    fn save_last_lander_states(&mut self) {
        self.flight_histories
            .iter_mut()
            .zip(self.lander_runner.current_landers_states())
            .for_each(|(h, s)| h.append_lander_state(s))
    }

    pub fn print_flight_state_results(&self) {
        println!("{}", self.flight_histories[0].pretty_to_string());

        let flight_state = self.lander_runner.current_flight_states().next().unwrap();
        println!("Finished {:?}", flight_state);
    }
}
