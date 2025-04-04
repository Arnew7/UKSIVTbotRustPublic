use super::memcached;

use reqwest::{Client};
use scraper::{Html, Selector};
use tokio;
use anyhow::{Result, Error, anyhow, Context};
use std::borrow::Cow;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use tokio::task::JoinHandle;
use futures::future::join_all;
use std::path::{Path, PathBuf};
use std::sync::{Arc};
use std::io::{self};
use regex::Regex;
use memcached::write_on_memcached;
use pdf_extract;
use std::fs;
use crate::parts::time::{after_tomorrow, today, tomorrow};

pub(crate) async fn replacements_main() -> Result<()> {
    //Все группы колледжа
    let groups_vec :Vec<String>  = vec![
        "24укск-1", "24укск-2", "24са-1", "24са-2", "24п-1", "24п-2", "24п-3", "24п-4", "24п-5", "24веб-1", "24веб-2",
        "24иис-1", "24оиб-1", "24оиб-2", "24з-1", "24з-2", "24э-1", "24э-2", "24ю-1", "24ю-2", "24ю-3", "24ю-4", "24пд-1",
        "24пд-2", "24л-1", "24л-2", "24ул-1",

        "23по-2", "23веб-1", "23веб-2", "23з-1", "23з-2", "23л-1", "23л-2", "23оиб-1", "23оиб-2",
        "23п-1", "23п-2", "23п-3", "23п-4", "23п-5", "23п-6", "23пд-1", "23пд-2", "23по-1", "23по-2",
        "23по-3", "23по-4", "23по-5", "23са-1", "23са-2", "23са-3", "23укск-1", "23ул-1", "23э-1",
        "23э-2",

        "22бд-1", "22веб-1", "22веб-2", "22зио-1", "22зио-2", "22ис-1", "22л-1", "22л-2",
        "22оиб-1", "22оиб-2", "22п-1", "22п-2", "22п-3", "22пд-1", "22пд-2", "22по-1", "22по-2",
        "22по-3", "22пса-1", "22пса-2", "22пса-3", "22са-1", "22са-2", "22укск-1", "22укск-2",
        "22э-1", "22э-2",

        "21бд-1", "21веб-1", "21веб-2", "21зио-1", "21зио-2", "21зио-3",
        "21ис-1", "21л-1", "21л-2", "21оиб-1", "21оиб-2", "21оиб-3", "21п-1", "21п-2", "21п-3",
        "21пд-1", "21пд-1", "21пд-2", "21пд-2", "21пд-3", "21пд-3",  "21по-1", "21по-1",
        "21по-2", "21по-2", "21по-3", "21по-4",
        "21пса-1", "21пса-2", "21пса-3", "21пса-4", "21пса-6",
        "21са-1",
        "21са-2", "21са-2",
        "21укск-1",
        "21э-1",
        "21э-2",

    ].iter().map(|s| s.to_string()).collect();
    let groups_arc_vector = Arc::new(groups_vec);


    let day = today().await.to_string();

    let day_y = tomorrow().await.to_string();
    let day_ay = after_tomorrow().await.to_string();

    //let month_n =

    let url = "https://uksivt.ru/замены/";

    let arr_url: Vec<Cow<str>> = scrape_page(url).await?;
    let dates = vec![day, day_y, day_ay];


    let r_links = process_search(arr_url, dates).await?;

    let file_paths = process_download(r_links).await?;
    let vec_texts = read_and_process_files(&file_paths).await?;

    let vec_texts_arc = Arc::new(vec_texts);
    let file_paths_arc = Arc::new(file_paths);

    let mut handles = Vec::new();

    for group in groups_arc_vector.iter() {

        let vec_texts_clone = Arc::clone(&vec_texts_arc);
        let file_paths_clone = Arc::clone(&file_paths_arc);
        let groups_arc_vector_clone = Arc::clone(&groups_arc_vector);
        let group_clone = group.clone();

        let handle: JoinHandle<Result<(), String>> = tokio::spawn(async move {

            let mut processed_text = finished(
                &vec_texts_clone,
                &file_paths_clone,
                &groups_arc_vector_clone,
                &group_clone,
            )
                .await;
            processed_text.insert_str(0, "\n");
            processed_text.insert_str(0, &group_clone);

            write_on_memcached(processed_text, group_clone)
                .await
                .map_err(|e| e.to_string())?; // Convert error


            Ok(())
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    //let file_paths: Vec<&Path> = file_paths_arc.iter().map(|path_buf| path_buf.as_path()).collect();
    //delete_files(&file_paths).await?;
    Ok(())
}


async fn finished(vec_texts: &Vec<String>, file_paths: &Vec<PathBuf>, groups_vec: &Vec<String>, group:&String) -> String {
    let mut combined_text = String::new();
    for (i, text) in vec_texts.iter().enumerate() {
        let file_path = PathBuf::from(&file_paths[i]);
        if text.is_empty() {
            eprintln!("text {}: ошибка при обработке", file_path.display());
            continue; // Пропускаем пустые строки (ошибки чтения) и не добавляем в combined_text
        }

        let date = extract_date(text);

        // Обработка текста
        let mut replaced = process_text(text);
        replaced = replaced.to_lowercase();


        replaced = remove_l(&replaced, group);
        replaced = indexing(&replaced, &groups_vec);

        let last_check_result = last_check(&replaced);
        let mut processed_text = replacements(&replaced);

        if !last_check_result.is_empty() {
            processed_text = last_check_result
        }


        if let Some(date_val) = date {
            let mut date_string = date_val.to_string();
            date_string.push_str("\n");
            processed_text.insert_str(0, &date_string);
        }


        combined_text.push_str(&processed_text);
        combined_text.push_str("\n"); // Добавляем разделитель между текстами
    }
    combined_text
}



async fn find_link(link: Cow<'static, str>, date: Arc<String>) -> Result<Option<String>> {
    if link.contains(&*date) {
        Ok(Some(link.to_string()))
    } else {
        Ok(None)
    }
}

async fn search_link(arr_link: &[Cow<'static, str>], date: &str) -> Result<String> {
    let mut tasks = Vec::new();
    let shared_date = Arc::new(date.to_string());

    for link in arr_link {
        let owned_link = link.clone();
        tasks.push(tokio::spawn(find_link(owned_link, shared_date.clone())));
    }

    let mut results = Vec::new();
    for task in tasks {
        let res = task.await??;
        if let Some(link) = res {
            results.push(link);
        }
    }

    for result in results {
        //%20 заменяет пробел
        let url = format!("https://www.uksivt.ru/{}", result[18..].replace(" ", "%20"));
        return Ok(url);
    }
    Err(Error::msg("Ссылка не найдена"))
}


async fn scrape_page(url: &str) -> Result<Vec<Cow<'static, str>>> {
    let client = Client::new();
    let resp = client.get(url).send().await.expect("Connect to URL 'UKSIVT' has been failed");
    let html = resp.text().await?;

    let fragment = Html::parse_document(&html);
    let selector = match Selector::parse("a") {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    let links: Vec<Cow<'static, str>> = fragment.select(&selector)
        .filter_map(|a| a.value().attr("href").map(|s| s.to_string().into()))
        .collect();

    Ok(links)
}
async fn process_search(arr_url: Vec<Cow<'static, str>>, dates: Vec<String>) -> Result<Vec<String>> {
    let arr_url = Arc::new(arr_url); // Оборачиваем в Arc
    let mut tasks: Vec<JoinHandle<(String, Result<String>)>> = Vec::new();

    for date in dates {
        let arr_url_clone: Arc<Vec<Cow<str>>> = Arc::clone(&arr_url); // Клонируем Arc
        tasks.push(tokio::spawn(async move {
            let result = search_link(&arr_url_clone, &date).await;
            (date, result)
        }));
    }


    let results = join_all(tasks).await;
    let mut links = Vec::new();
    for result in results {
        let (date, link_result) = result.expect("Ошибка при получения нужного url");
        match link_result {
            Ok(link) => {
                println!("Для даты {} найдена ссылка: {}", date, link);
                links.push(link);
            }
            Err(e) => println!("Для даты {} не найдена ссылка: {}", date, e),
        }
    }
    Ok(links)
}


async fn download_file(url: &str, filepath: &Path) -> Result<()> {


    let client = Client::new();
    let mut resp = client.get(url).send().await?;

    let mut file = File::create(filepath).await?;

    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk).await?;
    }
    Ok(())
}

async fn process_download(links: Vec<String>) -> Result<Vec<PathBuf>> {
    let mut download_tasks: Vec<JoinHandle<std::result::Result<Vec<PathBuf>, Error>>> = Vec::new();

    for url in links {
        let url_clone = url.clone();

        download_tasks.push(tokio::spawn(async move {
            let file_name: String = url_clone.split("/").last().unwrap_or("default.pdf").replace("%20", "_");
            let file_path: PathBuf = Path::new("downloaded_files").join(file_name);

            if !Path::new("downloaded_files").exists() {
                tokio::fs::create_dir_all("downloaded_files").await?;
            }

            match download_file(&url_clone, &file_path).await {
                Ok(_) => Ok(vec![file_path]),
                Err(e) => {
                    println!("Ошибка загрузки файла: {} из {} ", url_clone, e);
                    Err(anyhow!("Ошибка загрузки файла"))
                }
            }
        }));
    }

    let mut all_file_paths: Vec<PathBuf> = Vec::new();
    for task_result in join_all(download_tasks).await {
        match task_result {
            Ok(Ok(file_paths)) => all_file_paths.extend(file_paths), // Добавляем пути в общий вектор
            Ok(Err(e)) => println!("Ошибка в задаче: {}", e),
            Err(e) => println!("Ошибка при join: {}", e),
        }
    }

    Ok(all_file_paths)
}



async fn read_and_process_file(path: &PathBuf) -> Result<String, io::Error> {
    let file_name: String = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let extension: &str = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    let result = match extension {
        "pdf" => {
            match process_pdf(path).await {
                Ok(text) => format!("PDF {}: {}", file_name, text),
                Err(e) => {
                    eprintln!("Ошибка при обработке PDF файла: {}{}", path.display(), e);
                    format!("PDF {}: Ошибка при обработке in FN read_and_process_file", file_name)
                }
            }
        }
        "docx" => {
            "Присутствует docx файл, Его обработка появиться в скором времени)".to_string()
        }
        _ => {
            match process_text_file(path).await{
                Ok(text) => format!("Text {}: {}",file_name, text),
                Err(_) =>{
                    eprintln!("Ошибка при обработке текстового файла: {}", path.display());
                    format!("Text {}: Ошибка при обработке", file_name)
                }
            }
        }
    };
    Ok(result)
}
async fn read_and_process_files(file_paths: &Vec<PathBuf>) -> Result<Vec<String>, io::Error> {
    let futures: Vec<_> = file_paths
        .iter()
        .map(read_and_process_file)
        .collect();
    let results = join_all(futures).await;
    let mut texts: Vec<String> = Vec::new();
    for result in results {
        match result {
            Ok(output) => texts.push(output),
            Err(e) => eprintln!("Ошибка при обработке файла: {:?}", e),
        }
    }
    Ok(texts)

}

async fn process_pdf(path: &Path) -> Result<String> {
    let mut file: File = File::open(path).await?;
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).await?;
    match pdf_extract::extract_text_from_mem(&buffer) {
        Ok(text) => Ok(text),
        Err(e) =>  Err(anyhow!("Ошибка загрузки файла{}", e))
    }
}

async fn process_text_file(path: &Path) -> Result<String, io::Error> {
    let mut file: File = File::open(path).await?;
    let mut text: String = String::new();
    file.read_to_string(&mut text).await?;
    Ok(text)
}


fn last_check (text: &String) -> String {
    let keywords = ["зам.директора", "корректировка", "замена"];
    for line in text.lines(){
        let trimmed_line = line.trim();

        if trimmed_line.is_empty() {
            continue;
        }

        for keyword in &keywords {
            if trimmed_line.contains(keyword) {
                return "отсутствуют".to_string();
            }
        }
    }
    "".to_string()
}


fn process_text(text: &str) -> String {
    let pattern = Regex::new(r"(21|22|23|24|25)\s*([а-яА-Я]+|[-–]\d+)\s*-?\s*(\d)").unwrap();
    let replacement = "$1$2-$3";

    let lines: Vec<&str> = text.split('\n').collect();
    let mut processed_lines: Vec<String> = Vec::new();


    for line in lines {
        let replaced_line = pattern.replace_all(line, replacement).to_string();
        processed_lines.push(replaced_line);
    }

    processed_lines.join("\n")
}


fn extract_date(text: &str) -> Option<String> {
    let re = Regex::new(r"НА (\d{1,2} [А-Яа-я]+(?: – [А-Яа-я]+)?)").unwrap();
    re.captures(text)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}




fn remove_l(text: &str, group: &str) -> String {
    if let Some(index) = text.find(group) {
        text[index..].replace(group, "")
    } else {
        text.to_string()
    }
}

fn indexing(text: &str, groups: &Vec<String>) -> String {
    let min_index = groups.iter()
        .filter_map(|group| {
            text.find(group).or_else(|| {
                group.find('-')
                    .and_then(|dash_index| text.find(&group[..dash_index]))
            })
        })
        .min()
        ;

    match min_index {
        Some(index) => text[..index].to_lowercase(),
        None => text.to_lowercase()
    }
}




fn replacements(text: &str) -> String {
    const ABSENT_STRING: &str = "отсутствуют";
    const PRACTICE_STRING: &str = "-на практике";


    if text.len() <= 25 || text.contains(PRACTICE_STRING) {
        ABSENT_STRING.to_string()
    } else {
        text.split('\n')
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join("\n")
    }
}



async fn delete_file(path: &Path) -> Result<()> {
    if !path.exists() {
        println!("Файл {} не существует", path.display());
        return Ok(()); // Файл не существует, ничего не делаем
    }

    if !path.is_file() {
        eprintln!("{} не является файлом!", path.display());
        return Ok(());
    }
    fs::remove_file(path)
        .with_context(|| format!("Ошибка при удалении файла {}", path.display()))?;
    println!("Файл {} успешно удален", path.display());

    Ok(())
}

async fn delete_files(paths: &Vec<&Path>) -> Result<()>{
    for path in paths {
        delete_file(path).await?;
    }
    Ok(())
}
