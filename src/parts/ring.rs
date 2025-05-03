
use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, TimeZone, Weekday};
use std::collections::HashMap;

// Константы для дней недели
const MONDAY: u32 = 1;
const TUESDAY: u32 = 2;
const WEDNESDAY: u32 = 3;
const THURSDAY: u32 = 4;
const FRIDAY: u32 = 5;
const SATURDAY: u32 = 6;
const SUNDAY: u32 = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LessonTime {
    hour: u32,
    minute: u32,
}

impl LessonTime {
    fn new(hour: u32, minute: u32) -> Self {
        // Можно добавить валидацию значений hour и minute здесь
        LessonTime { hour, minute }
    }
}

// Функция для преобразования LessonTime в DateTime<Local>
fn lesson_time_to_datetime(lesson_time: LessonTime, today: chrono::Date<Local>) -> Option<DateTime<Local>> {
    let naive_time = NaiveTime::from_hms_opt(lesson_time.hour, lesson_time.minute, 0)?;
    Some(today.and_time(naive_time)?)
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

    // Расписание для субботы (weekday 6)
    let mut saturday_schedule: DaySchedule = HashMap::new();
    saturday_schedule.insert(1, LessonTime::new(8, 0));
    saturday_schedule.insert(2, LessonTime::new(9, 30));
    saturday_schedule.insert(3, LessonTime::new(11, 0));
    saturday_schedule.insert(4, LessonTime::new(12, 30));
    saturday_schedule.insert(5, LessonTime::new(14, 0));
    saturday_schedule.insert(6, LessonTime::new(15, 30));
    saturday_schedule.insert(7, LessonTime::new(17, 0));
    schedule_with_lunch.insert(SATURDAY, saturday_schedule); // Subbota is 6.

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

async fn calculate_time_delta(lesson_time: LessonTime, current_day: chrono::Date<Local>) -> Result<(u64, u64), String> {
    let lesson_datetime = match lesson_time_to_datetime(lesson_time, current_day) {
        Some(dt) => dt,
        None => return Err("Недопустимое время урока".into()),
    };

    // Calculate the duration until the lesson
    let now = Local::now();
    let duration: Duration = lesson_datetime.signed_duration_since(now);

    if duration.num_seconds() < 0 {
        return Err("Lesson already passed or is ongoing".into());
    }

    let hours = duration.num_hours() as u64;
    let minutes = (duration.num_minutes() % 60) as u64;

    Ok((hours, minutes))
}

// API 1: Получение времени до указанной пары
pub async fn get_time_delta(n_lesson: u32) -> Result<String, String> {
    let (schedule_with_lunch, default_schedule_with_lunch, schedule_without_lunch, default_schedule_without_lunch) = get_hardcoded_schedule();

    let today = Local::today();
    let weekday = today.weekday();
    let weekday_num = weekday.number_from_monday();
    let is_saturday = weekday == Weekday::Sat;

    // Обработка воскресенья
    if weekday == Weekday::Sun{

        return Err("Сегодня воскресенье, пар нет.".into());
    }

    let ring_with_lunch: &DaySchedule = if is_saturday {
        schedule_with_lunch
            .get(&weekday_num)
            .unwrap_or(&default_schedule_with_lunch) // Используем расписание с обедом для субботы
    } else {
        schedule_with_lunch
            .get(&weekday_num)
            .unwrap_or(&default_schedule_with_lunch)
    };

    let ring_without_lunch: &DaySchedule =  if is_saturday {
        &default_schedule_without_lunch // Используем расписание без обеда для субботы
    } else {
        schedule_without_lunch
            .get(&weekday_num)
            .unwrap_or(&default_schedule_without_lunch)
    };



    if !ring_with_lunch.contains_key(&n_lesson) && !ring_without_lunch.contains_key(&n_lesson) {
        return Err("Некорректный номер пары.".into());
    }

    let mut result_lines: Vec<String> = Vec::new();

    // Helper function
    async fn process_schedule(
        n_lesson: u32,
        schedule: &DaySchedule,
        schedule_type: &str,
        today: chrono::Date<Local>,
    ) -> Option<String> {
        if let Some(&pair_time) = schedule.get(&n_lesson) {
            match calculate_time_delta(pair_time, today).await {
                Ok((hours, minutes)) => {
                    let message = format!(
                        "До начала {}-й пары осталось: {} часов и {} минут.",
                        n_lesson, hours, minutes
                    );

                    Some(format!("{}:  {}", schedule_type, message))

                }
                Err(_) => {
                    Some(format!("{} пара ({}) уже прошла или идёт:", n_lesson, schedule_type))
                }
            }
        } else {
            None
        }
    }

    let with_lunch_result = process_schedule(n_lesson, ring_with_lunch, "С обедом", today).await;
    let without_lunch_result = process_schedule(n_lesson, ring_without_lunch, "Без обеда", today).await;


    if is_saturday {
        if let Some(result) = with_lunch_result {
            result_lines.push(result);
        }
    } else {
        match (with_lunch_result, without_lunch_result) {
            (Some(with_lunch), Some(without_lunch)) => {
                if with_lunch == without_lunch {
                    result_lines.push(with_lunch);
                } else {
                    result_lines.push(with_lunch);
                    result_lines.push(without_lunch);
                }
            }
            (Some(result), None) | (None, Some(result)) => {
                result_lines.push(result);
            }
            (None, None) => (), // No lesson found
        }
    }

    if result_lines.is_empty() {
        return Err("Некорректный номер пары.".into());
    }

    Ok(result_lines.join("\n"))
}

pub async fn get_next_lesson() -> Result<Vec<String>, String> {
    let (schedule_with_lunch, default_schedule_with_lunch, schedule_without_lunch, default_schedule_without_lunch) = get_hardcoded_schedule();

    let today = Local::today();
    let mut weekday = today.weekday();
    let mut weekday_num = weekday.number_from_monday();
    let mut current_day = today;

    // Проверяем, не воскресенье ли сегодня. Если да, переходим на понедельник
    if weekday_num == 7 {
        weekday_num = 1;
        weekday = Weekday::Mon;
        current_day = today.succ(); // Переходим на завтра (понедельник)
    }

    let ring_with_lunch = schedule_with_lunch.get(&weekday_num).unwrap_or(&default_schedule_with_lunch);
    let ring_without_lunch = schedule_without_lunch.get(&weekday_num).unwrap_or(&default_schedule_without_lunch);

    let mut results: Vec<String> = Vec::new();
    let now = Local::now().time();

    // Function to find the next lesson
    fn find_next_lesson(ring: &DaySchedule, now: chrono::NaiveTime) -> Option<(u32, LessonTime)> {
        ring.iter()
            .filter(|(_, &lesson_time)| {
                NaiveTime::from_hms_opt(lesson_time.hour, lesson_time.minute, 0).map_or(false, |lesson_time_naive| {
                    now <= lesson_time_naive // Use <= instead of <
                })
            })
            .min_by_key(|&(lesson_num, _)| *lesson_num)
            .map(|(&lesson_num, &lesson_time)| (lesson_num, lesson_time))
    }

    // Находим следующий урок в обоих расписаниях
    let next_lesson_with_lunch = find_next_lesson(ring_with_lunch, now);
    let next_lesson_without_lunch = find_next_lesson(ring_without_lunch, now);

    // Обрабатываем уроки
    match (next_lesson_with_lunch, next_lesson_without_lunch) {
        (Some((lesson_num_with, lesson_time_with)), Some((lesson_num_without, lesson_time_without))) => {
            if lesson_time_with == lesson_time_without {
                // Если время одинаковое, выводим одно сообщение без префикса
                match calculate_time_delta(lesson_time_with, current_day).await {
                    Ok((hours, minutes)) => {
                        results.push(format!(
                            "До начала {}й пары осталось: {} часов и {} минут.",
                            lesson_num_with, hours, minutes
                        ));
                    }
                    Err(e) => {
                        results.push(format!("{} пара уже прошла или идёт: {}", lesson_num_with, e));
                    }
                }
            } else {
                // Если время разное, выводим сообщения для каждого урока отдельно
                match calculate_time_delta(lesson_time_with, current_day).await {
                    Ok((hours, minutes)) => {
                        results.push(format!(
                            "С обедом: До начала {}й пары осталось: {} часов и {} минут.",
                            lesson_num_with, hours, minutes
                        ));
                    }
                    Err(e) => {
                        results.push(format!("С обедом: {} пара уже прошла или идёт: {}", lesson_num_with, e));
                    }
                }

                match calculate_time_delta(lesson_time_without, current_day).await {
                    Ok((hours, minutes)) => {
                        results.push(format!(
                            "Без обеда: До начала {}й пары осталось: {} часов и {} минут.",
                            lesson_num_without, hours, minutes
                        ));
                    }
                    Err(e) => {
                        results.push(format!("Без обеда: {} пара уже прошла или идёт: {}", lesson_num_without, e));
                    }
                }
            }
        }
        (Some((lesson_num, lesson_time)), None) => {
            match calculate_time_delta(lesson_time, current_day).await {
                Ok((hours, minutes)) => {
                    results.push(format!(
                        "С обедом: До начала {}й пары осталось: {} часов и {} минут.",
                        lesson_num, hours, minutes
                    ));
                }
                Err(e) => {
                    results.push(format!("С обедом: {} пара уже прошла или идёт: {}", lesson_num, e));
                }
            }
        }
        (None, Some((lesson_num, lesson_time))) => {
            match calculate_time_delta(lesson_time, current_day).await {
                Ok((hours, minutes)) => {
                    results.push(format!(
                        "Без обеда: До начала {}й пары осталось: {} часов и {} минут.",
                        lesson_num, hours, minutes
                    ));
                }
                Err(e) => {
                    results.push(format!("Без обеда: {} пара уже прошла или идёт: {}", lesson_num, e));
                }
            }
        }
        (None, None) => { // Если уроков сегодня больше нет, ищем первый урок на следующий день
            let mut next_day = current_day.succ();
            let mut next_weekday_num = next_day.weekday().number_from_monday();

            // Если завтра воскресенье, переходим на понедельник
            if next_weekday_num == 7 {
                next_day = next_day.succ();
                next_weekday_num = next_day.weekday().number_from_monday();
            }

            let next_ring_with_lunch = schedule_with_lunch.get(&next_weekday_num).unwrap_or(&default_schedule_with_lunch);
            let next_ring_without_lunch = schedule_without_lunch.get(&next_weekday_num).unwrap_or(&default_schedule_without_lunch);

            let first_with_lunch = next_ring_with_lunch
                .iter()
                .min_by_key(|&(lesson_num, _)| *lesson_num)
                .map(|(&lesson_num, &lesson_time)| (lesson_num, lesson_time, "c обедом"));

            let first_without_lunch = next_ring_without_lunch
                .iter()
                .min_by_key(|&(lesson_num, _)| *lesson_num)
                .map(|(&lesson_num, &lesson_time)| (lesson_num, lesson_time, "без обеда"));

            // Сравниваем первый урок в обоих расписаниях и выбираем ближайший
            match (first_with_lunch, first_without_lunch) {
                (Some((lesson_num_lunch, lesson_time_lunch, _)), Some((lesson_num_no_lunch, lesson_time_no_lunch, _))) => {
                    // Сравниваем времена начала уроков и выбираем ближайший

                    //Создаем DateTime для времени начала урока на следующий день
                    let next_day_lunch_datetime = lesson_time_to_datetime(lesson_time_lunch, next_day)
                        .ok_or("Invalid lesson time".to_string())?;
                    let next_day_no_lunch_datetime = lesson_time_to_datetime(lesson_time_no_lunch, next_day)
                        .ok_or("Invalid lesson time".to_string())?;
                    let now = Local::now(); // Получаем текущее DateTime

                    let duration_lunch = next_day_lunch_datetime.signed_duration_since(now);
                    let duration_no_lunch = next_day_no_lunch_datetime.signed_duration_since(now);

                    if duration_lunch <= duration_no_lunch {
                        let hours = duration_lunch.num_hours();
                        let minutes = (duration_lunch.num_minutes() % 60).abs();
                        results.push(format!(
                            "Завтра (с обедом): Первая пара {} через {} ч {} мин.",
                            lesson_num_lunch, hours, minutes
                        ));
                    } else {
                        let hours = duration_no_lunch.num_hours();
                        let minutes = (duration_no_lunch.num_minutes() % 60).abs();

                        results.push(format!(
                            "Завтра (без обеда): Первая пара {} через {} ч {} мин.",
                            lesson_num_no_lunch, hours, minutes
                        ));
                    }
                }
                (Some((lesson_num, lesson_time, _)), None) => {
                    let next_day_datetime = lesson_time_to_datetime(lesson_time, next_day)
                        .ok_or("Invalid lesson time".to_string())?;
                    let now = Local::now();

                    let duration = next_day_datetime.signed_duration_since(now);
                    let hours = duration.num_hours();
                    let minutes = (duration.num_minutes() % 60).abs();
                    results.push(format!(
                        "Завтра (с обедом): Первая пара {} через {} ч {} мин.",
                        lesson_num,  hours, minutes
                    ));
                }
                (None, Some((lesson_num, lesson_time, _))) => {
                    let next_day_datetime = lesson_time_to_datetime(lesson_time, next_day)
                        .ok_or("Invalid lesson time".to_string())?;
                    let now = Local::now();

                    let duration = next_day_datetime.signed_duration_since(now);
                    let hours = duration.num_hours();
                    let minutes = (duration.num_minutes() % 60).abs();

                    results.push(format!(
                        "Завтра (без обеда): Первая пара {} через {} ч {} мин.",
                        lesson_num,  hours, minutes
                    ))
                }
                (None, None) => results.push("Завтра уроков нет.".to_string()),
            }
        }
    }

    if results.is_empty() {
        results.push("Уроков на сегодня больше нет".to_string());
    }

    Ok(results)
}
