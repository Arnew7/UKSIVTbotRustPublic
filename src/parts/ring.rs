
use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, TimeZone, Timelike, Weekday};
use std::collections::HashMap;

// Структура для представления времени
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LessonTime {
    hour: u32,
    minute: u32,
}

impl LessonTime {
    fn new(hour: u32, minute: u32) -> Self {
        LessonTime { hour, minute }
    }
}

// Функция для преобразования LessonTime в DateTime<Local>
fn lesson_time_to_datetime(lesson_time: LessonTime, today: chrono::Date<Local>) -> DateTime<Local> {
    today.and_hms_opt(lesson_time.hour, lesson_time.minute, 0).unwrap()
}

// Тип для расписания: номер урока -> время
type DaySchedule = HashMap<u32, LessonTime>;

// Тип для расписания: день недели -> номер урока -> время
type Schedule = HashMap<u32, DaySchedule>;

// Хардкод расписания
fn get_hardcoded_schedule() -> (Schedule, DaySchedule, Schedule, DaySchedule) {
    // Расписание с обедом
    let mut schedule_with_lunch: Schedule = HashMap::new();
    let mut default_schedule_with_lunch: DaySchedule = HashMap::new();

    // Расписание для субботы (weekday 5)
    let mut saturday_schedule: DaySchedule = HashMap::new();
    saturday_schedule.insert(1, LessonTime::new(8, 0));
    saturday_schedule.insert(2, LessonTime::new(9, 30));
    saturday_schedule.insert(3, LessonTime::new(11, 0));
    saturday_schedule.insert(4, LessonTime::new(12, 30));
    saturday_schedule.insert(5, LessonTime::new(14, 0));
    saturday_schedule.insert(6, LessonTime::new(15, 30));
    saturday_schedule.insert(7, LessonTime::new(17, 0));
    schedule_with_lunch.insert(6, saturday_schedule); // Corrected to 6, Saturday is 6.

    // Расписание по умолчанию (с обедом)
    default_schedule_with_lunch.insert(1, LessonTime::new(7, 50));
    default_schedule_with_lunch.insert(2, LessonTime::new(9, 30));
    default_schedule_with_lunch.insert(3, LessonTime::new(11, 15));
    default_schedule_with_lunch.insert(4, LessonTime::new(13, 35));
    default_schedule_with_lunch.insert(5, LessonTime::new(15, 20));
    default_schedule_with_lunch.insert(6, LessonTime::new(17, 0));
    default_schedule_with_lunch.insert(7, LessonTime::new(18, 30));

    // Расписание без обеда. На субботу не нужно
    let mut schedule_without_lunch: Schedule = HashMap::new();
    let mut default_schedule_without_lunch: DaySchedule = HashMap::new();

    // Расписание по умолчанию без обеда
    default_schedule_without_lunch.insert(1, LessonTime::new(7, 50));
    default_schedule_without_lunch.insert(2, LessonTime::new(9, 30));
    default_schedule_without_lunch.insert(3, LessonTime::new(11, 15));
    default_schedule_without_lunch.insert(4, LessonTime::new(12, 50));
    default_schedule_without_lunch.insert(5, LessonTime::new(14, 30));
    default_schedule_without_lunch.insert(6, LessonTime::new(16, 10));
    default_schedule_without_lunch.insert(7, LessonTime::new(17, 50));

    (
        schedule_with_lunch,
        default_schedule_with_lunch,
        schedule_without_lunch,
        default_schedule_without_lunch,
    )
}

// Helper function to calculate the time delta
async fn calculate_time_delta(lesson_time: LessonTime, today: chrono::Date<Local>) -> Result<(i64, i64), String> {
    let pair_datetime = lesson_time_to_datetime(lesson_time, today);
    let now_datetime = Local::now();
    let delta = pair_datetime.signed_duration_since(now_datetime);

    if delta < Duration::zero() {
        return Err("Пара уже прошла".into());
    }

    let hours = delta.num_hours();
    let minutes = (delta.num_minutes() % 60) as i64; // Cast to i64
    let hours = hours as i64;

    Ok((hours, minutes))
}

// API 1: Получение времени до указанной пары
pub async fn get_time_delta(n_lesson: u32) -> Result<String, String> {
    let (schedule_with_lunch, default_schedule_with_lunch, schedule_without_lunch, default_schedule_without_lunch) = get_hardcoded_schedule();

    let today = Local::today();
    let weekday = today.weekday();
    let weekday_num = weekday.number_from_monday();

    let ring_with_lunch: &DaySchedule = schedule_with_lunch
        .get(&weekday_num)
        .unwrap_or(&default_schedule_with_lunch);

    let ring_without_lunch: &DaySchedule = schedule_without_lunch
        .get(&weekday_num)
        .unwrap_or(&default_schedule_without_lunch);

    if !ring_with_lunch.contains_key(&n_lesson) && !ring_without_lunch.contains_key(&n_lesson) {
        return Err("Некорректный номер пары.".into());
    }

    let mut result_lines: Vec<String> = Vec::new();
    let is_saturday = weekday == Weekday::Sat; // Проверяем, является ли сегодня суббота

    let mut with_lunch_result: Option<String> = None;
    let mut without_lunch_result: Option<String> = None;

    // Проверяем расписание с обедом
    if let Some(&pair_time_with_lunch) = ring_with_lunch.get(&n_lesson) {
        match calculate_time_delta(pair_time_with_lunch, today).await {
            Ok((hours, minutes)) => {
                let message = format!(
                    "До начала {}й пары осталось: {} часов и {} минут.",
                    n_lesson, hours, minutes
                );
                if is_saturday {
                    // Убрать префикс в субботу
                    result_lines.push(message);
                } else {
                    with_lunch_result = Some(format!("С обедом: {}", message));
                }
            }
            Err(_) => {
                result_lines.push(format!("{} пара (с обедом) уже прошла или идёт:", n_lesson));
            }
        }
    }

    // Проверяем расписание без обеда
    if let Some(&pair_time_without_lunch) = ring_without_lunch.get(&n_lesson) {
        match calculate_time_delta(pair_time_without_lunch, today).await {
            Ok((hours, minutes)) => {
                let message = format!(
                    "До начала {}й пары осталось: {} часов и {} минут.",
                    n_lesson, hours, minutes
                );
                if is_saturday {
                    // Убрать префикс в субботу
                    if result_lines.is_empty() {
                        result_lines.push(message);
                    }
                } else {
                    without_lunch_result = Some(format!("Без обеда: {}", message));
                }
            }
            Err(_) => {
                result_lines.push(format!("{} пара (без обеда) уже прошла или идёт:", n_lesson));
            }
        }
    }

    // Объединяем результаты для обычных дней
    if !is_saturday {
        // Используем clone(), чтобы избежать перемещения
        if let (Some(with_lunch), Some(without_lunch)) = (with_lunch_result.clone(), without_lunch_result.clone()) {
            if with_lunch == without_lunch {
                result_lines.push(with_lunch); // Если совпадают, добавляем только один ответ
            } else {
                result_lines.push(with_lunch);
                result_lines.push(without_lunch);
            }
        } else if let Some(with_lunch) = with_lunch_result.clone() {
            result_lines.push(with_lunch);
        } else if let Some(without_lunch) = without_lunch_result.clone() {
            result_lines.push(without_lunch);
        }
    }

    if result_lines.is_empty() {
        return Err("Некорректный номер пары.".into());
    }

    Ok(result_lines.join("\\n"))
}

// API 2: Получение времени до следующей пары
pub async fn get_next_lesson() -> Result<Vec<String>, String> {
    let (schedule_with_lunch, default_schedule_with_lunch, schedule_without_lunch, default_schedule_without_lunch) = get_hardcoded_schedule();

    let today = Local::today();
    let weekday = today.weekday();
    let weekday_num = weekday.number_from_monday();

    let ring_with_lunch = schedule_with_lunch.get(&weekday_num).unwrap_or(&default_schedule_with_lunch);
    let ring_without_lunch = schedule_without_lunch.get(&weekday_num).unwrap_or(&default_schedule_without_lunch);

    let mut results: Vec<String> = Vec::new();
    let now = Local::now().time();

    // Function to find the next lesson
    fn find_next_lesson(ring: &DaySchedule, now: chrono::NaiveTime) -> Option<(u32, LessonTime)> {
        ring.iter()
            .filter(|(_, &lesson_time)| {
                NaiveTime::from_hms_opt(lesson_time.hour, lesson_time.minute, 0).map_or(false, |lesson_time_naive| {
                    now < lesson_time_naive
                })
            })
            .min_by_key(|&(lesson_num, lesson_time)| *lesson_num)
            .map(|(&lesson_num, &lesson_time)| (lesson_num, lesson_time))
    }

    if let Some((lesson_num, lesson_time)) = find_next_lesson(ring_with_lunch, now) {
        let prefix = if today.weekday() == Weekday::Sat {
            ""
        } else {
            "С обедом: "
        };
        match calculate_time_delta(lesson_time, today).await {
            Ok((hours, minutes)) => {
                results.push(format!(
                    "{prefix}До начала {}й пары осталось: {} часов и {} минут.",
                    lesson_num, hours, minutes, prefix = prefix
                ));
            }
            Err(err) => {
                results.push(format!("{prefix}{} пара уже прошла или идёт:", lesson_num, prefix = prefix));
            }
        }
    } else {
        // No more lessons today, find first lesson tomorrow
        let next_day = today.succ();
        let next_weekday_num = next_day.weekday().number_from_monday();
        let next_day_weekday = next_day.weekday();

        //Consider both with lunch and without lunch schedules
        let next_ring_with_lunch = schedule_with_lunch.get(&next_weekday_num).unwrap_or(&default_schedule_with_lunch);
        let next_ring_without_lunch = schedule_without_lunch.get(&next_weekday_num).unwrap_or(&default_schedule_without_lunch);

        //Find the earliest lesson in both schedules
        let first_with_lunch = next_ring_with_lunch
            .iter()
            .min_by_key(|&(lesson_num, _)| *lesson_num)
            .map(|(&lesson_num, &lesson_time)| (lesson_num, lesson_time, "с обедом"));
        let first_without_lunch = next_ring_without_lunch
            .iter()
            .min_by_key(|&(lesson_num, _)| *lesson_num)
            .map(|(&lesson_num, &lesson_time)| (lesson_num, lesson_time, "без обеда"));

        //Determine which schedule has the earliest lesson
        let next_lesson = match (first_with_lunch, first_without_lunch) {
            (Some((lesson_num_with, lesson_time_with, label_with)), Some((lesson_num_without, lesson_time_without, label_without))) => {
                if lesson_time_with < lesson_time_without {
                    Some((lesson_num_with, lesson_time_with, label_with))
                } else {
                    Some((lesson_num_without, lesson_time_without, label_without))
                }
            }
            (Some((lesson_num, lesson_time, label)), None) => Some((lesson_num, lesson_time, label)),
            (None, Some((lesson_num, lesson_time, label))) => Some((lesson_num, lesson_time, label)),
            (None, None) => None, //No lessons tomorrow either
        };

        if let Some((lesson_num, lesson_time, label)) = next_lesson {
            let prefix = if next_day_weekday == Weekday::Sat {
                ""
            } else {
                label
            };
            match calculate_time_delta(lesson_time, next_day).await {
                Ok((hours, minutes)) => {
                    results.push(format!(
                        "{prefix}До начала {}й пары осталось: {} часов и {} минут.",
                        lesson_num, hours, minutes, prefix = prefix
                    ));
                }
                Err(err) => {
                    results.push(format!("{prefix}{} пара уже прошла или идёт:", lesson_num, prefix = prefix));
                }
            }
        } else {
            return Err("На следующие дни расписание не найдено.".into());
        }
    }

    Ok(results)
}

// Функция обработки ввода номера пары пользователем
pub async fn w_lesson(user_input: &str) -> Result<String, String> {
    match user_input.parse::<u32>() {
        Ok(n_lesson) => {
            let time_delta = get_time_delta(n_lesson).await;
            time_delta
        }
        Err(_) => Err("Пожалуйста, введите корректное число.".into()),
    }
}
