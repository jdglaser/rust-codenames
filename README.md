# Codewords

Codewords is a clone of the popular game Codenames I wrote as a hobby project. The backend is written in Rust using the actix-web framework and the frontend is written in React.js + Typescript. The app is currently hosted on Google Cloud Run [here](http://codenames.jarredapps.com/).

## Running Locally

Requirements:
+ Rust 1.62+
+ Node 18+
+ Docker

To startup locally:
+ With Docker:
  + run `docker compose up`
+ With rust and node for local development
  + open a terminal and run
    ```sh
    cd app
    cargo run
    ```
  + open another terminal and run
    ```sh
    cd frontend
    npm run dev
    ```
  + Go to [localhost:3000](http://localhost:3000/) in your browser to view the app