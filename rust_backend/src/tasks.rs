use std::path::Path;

use crate::docker;

pub enum TaskTypes {
    CreateImage(String),
    StartContainer(String),
    RemoveContainer(String),
    CreateContainer(String),
}

pub struct Task {
    pub(crate) task_type: TaskTypes,
    pub dependencies: Vec<Task>,
}

impl Task {
    pub fn new(task_type: TaskTypes) -> Self {
        Self {
            task_type: task_type,
            dependencies: vec![],
        }
    }
    pub async fn run_task(&self) {
        match &self.task_type {
            TaskTypes::CreateImage(s) => self.create_image(s).await,
            TaskTypes::StartContainer(s) => self.start_container(s).await,
            _ => todo!("Unimplemented task"),
        }
    }
    async fn create_image(&self, file_name: &str) {
        let file_path: &Path = Path::new(file_name);
        docker::create_image(file_path)
            .await
            .expect("Failed to create image");
    }
    async fn start_container(&self, tag: &str) {
        docker::start_container(tag)
            .await
            .expect("failed to start container");
    }
}

pub struct Queue<TaskTypes> {
    queue: Vec<TaskTypes>,
}

impl<Task> Queue<Task> {
    pub fn new() -> Self {
        Queue { queue: Vec::new() }
    }
    pub fn enqueue(&mut self, item: Task) {
        self.queue.push(item)
    }

    pub fn dequeue(&mut self) -> Task {
        let de_task = self.queue.remove(0);
        for task in self.queue {
            if let Some(element) = task.dependencies.iter().position(|x| *x == task) {
                task.dependencies.remove(element);
            }
        }
        return de_task;
    }
    pub fn length(&self) -> usize {
        self.queue.len()
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
