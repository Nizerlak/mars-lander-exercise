use super::Thrust;

pub trait CommandProvider{
    fn get_cmd(&self, id: usize) -> Option<Thrust>;
}