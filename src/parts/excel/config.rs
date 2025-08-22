/// Параметры «шапки» и макета листа.
#[derive(Clone)]
pub struct ReportHeader {
    pub group: String,        // напр. "23ВЕБ-1"
    pub month_label: String,  // напр. "апрель"
    pub academic_year: String,// напр. "2024/2025"
    pub starosta: String,     // напр. "Атабаева Согдиана Дамировна"
    pub curator: String,      // напр. "Бокарева Светлана Флюровна"
}

impl ReportHeader {
    pub fn sample() -> Self {
        Self {
            group: "23ВЕБ-1".into(),
            month_label: "апрель".into(),
            academic_year: "2024/2025".into(),
            starosta: "Атабаева Согдиана Дамировна".into(),
            curator: "Бокарева Светлана Флюровна".into(),
        }
    }
}
