use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Todo {
    id: usize,
    title: String,
    description: String,
    completed: bool,
    created_at: DateTime<Local>,
    updated_at: DateTime<Local>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TodoList {
    todos: Vec<Todo>,
    next_id: usize,
}

impl TodoList {
    fn new() -> Self {
        TodoList {
            todos: Vec::new(),
            next_id: 1,
        }
    }

    fn add_todo(&mut self, title: String, description: String) {
        let now = Local::now();
        let todo = Todo {
            id: self.next_id,
            title,
            description,
            completed: false,
            created_at: now,
            updated_at: now,
        };
        self.todos.push(todo);
        self.next_id += 1;
    }

    fn get_todo(&self, id: usize) -> Option<&Todo> {
        self.todos.iter().find(|todo| todo.id == id)
    }

    fn get_todo_mut(&mut self, id: usize) -> Option<&mut Todo> {
        self.todos.iter_mut().find(|todo| todo.id == id)
    }

    fn edit_todo(&mut self, id: usize, title: String, description: String) -> bool {
        if let Some(todo) = self.get_todo_mut(id) {
            todo.title = title;
            todo.description = description;
            todo.updated_at = Local::now();
            true
        } else {
            false
        }
    }

    fn delete_todo(&mut self, id: usize) -> bool {
        let position = self.todos.iter().position(|todo| todo.id == id);
        if let Some(pos) = position {
            self.todos.remove(pos);
            true
        } else {
            false
        }
    }

    fn toggle_completed(&mut self, id: usize) -> bool {
        if let Some(todo) = self.get_todo_mut(id) {
            todo.completed = !todo.completed;
            todo.updated_at = Local::now();
            true
        } else {
            false
        }
    }

    fn list_todos(&self) {
        if self.todos.is_empty() {
            println!("No todos found.");
            return;
        }

        println!("{:<5} {:<30} {:<50} {:<10}", "ID", "TITLE", "DESCRIPTION", "STATUS");
        println!("{}", "-".repeat(100));

        for todo in &self.todos {
            let status = if todo.completed { "Completed" } else { "Pending" };
            println!("{:<5} {:<30} {:<50} {:<10}",
                todo.id,
                truncate(&todo.title, 27),
                truncate(&todo.description, 47),
                status
            );
        }
    }

    fn save_to_file(&self, filename: &str) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(filename)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    fn load_from_file(filename: &str) -> io::Result<Self> {
        if !Path::new(filename).exists() {
            return Ok(TodoList::new());
        }

        let mut file = File::open(filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let todo_list: TodoList = serde_json::from_str(&contents)?;
        Ok(todo_list)
    }
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        s.to_string()
    } else {
        format!("{}...", &s[0..max_chars-3])
    }
}

fn get_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

fn get_confirmation(prompt: &str) -> bool {
    loop {
        let input = get_input(&format!("{} (y/n): ", prompt)).to_lowercase();
        match input.as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Please enter 'y' or 'n'"),
        }
    }
}

fn display_menu() {
    println!("\n===== TODO APP =====");
    println!("1. List all todos");
    println!("2. Add a new todo");
    println!("3. Edit a todo");
    println!("4. Toggle todo completion status");
    println!("5. Delete a todo");
    println!("0. Exit");
    println!("====================");
}

fn main() -> io::Result<()> {
    const FILENAME: &str = "todos.json";
    let mut todo_list = TodoList::load_from_file(FILENAME).unwrap_or_else(|_| {
        println!("Creating new todo list.");
        TodoList::new()
    });

    loop {
        display_menu();
        let choice = get_input("Enter your choice:");

        match choice.as_str() {
            "1" => {
                println!("\n--- All Todos ---");
                todo_list.list_todos();
            },
            "2" => {
                let title = get_input("Enter todo title:");
                let description = get_input("Enter todo description:");
                todo_list.add_todo(title, description);
                println!("Todo added successfully!");
                todo_list.save_to_file(FILENAME)?;
            },
            "3" => {
                todo_list.list_todos();
                let id_str = get_input("Enter the ID of the todo to edit:");
                if let Ok(id) = id_str.parse::<usize>() {
                    if let Some(todo) = todo_list.get_todo(id) {
                        println!("Editing todo: {}", todo.title);
                        let title = get_input(&format!("Enter new title (current: {}):", todo.title));
                        let description = get_input(&format!("Enter new description (current: {}):", todo.description));

                        if todo_list.edit_todo(id, title, description) {
                            println!("Todo updated successfully!");
                            todo_list.save_to_file(FILENAME)?;
                        } else {
                            println!("Failed to update todo.");
                        }
                    } else {
                        println!("Todo with ID {} not found.", id);
                    }
                } else {
                    println!("Invalid ID format.");
                }
            },
            "4" => {
                todo_list.list_todos();
                let id_str = get_input("Enter the ID of the todo to toggle completion status:");
                if let Ok(id) = id_str.parse::<usize>() {
                    if todo_list.toggle_completed(id) {
                        println!("Todo status toggled successfully!");
                        todo_list.save_to_file(FILENAME)?;
                    } else {
                        println!("Todo with ID {} not found.", id);
                    }
                } else {
                    println!("Invalid ID format.");
                }
            },
            "5" => {
                todo_list.list_todos();
                let id_str = get_input("Enter the ID of the todo to delete:");
                if let Ok(id) = id_str.parse::<usize>() {
                    if let Some(todo) = todo_list.get_todo(id) {
                        println!("You are about to delete the following todo:");
                        println!("Title: {}", todo.title);
                        println!("Description: {}", todo.description);

                        if get_confirmation("Are you sure you want to delete this todo?") {
                            if todo_list.delete_todo(id) {
                                println!("Todo deleted successfully!");
                                todo_list.save_to_file(FILENAME)?;
                            } else {
                                println!("Failed to delete todo.");
                            }
                        } else {
                            println!("Deletion cancelled.");
                        }
                    } else {
                        println!("Todo with ID {} not found.", id);
                    }
                } else {
                    println!("Invalid ID format.");
                }
            },
            "0" => {
                println!("Exiting. Goodbye!");
                break;
            },
            _ => println!("Invalid choice. Please try again."),
        }
    }

    Ok(())
}