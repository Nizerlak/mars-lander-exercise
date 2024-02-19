use super::Thrust;

pub trait CommandProvider{
    fn get_cmd(&self, id: usize, sub_id: usize) -> Option<Thrust>;
}

pub struct Settings{
    pub num_of_runners: usize,
}