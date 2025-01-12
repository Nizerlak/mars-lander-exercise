use crate::simulation::*;

pub struct App {
    terrain: Terrain,
    lander_runner: LanderRunner,
    initial_lander_state: LanderState,
    flight_histories: Vec<LanderHistory>,
    solver: Solver,
    current_fitness: Vec<f64>,
    population_id: usize,
}

impl App {
    pub fn try_new(
        initial_lander_state: LanderState,
        terrain: Terrain,
        settings: Settings,
    ) -> Result<Self, String> {
        let solver_settings = SolverSettings {
            chromosome_size: settings.chromosome_size,
            elitism: settings.elitism,
            mutation_prob: settings.mutation_prob,
            population_size: settings.population_size,
            initial_angle: initial_lander_state.angle as i32,
            initial_thrust: initial_lander_state.power,
        };
        let solver = Solver::try_new(solver_settings)?;
        let lander_runner = LanderRunner::new(
            initial_lander_state.clone(),
            settings.population_size,
            Physics::default(),
            CollisionChecker::try_with_default_limits(terrain.clone())
                .ok_or("Failed to create collision checker")?,
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
            current_fitness: vec![0f64; settings.population_size],
            population_id: 0,
        })
    }

    pub fn next_population(&mut self) -> Result<(), String> {
        let fitness = calculate_fitness(
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
        )
        .ok_or("Failed to calculate fitness")?;
        self.solver.new_generation(fitness.iter().copied())?;
        self.current_fitness = fitness;
        self.population_id += 1;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.lander_runner
            .reinitialize(self.initial_lander_state.clone());
        self.flight_histories.iter_mut().for_each(|h| {
            *h = LanderHistory::with_initial_state(self.initial_lander_state.clone())
        });
        let mut population: Vec<_> = self.solver.iter_accumulated_population().collect();
        while let ExecutionStatus::InProgress = self
            .lander_runner
            .iterate(&mut population)
            .map_err(|e| e.to_string())?
        {
            self.save_last_lander_states_in_flight();
        }
        self.save_last_lander_states();
        Ok(())
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

    pub fn get_population_accumulated(&self) -> impl Iterator<Item = Chromosome> + '_ {
        self.solver.iter_accumulated_population()
    }

    pub fn get_terrain(&self) -> &Terrain {
        &self.terrain
    }

    pub fn get_population_id(&self) -> usize {
        self.population_id
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
