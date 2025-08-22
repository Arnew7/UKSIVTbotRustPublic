use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use rust_xlsxwriter::{Formula, Format, FormatAlign, FormatBorder, FormatPattern, Workbook, Worksheet, ColNum};

use crate::parts::excel::{config::ReportHeader, models::Student, traits::AttendanceApi};

/// Описание раскладки колонок (индексы 0-based).
/// AK = 36 (A=0, ..., Z=25, AA=26, ..., AK=36).
pub struct Layout;
impl Layout {
    pub const COL_A: u16 = 0;  // A
    pub const COL_B: u16 = 1;  // B
    pub const COL_C: u16 = 2;  // C (скрыта)
    pub const COL_D: u16 = 3;  // D (тонкая колонка перед днями)
    pub const COL_E: u16 = 4;  // E — первый "дневной" столбец, если надо
    pub const COL_AH: u16 = 33; // AH — 31-й день
    pub const COL_AI: u16 = 34; // "В"
    pub const COL_AJ: u16 = 35; // "У"
    pub const COL_AK: u16 = 36; // "Н"
}

/// Простой контейнер данных с API для заполнения (реализация интерфейса AttendanceApi).
pub struct AttendanceBook {
    pub header: ReportHeader,
    pub days: Vec<NaiveDate>,
    pub students: Vec<Student>,
}

impl AttendanceBook {
    pub fn new(header: ReportHeader, days: Vec<NaiveDate>) -> Self {
        Self {
            header,
            days,
            students: Vec::new(),
        }
    }
}

impl AttendanceApi for AttendanceBook {
    fn add_student(&mut self, student: Student) {
        self.students.push(student);
    }

    fn students(&self) -> &Vec<Student> {
        &self.students
    }
}

/// Экспортёр Excel: создаёт лист «посещаемость», максимально повторяющий твой шаблон.
pub struct ExcelExporter<'a, A: AttendanceApi> {
    book: &'a A,
    header: &'a ReportHeader,
    days: &'a [NaiveDate],
}

impl<'a, A: AttendanceApi> ExcelExporter<'a, A> {
    pub fn new(book: &'a A, header: &'a ReportHeader, days: &'a [NaiveDate]) -> Self {
        Self { book, header, days }
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet();

        // ------------------ Форматы ------------------
        // Базовый формат с тонкой рамкой и TNR 8pt.
        let mut base = Format::new();
        let base = base.set_border(FormatBorder::Thin).set_font_name("Times New Roman").set_font_size(8.0).clone();


        // Крупный жирный центрированный (шапка в A1..A3).
        let mut head_big = base.clone();
        let head_big = head_big
            .set_font_size(11.0)
            .set_bold()
            .set_align(FormatAlign::Center);


        // Мелкий жирный центрированный с переносом (строка заголовков).
        let mut head_small_center_bold = base.clone();
        let head_small_center_bold = head_small_center_bold
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_text_wrap();

        // Центр по горизонтали/вертикали.
        let mut center = base.clone();
        let center = center
            .set_align(FormatAlign::Center);


        // Имя: выравнивание по верхнему краю (как в оригинале).
        let mut name_fmt = base.clone();


        // Заголовки «В/У/Н» — с лёгкой заливкой.
        let mut wun_head = head_small_center_bold.clone();
        let wun_head = wun_head
            .set_pattern(FormatPattern::Solid);


        // Ячейки «В/У/Н» — тоже с заливкой и жирным.
        let mut wun_cell = center.clone();
        let wun_cell =wun_cell
            .set_bold()
            .set_pattern(FormatPattern::Solid);


        // ------------------ Ширины/высоты ------------------
        ws.set_column_width(Layout::COL_A.into(), 3.71)?;   // A
        ws.set_column_width(Layout::COL_B.into(), 27.43)?;  // B
        ws.set_column_width(Layout::COL_C.into(), 0.0)?;    // C (скрыта)
        ws.set_column_width(Layout::COL_D.into(), 3.43)?;   // D (узкая перед днями)

        // E..AG = 13.0 (это 30 столбцов: 1..30 дни)
        for col in (Layout::COL_E as u32)..=(32u32) {
            ws.set_column_width(col as ColNum, 13.0)?;
        }

        // AH = 3.0 (31-й день — узкий)
        ws.set_column_width(Layout::COL_AH.into(), 3.0)?;

        // AI..AK — под «В/У/Н»
        ws.set_column_width(Layout::COL_AI.into(), 4.57)?; // В
        ws.set_column_width(Layout::COL_AJ.into(), 4.43)?; // У
        ws.set_column_width(Layout::COL_AK.into(), 4.29)?; // Н

        // Высоты первых строк
        ws.set_row_height(0, 15.0)?;
        ws.set_row_height(1, 15.0)?;
        ws.set_row_height(2, 18.0)?;
        ws.set_row_height(3, 20.25)?;

        // ------------------ Шапка (мерджи и надписи) ------------------
        // A1:AK1
        ws.merge_range(
            0,
            Layout::COL_A,
            0,
            Layout::COL_AK,
            &format!("Отчет группы {}", self.header.group),
            &head_big,
        )?;

        // A2:AK2
        ws.merge_range(
            1,
            Layout::COL_A,
            1,
            Layout::COL_AK,
            &format!(
                "о посещаемости за {} {} учебного года ",
                self.header.month_label, self.header.academic_year
            ),
            &head_big,
        )?;

        // A3:O3 (O — 14)
        ws.merge_range(
            2,
            Layout::COL_A,
            2,
            14,
            &format!("       Староста: {}", self.header.starosta),
            &head_big,
        )?;

        // P3:AK3 (P=15 .. AK=36)
        ws.merge_range(
            2,
            15,
            2,
            Layout::COL_AK,
            &format!("Куратор: {}", self.header.curator),
            &head_big,
        )?;

        // ------------------ Строка заголовков (4-я строка) ------------------
        // A4
        ws.write_with_format(3, Layout::COL_A, "№ п/п", &head_small_center_bold)?;
        // B4 — важен точный текст с пробелами
        ws.write_with_format(3, Layout::COL_B, "Ф.И.О        Дни месяца", &head_small_center_bold)?;
        // C4 — скрыта; D4 — узкая разделительная колонка (оставляем пустой заголовок с форматом)
        ws.write_with_format(3, Layout::COL_C as u16, "", &head_small_center_bold)?;

        ws.write_with_format(3, Layout::COL_D as u16, "", &head_small_center_bold)?;


        // D4..AH4 — номера дней (в исходнике именно этот диапазон под 1..31)
        // Но фактически подписываем столько дней, сколько в self.days.
        for (i, day) in self.days.iter().enumerate() {
            let col = Layout::COL_D as u32 + i as u32;
            let label = day.day().to_string();
            ws.write_with_format(3, col as ColNum, label, &head_small_center_bold)?;
        }

        // AI4, AJ4, AK4
        ws.write_with_format(3, Layout::COL_AI, "В", &wun_head)?;
        ws.write_with_format(3, Layout::COL_AJ, "У", &wun_head)?;
        ws.write_with_format(3, Layout::COL_AK, "Н", &wun_head)?;

        // ------------------ Данные студентов ------------------
        // Первая строка с данными — 5-я визуально => row = 4 (0-based).
        for (idx, st) in self.book.students().iter().enumerate() {
            let row = 4 + idx as u32;

            // № п/п
            ws.write_with_format(row, Layout::COL_A, (idx + 1) as i32, &center)?;

            // ФИО
            ws.write_with_format(row, Layout::COL_B, &st.name, &name_fmt)?;

            // По дням: кладём часы, пустые — оставляем пустыми с границами.
            for (i, day) in self.days.iter().enumerate() {
                let col = Layout::COL_D as u32 + i as u32; // D.. (до AH)
                let val = st.hours_by_date.get(day).copied().unwrap_or(0);
                if val > 0 {
                    ws.write_with_format(row, col as ColNum, val as i32, &center)?;
                } else {
                    ws.write_with_format(row, col as u16, "", &center)?;

                }
            }

            // «В» — сумма по всем дням (D..D+days-1)
            let first_day_col = Layout::COL_D as u32;
            let last_day_col = first_day_col + (self.days.len() as u32).saturating_sub(1);
            let sum_formula = format!(
                "SUM({}:{})",
                to_a1(row, first_day_col),
                to_a1(row, last_day_col),
            );
            ws.write_formula_with_format(row, Layout::COL_AI as u16, &Formula::new(sum_formula), &wun_cell)?;

            // «У» — сумма уважительных часов (берём из структуры; пишем числом)
            let u_sum: i32 = st.excused_by_date.values().map(|v| *v as i32).sum();
            ws.write_with_format(row, Layout::COL_AJ, u_sum, &wun_cell)?;

            // «Н» — В - У, с IFERROR на случай пустых/текстовых.
            let n_formula = format!(
                "IFERROR({}-{},{})",
                to_a1(row, Layout::COL_AI as u32),
                to_a1(row, Layout::COL_AJ as u32),
                to_a1(row, Layout::COL_AI as u32)
            );
            ws.write_formula_with_format(row, Layout::COL_AK as u16, &Formula::new(n_formula), &wun_cell)?;
        }

        wb.save(path)?;
        Ok(())
    }
}

/// Преобразование координат (row, col) -> адрес A1.
/// row/col — 0-based.
fn to_a1(row: u32, col: u32) -> String {
    fn col_to_letters(mut col: u32) -> String {
        let mut s = String::new();
        let mut n = col + 1;
        while n > 0 {
            let rem = ((n - 1) % 26) as u8;
            s.insert(0, (b'A' + rem) as char);
            n = (n - 1) / 26;
        }
        s
    }
    format!("{}{}", col_to_letters(col), row + 1)
}

