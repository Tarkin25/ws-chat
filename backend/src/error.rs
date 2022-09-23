use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub enum Error {
    UsernameTaken
}