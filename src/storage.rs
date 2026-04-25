use std::{
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use directories::ProjectDirs;

use crate::task::Task;

const APP_DIR_NAME: &str = "rtasks";
const TASKS_FILE_NAME: &str = "tasks.jsonl";
const TASKS_TMP_FILE_NAME: &str = "tasks.tmp";

pub fn tasks_file_path() -> Result<PathBuf> {
    Ok(data_dir()?.join(TASKS_FILE_NAME))
}

pub fn load_tasks() -> Result<Vec<Task>> {
    load_tasks_from_path(&tasks_file_path()?)
}

pub fn save_tasks(tasks: &[Task]) -> Result<()> {
    save_tasks_to_path(&tasks_file_path()?, tasks)
}

pub fn add_task(tasks: &mut Vec<Task>, task: Task) -> Result<()> {
    tasks.push(task);
    save_tasks(tasks)
}

pub fn update_task(tasks: &mut [Task], updated: Task) -> Result<()> {
    let task = tasks
        .iter_mut()
        .find(|task| task.id == updated.id)
        .with_context(|| format!("task not found: {}", updated.id))?;
    *task = updated;
    save_tasks(tasks)
}

pub fn delete_task(tasks: &mut Vec<Task>, id: &str) -> Result<()> {
    let original_len = tasks.len();
    tasks.retain(|task| task.id != id);

    if tasks.len() == original_len {
        anyhow::bail!("task not found: {id}");
    }

    save_tasks(tasks)
}

pub fn load_tasks_from_path(path: &Path) -> Result<Vec<Task>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut tasks = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("failed to read line {}", index + 1))?;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let task = serde_json::from_str::<Task>(trimmed)
            .with_context(|| format!("invalid task JSON at line {}", index + 1))?;
        tasks.push(task);
    }

    Ok(tasks)
}

pub fn save_tasks_to_path(path: &Path, tasks: &[Task]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let tmp_path = path.with_file_name(TASKS_TMP_FILE_NAME);
    {
        let mut file = File::create(&tmp_path)
            .with_context(|| format!("failed to create {}", tmp_path.display()))?;

        for task in tasks {
            serde_json::to_writer(&mut file, task).context("failed to serialize task")?;
            file.write_all(b"\n").context("failed to write newline")?;
        }

        file.sync_all()
            .with_context(|| format!("failed to sync {}", tmp_path.display()))?;
    }

    fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "failed to replace {} with {}",
            path.display(),
            tmp_path.display()
        )
    })?;

    Ok(())
}

fn data_dir() -> Result<PathBuf> {
    if let Some(appdata) = env::var_os("APPDATA") {
        return Ok(PathBuf::from(appdata).join(APP_DIR_NAME));
    }

    let project_dirs = ProjectDirs::from("", "", APP_DIR_NAME)
        .context("failed to resolve application data directory")?;
    Ok(project_dirs.data_dir().to_path_buf())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    use super::*;
    use crate::task::{Priority, TaskSource, TaskStatus};

    #[test]
    fn save_and_load_tasks_roundtrip() {
        let path = unique_temp_path();
        let tasks = vec![sample_task("01", "comprar pan")];

        save_tasks_to_path(&path, &tasks).expect("save tasks");
        let loaded = load_tasks_from_path(&path).expect("load tasks");

        assert_eq!(loaded, tasks);
        let contents = fs::read_to_string(&path).expect("read tasks file");
        assert_eq!(contents.lines().count(), 1);
    }

    #[test]
    fn load_tasks_ignores_empty_lines() {
        let path = unique_temp_path();
        let task = sample_task("02", "reparar STL viewer");
        let encoded = serde_json::to_string(&task).expect("serialize task");
        fs::write(&path, format!("\n{encoded}\n\n")).expect("write tasks file");

        let loaded = load_tasks_from_path(&path).expect("load tasks");

        assert_eq!(loaded, vec![task]);
    }

    fn sample_task(id: &str, title: &str) -> Task {
        Task {
            id: id.to_owned(),
            title: title.to_owned(),
            status: Some(TaskStatus::Todo),
            priority: Some(Priority::Medium),
            due: None,
            created_at: "2026-04-25T18:30:00-03:00".to_owned(),
            updated_at: "2026-04-25T18:30:00-03:00".to_owned(),
            completed_at: None,
            source: TaskSource::DesktopQuickAdd,
        }
    }

    fn unique_temp_path() -> PathBuf {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let dir = env::temp_dir().join(format!("rtasks-test-{}-{id}", std::process::id()));
        fs::create_dir_all(&dir).expect("create temp task dir");
        dir.join(TASKS_FILE_NAME)
    }
}
