use bincode::{Decode, Encode};

#[derive(Encode, Decode)]
pub struct Slot {
    pub appetizer_id: String,
    pub main_course_id: String,
    pub dessert_id: String,
    pub accompaniment_id: Option<String>,
}
