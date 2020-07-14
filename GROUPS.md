# Flow For Starting Group Matching

- Run `!start group`
- Bot responds with `Initializing a group under user {}`
- Bot asks and user answers the following questions
  - How many people per group?
    - `n-n` for range
    - `n` for single
    - What to do if not even distribution?
  - How to group?
    - `r` for random
    - `w` weighted
    - `u` unweighted
  - If not random, flow for setting up grouping questions
    - What question do you want to ask?
    - What are the options for the response?
    - Group on similarity or difference?
      - `s`
      - `d`
    - If weighted, what weight from `1 (min) - 5 (max)`
  - Add another question, respond with `no` if not
- Return `GROUP_ID` for group matching

# Flow For Joining

- Run `!join group {GROUP_ID}`
- Discord name gets saved
- Bot asks and user answers questions

# Flow For Checking Who Is In

- Run `!find group {GROUP_ID}`
- Lists all users who have filled out form

# Flow For Closing The Grouping

- Run `!close group {GROUP_ID}`
- Check if user owns the group
  - If so, close
  - PM everyone who filled out the matching their group
    - Ask if satisfied with group

# Flow For Checking Groups

- Run `!check group {GROUP_ID}`
  - Return group for user if group exists and user has group
- Run `!check group {GROUP_ID} all`
  - Return all groups

===========================

# Flow For Deleting Groups

- Same as checking
