CREATE TABLE `students` (
    `id` TEXT NOT NULL PRIMARY KEY,
    `email` TEXT NOT NULL,
    `first_name` TEXT NOT NULL,
    `middle_initial` TEXT NOT NULL,
    `last_name` TEXT NOT NULL,
    `college` TEXT NOT NULL,
    `department` TEXT NOT NULL,
    `major` TEXT NOT NULL,
    `class` INTEGER NOT NULL,
    `graduation_semester` TEXT NOT NULL
);
