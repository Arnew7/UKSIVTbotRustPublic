
use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, TimeZone, Weekday};
use std::collections::HashMap;

// Типы для расписания
type DaySchedule = HashMap<u32, LessonTime>;
type Schedule = HashMap<u32, DaySchedule>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LessonTime {
    hour: u32,
    minute: u32,
}

impl LessonTime {
    fn new(hour: u32, minute: u32) -> Self {
        LessonTime { hour, minute }
    }

    fn to_datetime(&self, date: chrono::Date<Local>) -> Option<DateTime<Local>> {
        let naive_time = NaiveTime::from_hms_opt(self.hour, self.minute, 0)?;
        Some(date.and_time(naive_time)?)
    }
}

fn get_hardcoded_schedule() -> (Schedule, DaySchedule, Schedule, DaySchedule) {
    let default_schedule_with_lunch = create_schedule(vec![
        (1, 7, 50), (2, 9, 30), (3, 11, 15), (4, 13, 35), (5, 15, 20), (6, 17, 0), (7, 18, 30)
    ]);

    let saturday_schedule = create_schedule(vec![
        (1, 8, 0), (2, 9, 30), (3, 11, 0), (4, 12, 30), (5, 14, 0), (6, 15, 30), (7, 17, 0)
    ]);

    let default_schedule_without_lunch = create_schedule(vec![
        (1, 7, 50), (2, 9, 30), (3, 11, 15), (4, 12, 50), (5, 14, 30), (6, 16, 10), (7, 17, 50)
    ]);

    let mut schedule_with_lunch = HashMap::new();
    schedule_with_lunch.insert(Weekday::Sat.number_from_monday(), saturday_schedule);

    (schedule_with_lunch, default_schedule_with_lunch, HashMap::new(), default_schedule_without_lunch)
}

fn create_schedule(times: Vec<(u32, u32, u32)>) -> DaySchedule {
    times.into_iter().map(|(n, h, m)| (n, LessonTime::new(h, m))).collect()
}

async fn calculate_time_delta(lesson_time: LessonTime, current_day: chrono::Date<Local>) -> Result<(u64, u64), String> {
    let lesson_datetime = lesson_time.to_datetime(current_day).ok_or("Недопустимое время урока")?;
    let now = Local::now();
    let duration: Duration = lesson_datetime.signed_duration_since(now);

    if duration.num_seconds() < 0 {
        return Err("Lesson already passed or is ongoing".into());
    }

    let hours = duration.num_hours() as u64;
    let minutes = (duration.num_minutes() % 60) as u64;

    Ok((hours, minutes))
}

fn find_next_lesson(ring: &DaySchedule, now: chrono::NaiveTime) -> Option<(u32, LessonTime)> {
    ring.iter()
        .filter(|(_, &lesson_time)| {
            NaiveTime::from_hms_opt(lesson_time.hour, lesson_time.minute, 0).map_or(false, |lesson_time_naive| {
                now <= lesson_time_naive
            })
        })
        .min_by_key(|&(lesson_num, _)| *lesson_num)
        .map(|(&lesson_num, &lesson_time)| (lesson_num, lesson_time))
}

pub async fn get_time_delta(n_lesson: u32) -> Result<String, String> {
    let (schedule_with_lunch, default_schedule_with_lunch, schedule_without_lunch, default_schedule_without_lunch) = get_hardcoded_schedule();

    let today = Local::today();
    let weekday_num = today.weekday().number_from_monday();

    if today.weekday() == Weekday::Sun {
        return Err("Сегодня воскресенье, пар нет.".into());
    }

    let ring_with_lunch = schedule_with_lunch.get(&weekday_num).unwrap_or(&default_schedule_with_lunch);
    let ring_without_lunch = schedule_without_lunch.get(&weekday_num).unwrap_or(&default_schedule_without_lunch);

    let mut results = vec![];

    for (schedule, schedule_type) in &[(ring_with_lunch, "С обедом"), (ring_without_lunch, "Без обеда")] {
        if let Some(&pair_time) = schedule.get(&n_lesson) {
            match calculate_time_delta(pair_time, today).await {
                Ok((hours, minutes)) => {
                    results.push(format!(
                        "{}: До начала {}-й пары осталось: {} часов и {} минут.",
                        schedule_type, n_lesson, hours, minutes
                    ));
                }
                Err(_) => {
                    results.push(format!(
                        "{}: {} пара уже прошла или идёт.",
                        schedule_type, n_lesson
                    ));
                }
            }
        }
    }

    if results.is_empty() {
        return Err("Некорректный номер пары.".into());
    }

    Ok(results.join("\n"))
}

pub async fn get_next_lesson() -> Result<Vec<String>, String> {
    let (schedule_with_lunch, default_schedule_with_lunch, schedule_without_lunch, default_schedule_without_lunch) = get_hardcoded_schedule();

    let today = Local::today();
    let mut current_day = today;
    let mut weekday_num = current_day.weekday().number_from_monday();

    // Если воскресенье, переходим на понедельник
    if current_day.weekday() == Weekday::Sun {
        current_day = today.succ(); // Переходим на понедельник
        weekday_num = Weekday::Mon.number_from_monday();
    }

    let ring_with_lunch = schedule_with_lunch.get(&weekday_num).unwrap_or(&default_schedule_with_lunch);
    let ring_without_lunch = schedule_without_lunch.get(&weekday_num).unwrap_or(&default_schedule_without_lunch);

    let now = Local::now().time();
    let mut results = vec![];

    // Поиск следующей пары для текущего дня
    let next_lesson_with_lunch = find_next_lesson(ring_with_lunch, now);
    let next_lesson_without_lunch = find_next_lesson(ring_without_lunch, now);

    match (next_lesson_with_lunch, next_lesson_without_lunch) {
        (Some((lesson_num_with, lesson_time_with)), Some((lesson_num_without, lesson_time_without))) => {
            if lesson_time_with == lesson_time_without {
                match calculate_time_delta(lesson_time_with, current_day).await {
                    Ok((hours, minutes)) => {
                        results.push(format!(
                            "До начала {}-й пары осталось: {} часов и {} минут.",
                            lesson_num_with, hours, minutes
                        ));
                    }
                    Err(e) => {
                        results.push(format!("{} пара уже прошла или идёт: {}", lesson_num_with, e));
                    }
                }
            } else {
                process_lesson_time("С обедом", lesson_num_with, lesson_time_with, current_day, &mut results).await;
                process_lesson_time("Без обеда", lesson_num_without, lesson_time_without, current_day, &mut results).await;
            }
        }
        (Some((lesson_num, lesson_time)), None) => {
            process_lesson_time("С обедом", lesson_num, lesson_time, current_day, &mut results).await;
        }
        (None, Some((lesson_num, lesson_time))) => {
            process_lesson_time("Без обеда", lesson_num, lesson_time, current_day, &mut results).await;
        }
        (None, None) => {
            // Если уроков больше нет, ищем первый урок следующего дня
            let next_day = find_next_working_day(current_day);
            let next_weekday_num = next_day.weekday().number_from_monday();

            let next_ring_with_lunch = schedule_with_lunch.get(&next_weekday_num).unwrap_or(&default_schedule_with_lunch);
            let next_ring_without_lunch = schedule_without_lunch.get(&next_weekday_num).unwrap_or(&default_schedule_without_lunch);

            let mut next_day_results = vec![];
            find_first_lesson(next_day, next_ring_with_lunch, "с обедом", &mut next_day_results).await;
            find_first_lesson(next_day, next_ring_without_lunch, "без обеда", &mut next_day_results).await;

            if let (Some(res1), Some(res2)) = (next_day_results.get(0), next_day_results.get(1)) {
                if res1 == res2 {
                    results.push(res1.clone());
                } else {
                    results.extend(next_day_results);
                }
            } else {
                results.extend(next_day_results);
            }
        }
    }

    if results.is_empty() {
        results.push("Уроков на сегодня больше нет".to_string());
    }
    Ok(results)
}

async fn process_lesson_time(
    schedule_type: &str,
    lesson_num: u32,
    lesson_time: LessonTime,
    current_day: chrono::Date<Local>,
    results: &mut Vec<String>,
) {
    match calculate_time_delta(lesson_time, current_day).await {
        Ok((hours, minutes)) => {
            results.push(format!(
                "{}: До начала {}-й пары осталось: {} часов и {} минут.",
                schedule_type, lesson_num, hours, minutes
            ));
        }
        Err(e) => {
            results.push(format!("{}: {} пара уже прошла или идёт: {}", schedule_type, lesson_num, e));
        }
    }
}

fn find_next_working_day(current_day: chrono::Date<Local>) -> chrono::Date<Local> {
    let mut next_day = current_day.succ();
    while next_day.weekday() == Weekday::Sun {
        next_day = next_day.succ();
    }
    next_day
}

async fn find_first_lesson(
    next_day: chrono::Date<Local>,
    ring: &DaySchedule,
    schedule_type: &str,
    results: &mut Vec<String>,
) {
    if let Some((&lesson_num, &lesson_time)) = ring.iter().min_by_key(|&(lesson_num, _)| lesson_num) {
        let now = Local::now();
        let next_lesson_datetime = lesson_time.to_datetime(next_day).unwrap();
        let duration = next_lesson_datetime.signed_duration_since(now);
        let hours = duration.num_hours();
        let minutes = (duration.num_minutes() % 60).abs();

        results.push(format!(
            "Завтра: Первая пара {} через {} ч {} мин.",
            lesson_num, hours, minutes
        ));
    }
}
