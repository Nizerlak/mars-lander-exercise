# mars-lander-exercise
![rust workflow](https://github.com/Nizerlak/mars-lander-exercise/actions/workflows/rust.yml/badge.svg?event=push)

Visual playground and solution to CodinGame's exercise: [Mars Lander: episode 3](https://www.codingame.com/training/expert/mars-lander-episode-3)
![mars-lander-solving](https://github.com/user-attachments/assets/aaa13b80-038f-45e3-ac47-be554f66b46d)

# How I try to solve the exercise
My approach is based on genetic algorithm (mainly inspired by [this awesome article](https://www.codingame.com/blog/genetic-algorithm-mars-lander/?utm_source=codingame&utm_medium=details-page&utm_campaign=cg-blog&utm_content=mars-lander-2)).
Whole project is made in Rust (+ some HTML and JS for [GUI Tool](#gui-tool)).

# Project structure
The project follows standard [Cargo layout](https://doc.rust-lang.org/cargo/guide/project-layout.html). Namely:
- [src](src) - contains core library for solving the excercise
- [examples/web_gui](examples/web_gui) - contains GUI tool for debugging (see [GUI Tool](#gui-tool))
- [examples/solve_sim](examples/solve_sim) - conatins thin executable which run solver until the solution is found + proides some runtime metrics

Also there are some bacis UTs and integration tests.

To maintain code sanity there are pre-commit hooks defined. They're used by CI. In order to install them locally:

1. Install pre-commit tool, e.g.
    ```shell
    pip install pre-commit
    ```
1. Install hooks
    ```shell
    pre-commit install
    ```

# GUI Tool
Is web application communicating with `web_gui` backend server with REST API.

How to run:

1. Run server backend
    ```shell
    cargo run --release --example web_gui examples/sim4.json examples/settings.json
    ```
1. Open [main.html](examples/web_gui/gui/main.html) in your browser (e.g. with help of VS Code Live Server). You Should see something like this
![image](https://github.com/user-attachments/assets/17658eff-9388-4d36-ac05-c3dedfe78ae2)
1. Have fun!
