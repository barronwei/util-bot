# util-bot

The description of **Utility Software** on [Wikipedia](https://en.wikipedia.org/wiki/Utility_software#:~:text=Utility%20software%20is%20software%20designed,tasks%20that%20benefit%20ordinary%20users.) is 

>Utility software is software designed to help to analyze, configure, optimize or maintain a computer. It is used to support the computer infrastructure - in contrast to application software, which is aimed at directly performing tasks that benefit ordinary users.

That's exactlty what util-bot is set out to accomplish. Except that it's a bot, and it is designed to analyze, configure and optimize fellows' workflows

## What it does
We have implement 3 main features in the time we had:
 - Join a pool
   - This command streamlines the team generation process, something that is difficult to accomplish with 100+ fellows :laughing:
```
!utilbot pool start
!utilbot pool join
!utilbot pool skrt
!utilbot pool match
!utilbot pool check
```
 - Remind-a-bot
   - This command helps fellows set reminders for themselves or someone else. This could takes the form of a short text input within 
```
~remind @USER_HANDLE "_INSERT_MSG_HERE_" in 2mins OR
~remind @USER_HANDLE "_INSERT_MSG_HERE_" at 7:50pm
```
 - Get-help
   - This command seeks to get fellows get their tech related answers questions answered. It achieves this by storing a list of languages/tech that fellows are willing to assist with
```
!utilbot add [languages..]
!utilbot view
!utilbot clear
!utilbot get-help [language] [Question...]
```

## How we built it
We used:
 - [Rust](https://www.rust-lang.org/) as our language of choice
 - [PostgreSQL](https://www.postgresql.org/) to store users' information
 - [diesel](http://diesel.rs/) to help with SQL queries
 - [serenity](https://docs.rs/serenity/0.8.6/serenity/) for discord related functions
 - [chrono](https://docs.rs/chrono/0.4.13/chrono/) for a time library

## Challenges we ran into
Our main goal was to learn Rust since all of us had little to no experience prior. Rust is a statically and strongly typed systems programming language. With that came a lot of syntax and type related problems that made programming difficult. Ownership was a new concept to grasp that added fire to the flame. In addition, reading documentation was difficult as there was a lot of new and unfamiliar syntax and terminology being used.

## Accomplishments that we're proud of
We made a project in Rust :confetti_ball: ! William figured out how to manage threads, Barron and Rithvik worked extensively with PostgreSQL to properly manage the DB and achieve the streamlined team generation program. Mohammed was able to figure out the nuances of serenity and found ways of getting through the *Bot DMing* issues

## What we learned
Diving headfirst into new and daunting tech ain't that bad :grin:! There were a lot of **steep** ups and downs but it was all worth it. We were able to pick up and make a project with rust in a week, having no prior knowledge of the language. Rust docs are a lot easier to understand.

## What's next for util-bot
- Remind-a-bot
  - Hook this up to a db to save the reminders
  - Add spam detection (would be a counter, if it exceeds over 3 mentions stop queuing the mentions)
  - Add another regex detection for a keyword important or every which will ping a user for x amount of times for y duration that is specified by the user

- get-help
  - Add a `reply` function so that when a reciepient sends a message to the bot, it relays the message to the question asker. This avoid the use of a new direct message channels from being created

Adding more utility commands! The sky is the limit
