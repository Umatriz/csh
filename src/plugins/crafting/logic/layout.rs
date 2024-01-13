#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Layout<T>(pub Vec<T>);

impl<T> Layout<T> {
    pub fn is_one(&self) -> bool {
        self.0.len() == 1
    }

    pub fn is_many(&self) -> bool {
        self.0.len() > 1
    }

    pub fn get_many(&self) -> Option<&Vec<T>> {
        match self.is_many() {
            true => Some(&self.0),
            false => None,
        }
    }

    pub fn get_one(&self) -> Option<&T> {
        match self.is_one() {
            true => self.0.first(),
            false => None,
        }
    }

    pub fn get(&self) -> &Vec<T> {
        &self.0
    }
}
