# Scheduler



## Getting started

To make it easy for you to get started with GitLab, here's a list of recommended next steps.

Already a pro? Just edit this README.md and make it your own. Want to make it easy? [Use the template at the bottom](#editing-this-readme)!

## Add your files

- [ ] [Create](https://docs.gitlab.com/ee/user/project/repository/web_editor.html#create-a-file) or [upload](https://docs.gitlab.com/ee/user/project/repository/web_editor.html#upload-a-file) files
- [ ] [Add files using the command line](https://docs.gitlab.com/ee/gitlab-basics/add-file.html#add-a-file-using-the-command-line) or push an existing Git repository with the following command:

```
cd existing_repo
git remote add origin http://git.cydevcloud.com/libs/taskmaster.git
git branch -M master
git push -uf origin master
```

## Integrate with your tools

- [ ] [Set up project integrations](http://git.cydevcloud.com/libs/taskmaster/-/settings/integrations)

## Collaborate with your team

- [ ] [Invite team members and collaborators](https://docs.gitlab.com/ee/user/project/members/)
- [ ] [Create a new merge request](https://docs.gitlab.com/ee/user/project/merge_requests/creating_merge_requests.html)
- [ ] [Automatically close issues from merge requests](https://docs.gitlab.com/ee/user/project/issues/managing_issues.html#closing-issues-automatically)
- [ ] [Enable merge request approvals](https://docs.gitlab.com/ee/user/project/merge_requests/approvals/)
- [ ] [Set auto-merge](https://docs.gitlab.com/ee/user/project/merge_requests/merge_when_pipeline_succeeds.html)

## Test and Deploy

Use the built-in continuous integration in GitLab.

- [ ] [Get started with GitLab CI/CD](https://docs.gitlab.com/ee/ci/quick_start/index.html)
- [ ] [Analyze your code for known vulnerabilities with Static Application Security Testing (SAST)](https://docs.gitlab.com/ee/user/application_security/sast/)
- [ ] [Deploy to Kubernetes, Amazon EC2, or Amazon ECS using Auto Deploy](https://docs.gitlab.com/ee/topics/autodevops/requirements.html)
- [ ] [Use pull-based deployments for improved Kubernetes management](https://docs.gitlab.com/ee/user/clusters/agent/)
- [ ] [Set up protected environments](https://docs.gitlab.com/ee/ci/environments/protected_environments.html)

***

# Editing this README

When you're ready to make this README your own, just edit this file and use the handy template below (or feel free to structure it however you want - this is just a starting point!). Thanks to [makeareadme.com](https://www.makeareadme.com/) for this template.

## Suggestions for a good README

Every project is different, so consider which of these sections apply to yours. The sections used in the template are suggestions for most open source projects. Also keep in mind that while a README can be too long and detailed, too long is better than too short. If you think your README is too long, consider utilizing another form of documentation rather than cutting out information.

## Name
Choose a self-explaining name for your project.

## Description
Let people know what your project can do specifically. Provide context and add a link to any reference visitors might be unfamiliar with. A list of Features or a Background subsection can also be added here. If there are alternatives to your project, this is a good place to list differentiating factors.

## Badges
On some READMEs, you may see small images that convey metadata, such as whether or not all the tests are passing for the project. You can use Shields to add some to your README. Many services also have instructions for adding a badge.

## Visuals
Depending on what you are making, it can be a good idea to include screenshots or even a video (you'll frequently see GIFs rather than actual videos). Tools like ttygif can help, but check out Asciinema for a more sophisticated method.

## Installation
Within a particular ecosystem, there may be a common way of installing things, such as using Yarn, NuGet, or Homebrew. However, consider the possibility that whoever is reading your README is a novice and would like more guidance. Listing specific steps helps remove ambiguity and gets people to using your project as quickly as possible. If it only runs in a specific context like a particular programming language version or operating system or has dependencies that have to be installed manually, also add a Requirements subsection.

## Usage
Use examples liberally, and show the expected output if you can. It's helpful to have inline the smallest example of usage that you can demonstrate, while providing links to more sophisticated examples if they are too long to reasonably include in the README.

## Support
Tell people where they can go to for help. It can be any combination of an issue tracker, a chat room, an email address, etc.

## Roadmap
If you have ideas for releases in the future, it is a good idea to list them in the README.

## Contributing
State if you are open to contributions and what your requirements are for accepting them.

For people who want to make changes to your project, it's helpful to have some documentation on how to get started. Perhaps there is a script that they should run or some environment variables that they need to set. Make these steps explicit. These instructions could also be useful to your future self.

You can also document commands to lint the code or run tests. These steps help to ensure high code quality and reduce the likelihood that the changes inadvertently break something. Having instructions for running tests is especially helpful if it requires external setup, such as starting a Selenium server for testing in a browser.

## Authors and acknowledgment
Show your appreciation to those who have contributed to the project.

## License
For open source projects, say how it is licensed.

## Project status
If you have run out of energy or time for your project, put a note at the top of the README saying that development has slowed down or stopped completely. Someone may choose to fork your project or volunteer to step in as a maintainer or owner, allowing your project to keep going. You can also make an explicit request for maintainers.

# Scheduler

[![Crates.io](https://img.shields.io/crates/v/scheduler.svg)](https://crates.io/crates/scheduler)
[![Documentation](https://docs.rs/scheduler/badge.svg)](https://docs.rs/scheduler)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A non-blocking task scheduler for Rust with a fluent API, supporting interval-based and cron-like scheduling. Built on top of Tokio, it provides an efficient and ergonomic way to schedule and manage recurring tasks in async Rust applications.

## Features

- **Non-blocking execution**: Tasks run asynchronously without blocking the main thread
- **Flexible scheduling**: Support for intervals (seconds, minutes, hours, days, weeks) and specific times
- **Fluent API**: Intuitive builder pattern for creating and scheduling tasks
- **Type-safe**: Leverages Rust's type system for compile-time guarantees
- **Task monitoring**: Track task status, execution history, and failures
- **Optional integrations**: Support for RabbitMQ (and Kafka coming soon)
- **Well-tested**: Comprehensive test suite and benchmarks

## Quick Start

Add to your `Cargo.toml`:
```toml
[dependencies]
scheduler = "0.1.0"
```

Basic usage:
```rust
use scheduler::{Scheduler, TaskBuilder};

#[tokio::main]
async fn main() {
    // Create a new scheduler
    let scheduler = Scheduler::new();
    
    // Start the scheduler
    let _rx = scheduler.start().await;
    
    // Create and add a task
    let task = TaskBuilder::new("my_task", || {
        println!("Task executed!");
        Ok(())
    })
    .every_minutes(5)  // Run every 5 minutes
    .at("10:30")      // Starting at 10:30
    .build();
    
    scheduler.add_task(task).await.unwrap();
    
    // Keep the program running
    tokio::signal::ctrl_c().await.unwrap();
}
```

## Advanced Usage

### Interval-based Scheduling
```rust
let task = TaskBuilder::new("interval_task", || Ok(()))
    .every_minutes(30)    // Every 30 minutes
    .build();
```

### Daily Tasks
```rust
let task = TaskBuilder::new("daily_task", || Ok(()))
    .daily()
    .at("15:00")         // At 3 PM
    .build();
```

### Weekly Tasks
```rust
let task = TaskBuilder::new("weekly_task", || Ok(()))
    .every_weeks(1)
    .monday()            // Run on Mondays
    .at("09:00")        // At 9 AM
    .build();
```

### Task Status Monitoring
```rust
let task_id = scheduler.add_task(task).await?;
let status = scheduler.get_task_status(&task_id).await?;

match status {
    TaskStatus::Running => println!("Task is running"),
    TaskStatus::Completed => println!("Task completed"),
    TaskStatus::Failed(err) => println!("Task failed: {}", err),
    TaskStatus::Pending => println!("Task is pending"),
}
```

## Optional Features

Enable RabbitMQ integration:
```toml
[dependencies]
scheduler = { version = "0.1.0", features = ["rabbitmq"] }
```

## Performance

The scheduler is designed to be lightweight and efficient. Benchmark results show minimal overhead:
- Task scheduling: ~500ns
- Task execution: ~1μs overhead
- Memory usage: ~200 bytes per task

## Contributing

We welcome contributions! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
