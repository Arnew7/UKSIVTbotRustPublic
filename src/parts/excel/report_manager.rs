use crate::parts::excel::{
    config::ReportHeader,
    models::{Student},
    traits::AttendanceApi,
    excel_report::{AttendanceBook, ExcelExporter},
};
use anyhow::{Result, Context};
use chrono::{Local, NaiveDate};
use std::{fs, path::{Path, PathBuf}};

const REPORTS_DIR: &str = "reports";
const ARCHIVE_DIR: &str = "reports/archive";



pub struct ReportManager {
    group: String,
    date: NaiveDate,
    book: AttendanceBook,
}

impl ReportManager {
    /// Создать или открыть отчёт
    pub fn open_or_create(group: &str, header: ReportHeader, days: Vec<NaiveDate>) -> Result<Self> {
        let date = Local::now().naive_local().date();
        let path = Self::report_path(group, date);

        let book = if path.exists() {
            // Пока не поддерживаем чтение: создаём пустую структуру
            AttendanceBook::new(header, days)
        } else {
            fs::create_dir_all(REPORTS_DIR)?;
            let book = AttendanceBook::new(header.clone(), days.clone());
            let exporter = ExcelExporter::new(&book, &header, &days);
            exporter.save(path.to_str().unwrap())?;
            book
        };

        Ok(Self {
            group: group.to_string(),
            date,
            book,
        })
    }

    /// Добавить студента
    pub fn add_student(&mut self, student: Student) {
        self.book.add_student(student);
    }

    /// Сохранить файл (пересоздаёт весь файл с нуля — важно!)
    pub fn save(&self) -> Result<()> {
        let path = Self::report_path(&self.group, self.date);
        let exporter = ExcelExporter::new(&self.book, &self.book.header, &self.book.days);
        exporter.save(path.to_str().unwrap())?;
        Ok(())
    }

    /// Архивировать отчёт (переместить)
    pub fn archive(&self) -> Result<()> {
        let src = Self::report_path(&self.group, self.date);
        if !src.exists() {
            return Err(anyhow::anyhow!("Файл не найден: {:?}", src));
        }

        let month_str = self.date.format("%Y-%m").to_string();
        let archive_dir = Path::new(ARCHIVE_DIR).join(month_str);
        fs::create_dir_all(&archive_dir)?;
        let dst = archive_dir.join(src.file_name().unwrap());
        fs::rename(&src, &dst)?;
        Ok(())
    }

    /// Путь до текущего отчёта
    pub fn report_path(group: &str, date: NaiveDate) -> PathBuf {
        Path::new(REPORTS_DIR).join(format!("{}_{}.xlsx", group, date.format("%Y-%m")))
    }

    /// Получить путь к файлу (для отправки, например)
    pub fn get_file_path(&self) -> PathBuf {
        Self::report_path(&self.group, self.date)
    }
}
