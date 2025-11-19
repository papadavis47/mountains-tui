# Mountains

## For Inspiration and Mindfullness

### A Trail Running Training Log - _a digital tool to help runners get good at vert :)_

This is a training log to help trail runners get lighter. It grew out of my desire to improve my ability to go faster uphill and run on trails for many miles.

This app can help with food awareness - thereby helping runners lose weight - **my main motivation to create this software!**.

As currently implemented, **Mountains** requires a Turso Cloud account and database. This will sync with the `libsql` db created in the user's home directory: `~/.mountains/`

The simplest way to install currently is to clone the repo, use the example `.env.example` to make your own `.env` file, ( _which is listed in the `.gitignore` file with the repo_ ) fill in your credentials and build the application locally with: `cargo install --path .`

You can then create a `.mountains` directory in your `$HOME` or run the program for the first time and it will be created.

In order to use the app in any directory on the user's system post install - the program requires a `.env` file in the user's `~/.mountains/` directory for Turso Cloud.

Ater cloning the project, to quickly start using it with Turso Cloud anywhere on your system after credentials are entered into `.env`, do the following:

```shell

cd mountains
mkdir ~/.mountains
cp .env ~/.mountains/.env

```

I plan on updating the app as I go along.

Right now, I am simply using it for my own training.
