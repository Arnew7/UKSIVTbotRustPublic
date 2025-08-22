use crate::parts::excel::models::Student;

pub trait AttendanceApi {
    fn add_student(&mut self, student: Student);
    fn students(&self) -> &Vec<Student>;
}
